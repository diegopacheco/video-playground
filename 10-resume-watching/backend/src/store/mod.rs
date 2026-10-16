pub mod redis_store;
pub mod scylla_store;

use std::future::Future;
use std::pin::Pin;

use crate::error::StoreError;
use crate::model::Playhead;

pub use redis_store::RedisStore;
pub use scylla_store::ScyllaStore;

pub type StoreFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, StoreError>> + Send + 'a>>;

pub trait PlayheadStore: Send + Sync {
    fn name(&self) -> &'static str;
    fn upsert<'a>(&'a self, playhead: &'a Playhead) -> StoreFuture<'a, ()>;
    fn list<'a>(&'a self, profile_id: &'a str) -> StoreFuture<'a, Vec<Playhead>>;
    fn clear<'a>(&'a self, profile_id: &'a str) -> StoreFuture<'a, ()>;
}

pub async fn connect(kind: &str, redis_url: &str, scylla_node: &str) -> Result<Box<dyn PlayheadStore>, StoreError> {
    match kind {
        "redis" => Ok(Box::new(RedisStore::connect(redis_url).await?)),
        "scylla" => Ok(Box::new(ScyllaStore::connect(scylla_node).await?)),
        other => Err(StoreError::Config(format!("unknown store {other}, use redis or scylla"))),
    }
}
