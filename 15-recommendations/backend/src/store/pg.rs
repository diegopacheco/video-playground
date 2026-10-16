use super::Store;
use crate::embed::to_pgvector;
use crate::error::AppError;
use crate::types::{NewTitle, NewUser, Title, User};
use tokio_postgres::{Client, NoTls, Row};

const SCHEMA: &str = include_str!("../../sql/schema.sql");
const TITLE_COLUMNS: &str = "id, name, genres, tags, synopsis, year, is_new";

pub struct PgStore {
    client: Client,
}

fn title(row: &Row) -> Title {
    Title {
        id: row.get(0),
        name: row.get(1),
        genres: row.get(2),
        tags: row.get(3),
        synopsis: row.get(4),
        year: row.get(5),
        is_new: row.get(6),
    }
}

fn user(row: &Row) -> User {
    User { id: row.get(0), name: row.get(1), cluster: row.get(2), picked_genres: row.get(3) }
}

impl PgStore {
    pub async fn connect(url: &str) -> Result<Self, AppError> {
        let (client, connection) = tokio_postgres::connect(url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("postgres connection error: {e}");
            }
        });
        client.batch_execute(SCHEMA).await?;
        Ok(Self { client })
    }

    pub async fn reset(&self) -> Result<(), AppError> {
        self.client.batch_execute("TRUNCATE watches, users, titles RESTART IDENTITY CASCADE").await?;
        Ok(())
    }

    pub async fn explain_nearest(&self, query: &[f32], k: usize) -> Result<Vec<String>, AppError> {
        let sql = "EXPLAIN SELECT id FROM titles ORDER BY embedding <=> $1::text::vector LIMIT $2";
        let rows = self.client.query(sql, &[&to_pgvector(query), &(k as i64)]).await?;
        Ok(rows.iter().map(|r| r.get(0)).collect())
    }
}

impl Store for PgStore {
    async fn titles(&self) -> Result<Vec<Title>, AppError> {
        let sql = format!("SELECT {TITLE_COLUMNS} FROM titles ORDER BY id");
        Ok(self.client.query(&sql, &[]).await?.iter().map(title).collect())
    }

    async fn users(&self) -> Result<Vec<User>, AppError> {
        let rows = self.client.query("SELECT id, name, cluster, picked_genres FROM users ORDER BY id", &[]).await?;
        Ok(rows.iter().map(user).collect())
    }

    async fn watches(&self) -> Result<Vec<(i32, i32)>, AppError> {
        let rows = self.client.query("SELECT user_id, title_id FROM watches ORDER BY id", &[]).await?;
        Ok(rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    async fn insert_titles(&self, titles: &[NewTitle]) -> Result<Vec<Title>, AppError> {
        let sql = format!(
            "INSERT INTO titles (name, genres, tags, synopsis, year, is_new) VALUES ($1, $2, $3, $4, $5, $6) RETURNING {TITLE_COLUMNS}"
        );
        let stmt = self.client.prepare(&sql).await?;
        let mut out = Vec::with_capacity(titles.len());
        for t in titles {
            let row = self.client.query_one(&stmt, &[&t.name, &t.genres, &t.tags, &t.synopsis, &t.year, &t.is_new]).await?;
            out.push(title(&row));
        }
        Ok(out)
    }

    async fn insert_users(&self, users: &[NewUser]) -> Result<Vec<User>, AppError> {
        let stmt = self
            .client
            .prepare("INSERT INTO users (name, cluster, picked_genres) VALUES ($1, $2, $3) RETURNING id, name, cluster, picked_genres")
            .await?;
        let mut out = Vec::with_capacity(users.len());
        for u in users {
            let row = self.client.query_one(&stmt, &[&u.name, &u.cluster, &u.picked_genres]).await?;
            out.push(user(&row));
        }
        Ok(out)
    }

    async fn insert_watches(&self, watches: &[(i32, i32)]) -> Result<(), AppError> {
        let users: Vec<i32> = watches.iter().map(|w| w.0).collect();
        let titles: Vec<i32> = watches.iter().map(|w| w.1).collect();
        let sql = "INSERT INTO watches (user_id, title_id) SELECT u, t FROM UNNEST($1::int[], $2::int[]) AS x(u, t) ON CONFLICT DO NOTHING";
        self.client.execute(sql, &[&users, &titles]).await?;
        Ok(())
    }

    async fn save_embeddings(&self, items: &[(i32, Vec<f32>)]) -> Result<(), AppError> {
        let ids: Vec<i32> = items.iter().map(|i| i.0).collect();
        let vectors: Vec<String> = items.iter().map(|i| to_pgvector(&i.1)).collect();
        let sql = "UPDATE titles SET embedding = d.v::vector FROM UNNEST($1::int[], $2::text[]) AS d(id, v) WHERE titles.id = d.id";
        self.client.execute(sql, &[&ids, &vectors]).await?;
        Ok(())
    }

    async fn nearest(&self, query: &[f32], k: usize) -> Result<Vec<(i32, f32)>, AppError> {
        let sql = "SELECT id, (embedding <=> $1::text::vector)::float4 FROM titles WHERE embedding IS NOT NULL ORDER BY embedding <=> $1::text::vector LIMIT $2";
        let rows = self.client.query(sql, &[&to_pgvector(query), &(k as i64)]).await?;
        Ok(rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }
}
