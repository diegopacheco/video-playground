use std::time::{SystemTime, UNIX_EPOCH};

use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use scylla::statement::prepared::PreparedStatement;

use super::{PlayheadStore, StoreFuture};
use crate::error::{StoreError, scylla_err};
use crate::model::Playhead;

const KEYSPACE: &str = "CREATE KEYSPACE IF NOT EXISTS resume WITH replication = {'class': 'NetworkTopologyStrategy', 'replication_factor': 1}";
const TABLE: &str = "CREATE TABLE IF NOT EXISTS resume.playheads (profile_id text, title_id text, user_id text, device_id text, episode_id text, position_secs int, duration_secs int, client_ts_ms bigint, PRIMARY KEY (profile_id, title_id))";
const INSERT: &str = "INSERT INTO resume.playheads (profile_id, title_id, user_id, device_id, episode_id, position_secs, duration_secs, client_ts_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?) USING TIMESTAMP ?";
const SELECT: &str = "SELECT user_id, profile_id, device_id, title_id, episode_id, position_secs, duration_secs, client_ts_ms FROM resume.playheads WHERE profile_id = ?";
const DELETE: &str = "DELETE FROM resume.playheads USING TIMESTAMP ? WHERE profile_id = ?";

type Row = (String, String, String, String, String, i32, i32, i64);

pub struct ScyllaStore {
    session: Session,
    insert: PreparedStatement,
    select: PreparedStatement,
    delete: PreparedStatement,
}

impl ScyllaStore {
    pub async fn connect(node: &str) -> Result<Self, StoreError> {
        let session = SessionBuilder::new().known_node(node).build().await.map_err(scylla_err)?;
        session.query_unpaged(KEYSPACE, &[]).await.map_err(scylla_err)?;
        session.query_unpaged(TABLE, &[]).await.map_err(scylla_err)?;
        let insert = session.prepare(INSERT).await.map_err(scylla_err)?;
        let select = session.prepare(SELECT).await.map_err(scylla_err)?;
        let delete = session.prepare(DELETE).await.map_err(scylla_err)?;
        Ok(Self { session, insert, select, delete })
    }
}

fn to_int(v: u32) -> Result<i32, StoreError> {
    i32::try_from(v).map_err(scylla_err)
}

fn from_int(v: i32) -> Result<u32, StoreError> {
    u32::try_from(v).map_err(scylla_err)
}

impl PlayheadStore for ScyllaStore {
    fn name(&self) -> &'static str {
        "scylla"
    }

    fn upsert<'a>(&'a self, p: &'a Playhead) -> StoreFuture<'a, ()> {
        Box::pin(async move {
            let values = (
                &p.profile_id,
                &p.title_id,
                &p.user_id,
                &p.device_id,
                &p.episode_id,
                to_int(p.position_secs)?,
                to_int(p.duration_secs)?,
                p.client_ts_ms,
                p.client_ts_ms * 1000,
            );
            self.session.execute_unpaged(&self.insert, values).await.map_err(scylla_err)?;
            Ok(())
        })
    }

    fn list<'a>(&'a self, profile_id: &'a str) -> StoreFuture<'a, Vec<Playhead>> {
        Box::pin(async move {
            let result = self
                .session
                .execute_unpaged(&self.select, (profile_id,))
                .await
                .map_err(scylla_err)?
                .into_rows_result()
                .map_err(scylla_err)?;
            let mut out = Vec::new();
            for row in result.rows::<Row>().map_err(scylla_err)? {
                let (user_id, profile_id, device_id, title_id, episode_id, pos, dur, ts) = row.map_err(scylla_err)?;
                out.push(Playhead {
                    user_id,
                    profile_id,
                    device_id,
                    title_id,
                    episode_id,
                    position_secs: from_int(pos)?,
                    duration_secs: from_int(dur)?,
                    client_ts_ms: ts,
                });
            }
            Ok(out)
        })
    }

    fn clear<'a>(&'a self, profile_id: &'a str) -> StoreFuture<'a, ()> {
        Box::pin(async move {
            let newest = self.list(profile_id).await?.iter().map(|p| p.client_ts_ms * 1000).max();
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(scylla_err)?.as_micros() as i64;
            let tombstone = newest.map_or(now, |ts| ts.max(now) + 1);
            self.session.execute_unpaged(&self.delete, (tombstone, profile_id)).await.map_err(scylla_err)?;
            Ok(())
        })
    }
}
