use super::Store;
use crate::embed::cosine;
use crate::error::AppError;
use crate::types::{NewTitle, NewUser, Title, User};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

#[derive(Default)]
struct Inner {
    titles: Vec<Title>,
    users: Vec<User>,
    watches: Vec<(i32, i32)>,
    embeddings: HashMap<i32, Vec<f32>>,
}

#[derive(Default)]
pub struct MemStore {
    inner: Mutex<Inner>,
}

impl MemStore {
    fn lock(&self) -> Result<MutexGuard<'_, Inner>, AppError> {
        self.inner.lock().map_err(|_| AppError::Poisoned)
    }
}

impl Store for MemStore {
    async fn titles(&self) -> Result<Vec<Title>, AppError> {
        Ok(self.lock()?.titles.clone())
    }

    async fn users(&self) -> Result<Vec<User>, AppError> {
        Ok(self.lock()?.users.clone())
    }

    async fn watches(&self) -> Result<Vec<(i32, i32)>, AppError> {
        Ok(self.lock()?.watches.clone())
    }

    async fn insert_titles(&self, titles: &[NewTitle]) -> Result<Vec<Title>, AppError> {
        let mut inner = self.lock()?;
        let mut out = Vec::new();
        for t in titles {
            if inner.titles.iter().any(|x| x.name == t.name) {
                return Err(AppError::Invalid(format!("title {} already exists", t.name)));
            }
            let title = Title {
                id: inner.titles.len() as i32 + 1,
                name: t.name.clone(),
                genres: t.genres.clone(),
                tags: t.tags.clone(),
                synopsis: t.synopsis.clone(),
                year: t.year,
                is_new: t.is_new,
            };
            inner.titles.push(title.clone());
            out.push(title);
        }
        Ok(out)
    }

    async fn insert_users(&self, users: &[NewUser]) -> Result<Vec<User>, AppError> {
        let mut inner = self.lock()?;
        let mut out = Vec::new();
        for u in users {
            let user = User {
                id: inner.users.len() as i32 + 1,
                name: u.name.clone(),
                cluster: u.cluster.clone(),
                picked_genres: u.picked_genres.clone(),
            };
            inner.users.push(user.clone());
            out.push(user);
        }
        Ok(out)
    }

    async fn insert_watches(&self, watches: &[(i32, i32)]) -> Result<(), AppError> {
        let mut inner = self.lock()?;
        for w in watches {
            if !inner.watches.contains(w) {
                inner.watches.push(*w);
            }
        }
        Ok(())
    }

    async fn save_embeddings(&self, items: &[(i32, Vec<f32>)]) -> Result<(), AppError> {
        let mut inner = self.lock()?;
        for (id, v) in items {
            inner.embeddings.insert(*id, v.clone());
        }
        Ok(())
    }

    async fn nearest(&self, query: &[f32], k: usize) -> Result<Vec<(i32, f32)>, AppError> {
        let inner = self.lock()?;
        let mut out: Vec<(i32, f32)> = inner.embeddings.iter().map(|(id, v)| (*id, 1.0 - cosine(query, v))).collect();
        out.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        out.truncate(k);
        Ok(out)
    }
}
