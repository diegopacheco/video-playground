use crate::normalize::{Script, Token};

const START: char = '^';
const END: char = '$';

fn windows(chars: &[char], size: usize) -> Vec<String> {
    if chars.len() <= size {
        return vec![chars.iter().collect()];
    }
    chars.windows(size).map(|w| w.iter().collect()).collect()
}

fn latin_grams(token: &Token, open_ended: bool) -> Vec<String> {
    let mut padded = Vec::with_capacity(token.chars.len() + 2);
    padded.push(START);
    padded.extend(token.chars.iter().copied());
    if !open_ended {
        padded.push(END);
    }
    if padded.len() < 3 {
        return Vec::new();
    }
    windows(&padded, 3)
}

fn cjk_grams(token: &Token) -> Vec<String> {
    let mut grams: Vec<String> = token.chars.iter().map(|c| c.to_string()).collect();
    if token.chars.len() > 1 {
        grams.extend(windows(&token.chars, 2));
    }
    grams
}

pub fn index_grams(token: &Token) -> Vec<String> {
    match token.script {
        Script::Latin => latin_grams(token, false),
        Script::Cjk => cjk_grams(token),
    }
}

pub fn query_grams(token: &Token, open_ended: bool) -> Vec<String> {
    match token.script {
        Script::Latin => latin_grams(token, open_ended),
        Script::Cjk => cjk_grams(token),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::tokenize;

    #[test]
    fn latin_words_produce_padded_trigrams_so_word_edges_matter() {
        let t = &tokenize("naruto")[0];
        assert_eq!(
            index_grams(t),
            vec!["^na", "nar", "aru", "rut", "uto", "to$"]
        );
    }

    #[test]
    fn open_ended_query_token_has_no_end_marker_so_prefixes_match() {
        let t = &tokenize("kimet")[0];
        let grams = query_grams(t, true);
        assert!(!grams.iter().any(|g| g.ends_with(END)));
        let indexed = index_grams(&tokenize("kimetsu")[0]);
        assert!(grams.iter().all(|g| indexed.contains(g)));
    }

    #[test]
    fn cjk_uses_unigrams_plus_bigrams_so_single_kanji_and_longer_typos_still_hit() {
        let t = &tokenize("進撃の巨人")[0];
        let grams = index_grams(t);
        assert!(grams.contains(&"進撃".to_string()));
        assert!(!grams.contains(&"鬼".to_string()));
        assert!(grams.contains(&"巨".to_string()));
        assert_eq!(query_grams(&tokenize("進撃")[0], true), vec!["進", "撃", "進撃"]);
        assert_eq!(query_grams(&tokenize("巨")[0], true), vec!["巨"]);
    }
}
