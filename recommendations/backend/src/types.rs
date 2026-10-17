use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Title {
    pub id: i32,
    pub name: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub synopsis: String,
    pub year: i32,
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTitle {
    pub name: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub synopsis: String,
    pub year: i32,
    #[serde(default)]
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub cluster: Option<String>,
    pub picked_genres: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub cluster: Option<String>,
    pub picked_genres: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Scored {
    pub title_id: i32,
    pub score: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Profile {
    pub watched: Vec<i32>,
    pub genres: Vec<String>,
}
