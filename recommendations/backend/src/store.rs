pub mod mem;
pub mod pg;

use crate::error::AppError;
use crate::types::{NewTitle, NewUser, Title, User};
use std::future::Future;

pub trait Store: Send + Sync + 'static {
    fn titles(&self) -> impl Future<Output = Result<Vec<Title>, AppError>> + Send;
    fn users(&self) -> impl Future<Output = Result<Vec<User>, AppError>> + Send;
    fn watches(&self) -> impl Future<Output = Result<Vec<(i32, i32)>, AppError>> + Send;
    fn insert_titles(&self, titles: &[NewTitle]) -> impl Future<Output = Result<Vec<Title>, AppError>> + Send;
    fn insert_users(&self, users: &[NewUser]) -> impl Future<Output = Result<Vec<User>, AppError>> + Send;
    fn insert_watches(&self, watches: &[(i32, i32)]) -> impl Future<Output = Result<(), AppError>> + Send;
    fn save_embeddings(&self, items: &[(i32, Vec<f32>)]) -> impl Future<Output = Result<(), AppError>> + Send;
    fn nearest(&self, query: &[f32], k: usize) -> impl Future<Output = Result<Vec<(i32, f32)>, AppError>> + Send;
}
