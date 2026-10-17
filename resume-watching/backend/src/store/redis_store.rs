use std::collections::HashMap;

use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};

use super::{PlayheadStore, StoreFuture};
use crate::error::StoreError;
use crate::model::Playhead;

const LWW_UPSERT: &str = r"
local cur = redis.call('HGET', KEYS[2], ARGV[1])
if cur and tonumber(cur) >= tonumber(ARGV[2]) then return 0 end
redis.call('HSET', KEYS[2], ARGV[1], ARGV[2])
redis.call('HSET', KEYS[1], ARGV[1], ARGV[3])
return 1
";

pub struct RedisStore {
    conn: ConnectionManager,
    script: Script,
}

fn data_key(profile_id: &str) -> String {
    format!("ph:{profile_id}")
}

fn ts_key(profile_id: &str) -> String {
    format!("ts:{profile_id}")
}

impl RedisStore {
    pub async fn connect(url: &str) -> Result<Self, StoreError> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn, script: Script::new(LWW_UPSERT) })
    }
}

impl PlayheadStore for RedisStore {
    fn name(&self) -> &'static str {
        "redis"
    }

    fn upsert<'a>(&'a self, playhead: &'a Playhead) -> StoreFuture<'a, ()> {
        Box::pin(async move {
            let json = serde_json::to_string(playhead)?;
            let mut conn = self.conn.clone();
            let _: i64 = self
                .script
                .key(data_key(&playhead.profile_id))
                .key(ts_key(&playhead.profile_id))
                .arg(&playhead.title_id)
                .arg(playhead.client_ts_ms)
                .arg(json)
                .invoke_async(&mut conn)
                .await?;
            Ok(())
        })
    }

    fn list<'a>(&'a self, profile_id: &'a str) -> StoreFuture<'a, Vec<Playhead>> {
        Box::pin(async move {
            let mut conn = self.conn.clone();
            let raw: HashMap<String, String> = conn.hgetall(data_key(profile_id)).await?;
            raw.values()
                .map(|v| serde_json::from_str(v).map_err(StoreError::from))
                .collect()
        })
    }

    fn clear<'a>(&'a self, profile_id: &'a str) -> StoreFuture<'a, ()> {
        Box::pin(async move {
            let mut conn = self.conn.clone();
            let _: i64 = conn.del(&[data_key(profile_id), ts_key(profile_id)]).await?;
            Ok(())
        })
    }
}
