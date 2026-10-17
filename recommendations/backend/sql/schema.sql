CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS titles (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    genres TEXT[] NOT NULL,
    tags TEXT[] NOT NULL,
    synopsis TEXT NOT NULL,
    year INT NOT NULL,
    is_new BOOLEAN NOT NULL DEFAULT FALSE,
    embedding vector(256)
);

CREATE INDEX IF NOT EXISTS titles_embedding_hnsw ON titles USING hnsw (embedding vector_cosine_ops);

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    cluster TEXT,
    picked_genres TEXT[] NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS watches (
    id BIGSERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),
    title_id INT NOT NULL REFERENCES titles(id),
    watched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, title_id)
);
