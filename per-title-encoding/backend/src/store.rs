use crate::error::Result;
use crate::model::Analysis;
use std::path::PathBuf;
use std::sync::Mutex;

pub trait AnalysisStore: Send + Sync {
    fn load(&self) -> Result<Option<Analysis>>;
    fn save(&self, analysis: &Analysis) -> Result<()>;
}

pub struct JsonFileStore {
    pub path: PathBuf,
}

impl AnalysisStore for JsonFileStore {
    fn load(&self) -> Result<Option<Analysis>> {
        if !self.path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&self.path)?;
        Ok(Some(serde_json::from_str(&text)?))
    }

    fn save(&self, analysis: &Analysis) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(analysis)?)?;
        std::fs::rename(tmp, &self.path)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct MemoryStore {
    value: Mutex<Option<Analysis>>,
}

impl AnalysisStore for MemoryStore {
    fn load(&self) -> Result<Option<Analysis>> {
        Ok(self.value.lock().map(|v| v.clone()).unwrap_or(None))
    }

    fn save(&self, analysis: &Analysis) -> Result<()> {
        if let Ok(mut v) = self.value.lock() {
            *v = Some(analysis.clone());
        }
        Ok(())
    }
}
