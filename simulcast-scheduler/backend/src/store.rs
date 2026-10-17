use crate::entitlement::UnlockPlan;
use crate::error::AppError;
use crate::model::Episode;
use std::collections::BTreeMap;
use std::sync::{Arc, PoisonError, RwLock};

pub trait ScheduleStore: Send + Sync {
    fn upsert(&self, episode: Episode) -> Result<Arc<UnlockPlan>, AppError>;
    fn get(&self, id: &str) -> Result<Episode, AppError>;
    fn plan(&self, id: &str) -> Result<Arc<UnlockPlan>, AppError>;
    fn list(&self) -> Result<Vec<Episode>, AppError>;
}

#[derive(Default)]
pub struct InMemoryStore {
    rows: RwLock<BTreeMap<String, (Episode, Arc<UnlockPlan>)>>,
}

impl ScheduleStore for InMemoryStore {
    fn upsert(&self, episode: Episode) -> Result<Arc<UnlockPlan>, AppError> {
        let plan = Arc::new(UnlockPlan::build(&episode)?);
        let mut rows = self.rows.write().unwrap_or_else(PoisonError::into_inner);
        rows.insert(episode.id.clone(), (episode, plan.clone()));
        Ok(plan)
    }

    fn get(&self, id: &str) -> Result<Episode, AppError> {
        let rows = self.rows.read().unwrap_or_else(PoisonError::into_inner);
        rows.get(id)
            .map(|(e, _)| e.clone())
            .ok_or_else(|| AppError::NotFound(id.to_string()))
    }

    fn plan(&self, id: &str) -> Result<Arc<UnlockPlan>, AppError> {
        let rows = self.rows.read().unwrap_or_else(PoisonError::into_inner);
        rows.get(id)
            .map(|(_, p)| p.clone())
            .ok_or_else(|| AppError::NotFound(id.to_string()))
    }

    fn list(&self) -> Result<Vec<Episode>, AppError> {
        let rows = self.rows.read().unwrap_or_else(PoisonError::into_inner);
        Ok(rows.values().map(|(e, _)| e.clone()).collect())
    }
}
