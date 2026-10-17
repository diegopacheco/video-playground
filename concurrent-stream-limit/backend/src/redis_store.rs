use crate::error::LeaseError;
use crate::lease::{AcquireOutcome, AcquireRequest, Lease, LeaseStore};
use redis::Script;
use redis::aio::ConnectionManager;

pub struct RedisLeaseStore {
    conn: ConnectionManager,
    prefix: String,
    acquire: Script,
    heartbeat: Script,
    release: Script,
    list: Script,
}

impl RedisLeaseStore {
    pub async fn connect(url: &str, prefix: &str) -> Result<RedisLeaseStore, LeaseError> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(RedisLeaseStore {
            conn,
            prefix: prefix.to_string(),
            acquire: Script::new(include_str!("lua/acquire.lua")),
            heartbeat: Script::new(include_str!("lua/heartbeat.lua")),
            release: Script::new(include_str!("lua/release.lua")),
            list: Script::new(include_str!("lua/list.lua")),
        })
    }

    fn keys(&self, account: &str) -> (String, String) {
        (
            format!("{}:{{{}}}:leases", self.prefix, account),
            format!("{}:{{{}}}:started", self.prefix, account),
        )
    }
}

fn num(value: &str) -> Result<u64, LeaseError> {
    value
        .parse::<f64>()
        .map(|v| v as u64)
        .map_err(|_| LeaseError::BadReply(value.to_string()))
}

fn parse_acquire(device: &str, reply: Vec<String>) -> Result<AcquireOutcome, LeaseError> {
    match reply.first().map(String::as_str) {
        Some("granted") if reply.len() >= 3 => Ok(AcquireOutcome::Granted {
            lease: Lease {
                device: device.to_string(),
                started_at_ms: num(&reply[1])?,
                expires_at_ms: num(&reply[2])?,
            },
            evicted: reply[3..].to_vec(),
        }),
        Some("rejected") if reply.len() == 2 => Ok(AcquireOutcome::Rejected {
            active: num(&reply[1])? as usize,
        }),
        _ => Err(LeaseError::BadReply(reply.join(","))),
    }
}

fn parse_list(reply: Vec<String>) -> Result<Vec<Lease>, LeaseError> {
    if !reply.len().is_multiple_of(3) {
        return Err(LeaseError::BadReply(reply.join(",")));
    }
    let mut out = reply
        .chunks(3)
        .map(|c| {
            Ok(Lease {
                device: c[0].clone(),
                started_at_ms: num(&c[1])?,
                expires_at_ms: num(&c[2])?,
            })
        })
        .collect::<Result<Vec<Lease>, LeaseError>>()?;
    out.sort_by(|a, b| (a.started_at_ms, &a.device).cmp(&(b.started_at_ms, &b.device)));
    Ok(out)
}

impl LeaseStore for RedisLeaseStore {
    async fn acquire(&self, req: AcquireRequest<'_>) -> Result<AcquireOutcome, LeaseError> {
        let (leases, started) = self.keys(req.account);
        let mut conn = self.conn.clone();
        let reply: Vec<String> = self
            .acquire
            .key(leases)
            .key(started)
            .arg(req.device)
            .arg(req.now_ms)
            .arg(req.ttl_ms)
            .arg(req.limit)
            .arg(req.policy.as_str())
            .invoke_async(&mut conn)
            .await?;
        parse_acquire(req.device, reply)
    }

    async fn heartbeat(
        &self,
        account: &str,
        device: &str,
        now_ms: u64,
        ttl_ms: u64,
    ) -> Result<Option<Lease>, LeaseError> {
        let (leases, started) = self.keys(account);
        let mut conn = self.conn.clone();
        let reply: Vec<String> = self
            .heartbeat
            .key(leases)
            .key(started)
            .arg(device)
            .arg(now_ms)
            .arg(ttl_ms)
            .invoke_async(&mut conn)
            .await?;
        match reply.as_slice() {
            [] => Ok(None),
            [since, expires] => Ok(Some(Lease {
                device: device.to_string(),
                started_at_ms: num(since)?,
                expires_at_ms: num(expires)?,
            })),
            _ => Err(LeaseError::BadReply(reply.join(","))),
        }
    }

    async fn release(&self, account: &str, device: &str) -> Result<bool, LeaseError> {
        let (leases, started) = self.keys(account);
        let mut conn = self.conn.clone();
        let reply: String = self
            .release
            .key(leases)
            .key(started)
            .arg(device)
            .invoke_async(&mut conn)
            .await?;
        Ok(reply == "1")
    }

    async fn list(&self, account: &str, now_ms: u64) -> Result<Vec<Lease>, LeaseError> {
        let (leases, started) = self.keys(account);
        let mut conn = self.conn.clone();
        let reply: Vec<String> = self
            .list
            .key(leases)
            .key(started)
            .arg(now_ms)
            .invoke_async(&mut conn)
            .await?;
        parse_list(reply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn granted_reply_carries_every_evicted_device_so_clients_can_be_told() {
        let out = parse_acquire("tv", s(&["granted", "10", "20", "phone", "tab"])).unwrap();
        assert_eq!(
            out,
            AcquireOutcome::Granted {
                lease: Lease {
                    device: "tv".into(),
                    started_at_ms: 10,
                    expires_at_ms: 20
                },
                evicted: vec!["phone".into(), "tab".into()],
            }
        );
    }

    #[test]
    fn malformed_replies_fail_loud_instead_of_granting() {
        assert!(parse_acquire("tv", s(&["granted", "10"])).is_err());
        assert!(parse_acquire("tv", s(&["maybe"])).is_err());
        assert!(parse_list(s(&["tv", "1"])).is_err());
    }
}
