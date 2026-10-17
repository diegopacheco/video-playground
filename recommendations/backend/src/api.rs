use crate::error::AppError;
use crate::eval::EvalRow;
use crate::seed::{CLUSTERS, GENRES};
use crate::service::{Recommendations, Service, TitleDetail, TitleView, UserDetail, UserView};
use crate::store::Store;
use crate::types::{NewTitle, Title, User};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

type Shared<S> = State<Arc<Service<S>>>;

#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    #[serde(default)]
    pub genres: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateWatch {
    pub title_id: i32,
}

#[derive(Deserialize)]
pub struct AlgoQuery {
    pub algo: Option<String>,
}

#[derive(Serialize)]
pub struct Cluster {
    pub name: &'static str,
    pub genres: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct Genres {
    pub genres: Vec<&'static str>,
    pub clusters: Vec<Cluster>,
}

pub fn router<S: Store>(service: Arc<Service<S>>) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/genres", get(genres))
        .route("/api/eval", get(eval::<S>))
        .route("/api/titles", get(titles::<S>).post(create_title::<S>))
        .route("/api/titles/{id}", get(title::<S>))
        .route("/api/users", get(users::<S>).post(create_user::<S>))
        .route("/api/users/{id}", get(user::<S>))
        .route("/api/users/{id}/watches", post(add_watch::<S>))
        .route("/api/users/{id}/recommendations", get(recommendations::<S>))
        .with_state(service)
}

async fn genres() -> Json<Genres> {
    Json(Genres {
        genres: GENRES.iter().map(|g| g.name).collect(),
        clusters: CLUSTERS.iter().map(|(name, gs)| Cluster { name, genres: gs.to_vec() }).collect(),
    })
}

async fn eval<S: Store>(State(s): Shared<S>) -> Json<Vec<EvalRow>> {
    Json(s.eval())
}

async fn titles<S: Store>(State(s): Shared<S>) -> Result<Json<Vec<TitleView>>, AppError> {
    Ok(Json(s.titles()?))
}

async fn title<S: Store>(State(s): Shared<S>, Path(id): Path<i32>) -> Result<Json<TitleDetail>, AppError> {
    Ok(Json(s.title(id).await?))
}

async fn create_title<S: Store>(State(s): Shared<S>, Json(body): Json<NewTitle>) -> Result<(StatusCode, Json<Title>), AppError> {
    Ok((StatusCode::CREATED, Json(s.create_title(body).await?)))
}

async fn users<S: Store>(State(s): Shared<S>) -> Result<Json<Vec<UserView>>, AppError> {
    Ok(Json(s.users()?))
}

async fn user<S: Store>(State(s): Shared<S>, Path(id): Path<i32>) -> Result<Json<UserDetail>, AppError> {
    Ok(Json(s.user(id)?))
}

async fn create_user<S: Store>(State(s): Shared<S>, Json(body): Json<CreateUser>) -> Result<(StatusCode, Json<User>), AppError> {
    Ok((StatusCode::CREATED, Json(s.create_user(body.name, body.genres).await?)))
}

async fn add_watch<S: Store>(State(s): Shared<S>, Path(id): Path<i32>, Json(body): Json<CreateWatch>) -> Result<StatusCode, AppError> {
    s.add_watch(id, body.title_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn recommendations<S: Store>(
    State(s): Shared<S>,
    Path(id): Path<i32>,
    Query(q): Query<AlgoQuery>,
) -> Result<Json<Recommendations>, AppError> {
    Ok(Json(s.recommendations(id, q.algo.as_deref().unwrap_or("hybrid")).await?))
}
