use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::{compose, decompose_canonical};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Script {
    Latin,
    Cjk,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub chars: Vec<char>,
    pub positions: Vec<usize>,
    pub script: Script,
}

impl Token {
    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn span(&self, from: usize, len: usize) -> (usize, usize) {
        let last = (from + len).min(self.positions.len()).saturating_sub(1);
        (self.positions[from], self.positions[last] + 1)
    }
}

pub fn script_of(c: char) -> Script {
    match c as u32 {
        0x3005 | 0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF => {
            Script::Cjk
        }
        _ => Script::Latin,
    }
}

fn is_combining_diacritic(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F)
}

fn strip_diacritics(c: char, out: &mut Vec<char>) {
    let mut parts = Vec::with_capacity(2);
    decompose_canonical(c, |d| parts.push(d));
    if parts.iter().any(|d| is_combining_diacritic(*d)) {
        out.extend(parts.into_iter().filter(|d| !is_combining_diacritic(*d)));
    } else {
        out.push(c);
    }
}

fn fold_katakana(c: char) -> char {
    match c as u32 {
        0x30A1..=0x30F6 => char::from_u32(c as u32 - 0x60).unwrap_or(c),
        _ => c,
    }
}

fn is_long_vowel_tail(prev: char, c: char) -> bool {
    matches!((prev, c), ('o', 'u') | ('o', 'o') | ('u', 'u'))
}

fn push_folded(out: &mut Vec<(char, usize)>, c: char, pos: usize) {
    if matches!(c as u32, 0x3099 | 0x309A) {
        if let Some(last) = out.last_mut()
            && let Some(joined) = compose(last.0, c)
        {
            last.0 = joined;
        }
        return;
    }
    let c = if c.is_alphanumeric() { c } else { ' ' };
    if c == ' ' && out.last().is_none_or(|l| l.0 == ' ') {
        return;
    }
    if out.last().is_some_and(|last| is_long_vowel_tail(last.0, c)) {
        return;
    }
    out.push((c, pos));
}

pub fn normalize_mapped(text: &str) -> Vec<(char, usize)> {
    let mut out: Vec<(char, usize)> = Vec::with_capacity(text.len());
    let mut stripped = Vec::with_capacity(4);
    for (pos, original) in text.chars().enumerate() {
        for c in std::iter::once(original).nfkc() {
            for lower in c.to_lowercase() {
                stripped.clear();
                strip_diacritics(lower, &mut stripped);
                for s in stripped.iter() {
                    push_folded(&mut out, fold_katakana(*s), pos);
                }
            }
        }
    }
    while out.last().is_some_and(|l| l.0 == ' ') {
        out.pop();
    }
    out
}

pub fn normalize(text: &str) -> String {
    normalize_mapped(text).into_iter().map(|(c, _)| c).collect()
}

pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut current: Option<Token> = None;
    for (c, pos) in normalize_mapped(text) {
        if c == ' ' {
            tokens.extend(current.take());
            continue;
        }
        let script = script_of(c);
        match current.as_mut() {
            Some(t) if t.script == script => {
                t.chars.push(c);
                t.positions.push(pos);
            }
            _ => {
                tokens.extend(current.take());
                current = Some(Token {
                    chars: vec![c],
                    positions: vec![pos],
                    script,
                });
            }
        }
    }
    tokens.extend(current);
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullwidth_and_case_fold_to_the_same_form_users_type() {
        assert_eq!(normalize("ＮＡＲＵＴＯ"), "naruto");
    }

    #[test]
    fn diacritics_are_removed_so_pokemon_matches_without_accent() {
        assert_eq!(normalize("Pokémon"), "pokemon");
        assert_eq!(normalize("Nausicaä"), "nausicaa");
    }

    #[test]
    fn long_vowel_romaji_variants_collapse_to_one_spelling() {
        let canonical = normalize("Kyoshitsu");
        assert_eq!(normalize("Kyōshitsu"), canonical);
        assert_eq!(normalize("Kyoushitsu"), canonical);
        assert_eq!(normalize("Kyooshitsu"), canonical);
        assert_eq!(normalize("Shuumatsu"), normalize("Shumatsu"));
    }

    #[test]
    fn katakana_folds_into_hiragana_so_either_kana_matches() {
        assert_eq!(normalize("ナルト"), normalize("なると"));
        assert_eq!(normalize("ﾅﾙﾄ"), normalize("なると"));
        assert_eq!(normalize("ﾊﾟﾌﾟﾘｶ"), normalize("ぱぷりか"));
    }

    #[test]
    fn dakuten_kana_keep_their_voicing() {
        assert_eq!(normalize("ガンダム"), "がんだむ");
    }

    #[test]
    fn punctuation_becomes_word_boundaries() {
        assert_eq!(normalize("Re:ZERO -Starting Life-"), "re zero starting life");
        assert_eq!(normalize("ソードアート・オンライン"), "そーどあーと おんらいん");
    }

    #[test]
    fn tokens_split_on_script_change_and_keep_original_positions() {
        let tokens = tokenize("Re:ゼロから");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text(), "re");
        assert_eq!(tokens[0].script, Script::Latin);
        assert_eq!(tokens[1].text(), "ぜろから");
        assert_eq!(tokens[1].script, Script::Cjk);
        assert_eq!(tokens[1].span(0, 2), (3, 5));
    }
}
