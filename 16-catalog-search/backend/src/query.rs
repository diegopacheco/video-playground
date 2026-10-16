use crate::error::SearchError;
use crate::ngram::query_grams;
use crate::normalize::{Token, tokenize};

#[derive(Debug, Clone)]
pub struct Query {
    pub raw: String,
    pub tokens: Vec<Token>,
    pub grams: Vec<String>,
    pub open_ended: bool,
}

impl Query {
    pub fn parse(raw: &str) -> Result<Self, SearchError> {
        let tokens = tokenize(raw);
        if tokens.is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        let open_ended = !raw.ends_with(char::is_whitespace);
        let last = tokens.len() - 1;
        let mut grams: Vec<String> = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            for gram in query_grams(token, open_ended && i == last) {
                if !grams.contains(&gram) {
                    grams.push(gram);
                }
            }
        }
        Ok(Self {
            raw: raw.to_string(),
            tokens,
            grams,
            open_ended,
        })
    }

    pub fn is_open(&self, token_index: usize) -> bool {
        self.open_ended && token_index + 1 == self.tokens.len()
    }

    pub fn normalized(&self) -> String {
        self.tokens.iter().map(Token::text).collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_or_punctuation_only_input_is_rejected() {
        assert!(matches!(Query::parse("   "), Err(SearchError::EmptyQuery)));
        assert!(matches!(Query::parse("!!・・"), Err(SearchError::EmptyQuery)));
    }

    #[test]
    fn only_the_last_word_is_treated_as_a_prefix_while_typing() {
        let q = Query::parse("attack on ti").expect("query");
        assert!(!q.is_open(0));
        assert!(q.is_open(2));
        let finished = Query::parse("attack on titan ").expect("query");
        assert!(!finished.is_open(2));
    }
}
