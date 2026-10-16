use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::error::SearchError;
use crate::normalize::{Token, tokenize};

const DEFAULT_CATALOG: &str = include_str!("../../data/catalog.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anime {
    pub id: u32,
    pub romaji: String,
    pub english: String,
    pub japanese: String,
    pub year: u16,
    pub popularity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Field {
    Romaji,
    English,
    Japanese,
}

impl Field {
    pub const ALL: [Field; 3] = [Field::Romaji, Field::English, Field::Japanese];
}

#[derive(Debug, Clone)]
pub struct FieldText {
    pub field: Field,
    pub original: String,
    pub tokens: Vec<Token>,
    pub normalized_len: usize,
}

#[derive(Debug, Clone)]
pub struct PreparedDoc {
    pub anime: Anime,
    pub fields: Vec<FieldText>,
}

impl Anime {
    pub fn text(&self, field: Field) -> &str {
        match field {
            Field::Romaji => &self.romaji,
            Field::English => &self.english,
            Field::Japanese => &self.japanese,
        }
    }
}

impl PreparedDoc {
    pub fn new(anime: Anime) -> Self {
        let fields = Field::ALL
            .iter()
            .map(|&field| {
                let original = anime.text(field).to_string();
                let tokens = tokenize(&original);
                let normalized_len = tokens.iter().map(|t| t.chars.len()).sum();
                FieldText {
                    field,
                    original,
                    tokens,
                    normalized_len,
                }
            })
            .collect();
        Self { anime, fields }
    }
}

pub fn parse_catalog(json: &str) -> Result<Vec<Anime>, SearchError> {
    let items: Vec<Anime> = serde_json::from_str(json)?;
    let mut seen = HashSet::new();
    for item in &items {
        if !seen.insert(item.id) {
            return Err(SearchError::InvalidCatalog(format!("duplicate id {}", item.id)));
        }
        if Field::ALL.iter().any(|f| item.text(*f).trim().is_empty()) {
            return Err(SearchError::InvalidCatalog(format!("empty title for id {}", item.id)));
        }
        if item.popularity > 100 {
            return Err(SearchError::InvalidCatalog(format!("popularity above 100 for id {}", item.id)));
        }
    }
    Ok(items)
}

pub fn default_catalog() -> Result<Vec<Anime>, SearchError> {
    parse_catalog(DEFAULT_CATALOG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_catalog_is_large_enough_and_every_title_is_searchable_in_three_languages() {
        let items = default_catalog().expect("catalog parses");
        assert!(items.len() >= 200, "catalog has {}", items.len());
        for item in &items {
            let doc = PreparedDoc::new(item.clone());
            assert!(doc.fields.iter().all(|f| !f.tokens.is_empty()), "{}", item.romaji);
        }
    }

    #[test]
    fn duplicate_ids_are_rejected_because_results_are_keyed_by_id() {
        let json = r#"[
            {"id":1,"romaji":"a","english":"a","japanese":"あ","year":2000,"popularity":1},
            {"id":1,"romaji":"b","english":"b","japanese":"い","year":2000,"popularity":1}
        ]"#;
        assert!(matches!(parse_catalog(json), Err(SearchError::InvalidCatalog(_))));
    }

    #[test]
    fn empty_titles_are_rejected_because_they_can_never_be_found() {
        let json = r#"[{"id":1,"romaji":" ","english":"a","japanese":"あ","year":2000,"popularity":1}]"#;
        assert!(matches!(parse_catalog(json), Err(SearchError::InvalidCatalog(_))));
    }
}
