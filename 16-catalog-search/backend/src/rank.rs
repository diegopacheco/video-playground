use serde::Serialize;

use crate::catalog::{Field, FieldText, PreparedDoc};
use crate::distance::damerau_levenshtein;
use crate::normalize::{Script, Token};
use crate::query::Query;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub hit: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldHighlight {
    pub field: Field,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Breakdown {
    pub field: Field,
    pub similarity: f32,
    pub coverage: f32,
    pub edit_distance: usize,
    pub overlap: f32,
    pub boost: f32,
    pub popularity: f32,
    pub similarity_score: f32,
    pub coverage_score: f32,
    pub text_score: f32,
    pub popularity_score: f32,
    pub total: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Scored {
    pub total: f32,
    pub breakdown: Breakdown,
    pub highlights: Vec<FieldHighlight>,
}

pub trait Ranker: Send + Sync {
    fn score(&self, query: &Query, doc: &PreparedDoc, overlap: f32) -> Option<Scored>;
}

#[derive(Debug, Clone, Copy)]
pub struct FieldBoosts {
    pub romaji: f32,
    pub english: f32,
    pub japanese: f32,
}

impl FieldBoosts {
    pub fn of(&self, field: Field) -> f32 {
        match field {
            Field::Romaji => self.romaji,
            Field::English => self.english,
            Field::Japanese => self.japanese,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WeightedRanker {
    pub boosts: FieldBoosts,
    pub similarity_weight: f32,
    pub coverage_weight: f32,
    pub popularity_weight: f32,
    pub min_similarity: f32,
}

impl Default for WeightedRanker {
    fn default() -> Self {
        Self {
            boosts: FieldBoosts {
                romaji: 1.0,
                english: 0.95,
                japanese: 1.0,
            },
            similarity_weight: 0.75,
            coverage_weight: 0.13,
            popularity_weight: 0.12,
            min_similarity: 0.6,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TokenHit {
    score: f32,
    distance: usize,
    covered: usize,
    span: (usize, usize),
}

#[derive(Debug, Clone)]
struct FieldMatch {
    field: Field,
    similarity: f32,
    coverage: f32,
    distance: usize,
    spans: Vec<(usize, usize)>,
}

fn allowed_typos(len: usize) -> usize {
    match len {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    }
}

fn better(a: Option<TokenHit>, b: Option<TokenHit>) -> Option<TokenHit> {
    match (a, b) {
        (Some(x), Some(y)) => Some(if y.score > x.score { y } else { x }),
        (x, None) => x,
        (None, y) => y,
    }
}

fn latin_hit(q: &Token, f: &Token, open: bool) -> Option<TokenHit> {
    let (ql, fl) = (q.chars.len(), f.chars.len());
    if q.chars == f.chars {
        return Some(TokenHit { score: 1.0, distance: 0, covered: fl, span: f.span(0, fl) });
    }
    let prefix_weight = 0.75 + 0.25 * ql as f32 / fl.max(1) as f32;
    let mut best = None;
    if open && fl > ql && f.chars.starts_with(&q.chars) {
        best = Some(TokenHit { score: prefix_weight, distance: 0, covered: ql, span: f.span(0, ql) });
    }
    let full = damerau_levenshtein(&q.chars, &f.chars);
    if full <= allowed_typos(ql) {
        let score = 1.0 - full as f32 / ql.max(fl) as f32;
        best = better(best, Some(TokenHit { score, distance: full, covered: fl, span: f.span(0, fl) }));
    }
    if open && fl > ql {
        let partial = damerau_levenshtein(&q.chars, &f.chars[..ql]);
        if partial > 0 && partial <= allowed_typos(ql) {
            let score = (1.0 - partial as f32 / ql as f32) * prefix_weight;
            best = better(best, Some(TokenHit { score, distance: partial, covered: ql, span: f.span(0, ql) }));
        }
    }
    best
}

fn cjk_hit(q: &Token, f: &Token) -> Option<TokenHit> {
    let (ql, fl) = (q.chars.len(), f.chars.len());
    if ql <= fl
        && let Some(at) = f.chars.windows(ql).position(|w| w == q.chars.as_slice())
    {
        return Some(TokenHit { score: 1.0, distance: 0, covered: ql, span: f.span(at, ql) });
    }
    let allowed = ql / 3;
    if allowed == 0 {
        return None;
    }
    let (at, distance) = if ql <= fl {
        f.chars
            .windows(ql)
            .enumerate()
            .map(|(i, w)| (i, damerau_levenshtein(&q.chars, w)))
            .min_by_key(|(_, d)| *d)?
    } else {
        (0, damerau_levenshtein(&q.chars, &f.chars))
    };
    if distance > allowed {
        return None;
    }
    let len = ql.min(fl);
    Some(TokenHit {
        score: 1.0 - distance as f32 / ql as f32,
        distance,
        covered: len,
        span: f.span(at, len),
    })
}

fn token_hit(q: &Token, f: &Token, open: bool) -> Option<TokenHit> {
    match (q.script, f.script) {
        (Script::Latin, Script::Latin) => latin_hit(q, f, open),
        (Script::Cjk, Script::Cjk) => cjk_hit(q, f),
        _ => None,
    }
}

fn match_field(query: &Query, field: &FieldText) -> Option<FieldMatch> {
    let mut sum = 0.0;
    let mut distance = 0;
    let mut covered = 0;
    let mut spans = Vec::new();
    for (i, q) in query.tokens.iter().enumerate() {
        let open = query.is_open(i);
        let hit = field
            .tokens
            .iter()
            .fold(None, |acc, f| better(acc, token_hit(q, f, open)));
        if let Some(h) = hit {
            sum += h.score;
            distance += h.distance;
            covered += h.covered;
            spans.push(h.span);
        }
    }
    if spans.is_empty() {
        return None;
    }
    Some(FieldMatch {
        field: field.field,
        similarity: sum / query.tokens.len() as f32,
        coverage: (covered as f32 / field.normalized_len.max(1) as f32).min(1.0),
        distance,
        spans,
    })
}

pub fn segments(text: &str, spans: &[(usize, usize)]) -> Vec<Segment> {
    let chars: Vec<char> = text.chars().collect();
    let mut marks = vec![false; chars.len()];
    for &(start, end) in spans {
        for m in marks.iter_mut().take(end.min(chars.len())).skip(start) {
            *m = true;
        }
    }
    let mut out: Vec<Segment> = Vec::new();
    for (c, hit) in chars.into_iter().zip(marks) {
        match out.last_mut() {
            Some(s) if s.hit == hit => s.text.push(c),
            _ => out.push(Segment { text: c.to_string(), hit }),
        }
    }
    out
}

impl WeightedRanker {
    fn parts(&self, m: &FieldMatch) -> (f32, f32) {
        let boost = self.boosts.of(m.field);
        (
            boost * self.similarity_weight * m.similarity,
            boost * self.coverage_weight * m.coverage,
        )
    }

    fn text_score(&self, m: &FieldMatch) -> f32 {
        let (similarity, coverage) = self.parts(m);
        similarity + coverage
    }
}

impl Ranker for WeightedRanker {
    fn score(&self, query: &Query, doc: &PreparedDoc, overlap: f32) -> Option<Scored> {
        let matches: Vec<(&FieldText, Option<FieldMatch>)> =
            doc.fields.iter().map(|f| (f, match_field(query, f))).collect();
        let best = matches
            .iter()
            .filter_map(|(_, m)| m.as_ref())
            .filter(|m| m.similarity >= self.min_similarity)
            .max_by(|a, b| self.text_score(a).total_cmp(&self.text_score(b)))?;
        let popularity = doc.anime.popularity as f32 / 100.0;
        let (similarity_score, coverage_score) = self.parts(best);
        let text_score = similarity_score + coverage_score;
        let popularity_score = self.popularity_weight * popularity;
        let total = text_score + popularity_score;
        let highlights = matches
            .iter()
            .map(|(f, m)| FieldHighlight {
                field: f.field,
                segments: segments(&f.original, m.as_ref().map_or(&[][..], |m| &m.spans)),
            })
            .collect();
        Some(Scored {
            total,
            breakdown: Breakdown {
                field: best.field,
                similarity: best.similarity,
                coverage: best.coverage,
                edit_distance: best.distance,
                overlap,
                boost: self.boosts.of(best.field),
                popularity,
                similarity_score,
                coverage_score,
                text_score,
                popularity_score,
                total,
            },
            highlights,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Anime;

    fn doc(romaji: &str, english: &str, japanese: &str, popularity: u8) -> PreparedDoc {
        PreparedDoc::new(Anime {
            id: 1,
            romaji: romaji.into(),
            english: english.into(),
            japanese: japanese.into(),
            year: 2000,
            popularity,
        })
    }

    fn score(q: &str, d: &PreparedDoc) -> Option<Scored> {
        WeightedRanker::default().score(&Query::parse(q).expect("query"), d, 1.0)
    }

    #[test]
    fn exact_title_beats_a_longer_title_that_only_starts_with_it() {
        let short = doc("Naruto", "Naruto", "ナルト", 90);
        let long = doc("Naruto: Shippuuden", "Naruto Shippuden", "ナルト疾風伝", 90);
        assert!(score("naruto", &short).unwrap().total > score("naruto", &long).unwrap().total);
    }

    #[test]
    fn popularity_breaks_ties_between_equally_good_text_matches() {
        let popular = doc("Alpha Beta", "Alpha Beta", "あるふぁ", 99);
        let obscure = doc("Alpha Gamma", "Alpha Gamma", "あるふぁ", 10);
        assert!(score("alpha", &popular).unwrap().total > score("alpha", &obscure).unwrap().total);
    }

    #[test]
    fn breakdown_parts_add_up_to_the_total_shown_in_the_ui() {
        let d = doc("Shingeki no Kyojin", "Attack on Titan", "進撃の巨人", 99);
        let s = score("atack on titan", &d).unwrap();
        let b = &s.breakdown;
        assert_eq!(b.field, Field::English);
        assert_eq!(b.edit_distance, 1);
        assert!((b.text_score + b.popularity_score - s.total).abs() < 1e-6);
        assert!((b.similarity_score + b.coverage_score - b.text_score).abs() < 1e-6);
        let expected = b.boost * (0.75 * b.similarity + 0.13 * b.coverage);
        assert!((expected - b.text_score).abs() < 1e-6);
    }

    #[test]
    fn short_words_get_no_typo_allowance_to_avoid_noise() {
        let d = doc("Bleach", "Bleach", "ブリーチ", 90);
        assert!(score("blx", &d).is_none());
    }

    #[test]
    fn too_many_typos_drop_the_document() {
        let d = doc("Bleach", "Bleach", "ブリーチ", 90);
        assert!(score("bzzzch ", &d).is_none());
    }

    #[test]
    fn highlights_point_at_the_original_characters_including_kanji() {
        let d = doc("Shingeki no Kyojin", "Attack on Titan", "進撃の巨人", 99);
        let s = score("進撃", &d).unwrap();
        let jp = s.highlights.iter().find(|h| h.field == Field::Japanese).unwrap();
        assert_eq!(jp.segments[0], Segment { text: "進撃".into(), hit: true });
        let joined: String = jp.segments.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "進撃の巨人");
    }

    #[test]
    fn highlighting_a_prefix_marks_only_the_typed_part() {
        let d = doc("Kimetsu no Yaiba", "Demon Slayer", "鬼滅の刃", 98);
        let s = score("kimet", &d).unwrap();
        let ro = s.highlights.iter().find(|h| h.field == Field::Romaji).unwrap();
        assert_eq!(ro.segments[0], Segment { text: "Kimet".into(), hit: true });
    }

    #[test]
    fn segments_cover_the_whole_text_once() {
        let segs = segments("Attack on Titan", &[(0, 6), (10, 15)]);
        let joined: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "Attack on Titan");
        assert_eq!(segs.iter().filter(|s| s.hit).count(), 2);
    }
}
