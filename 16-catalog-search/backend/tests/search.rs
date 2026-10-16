use std::sync::OnceLock;

use catalog_search::catalog::{Field, default_catalog};
use catalog_search::engine::DefaultEngine;
use catalog_search::error::SearchError;
use catalog_search::workload::measure;

const ATTACK_ON_TITAN: u32 = 1;
const DEMON_SLAYER: u32 = 2;
const NARUTO: u32 = 3;
const ONE_PIECE: u32 = 5;
const FULLMETAL: u32 = 7;
const SPIRITED_AWAY: u32 = 18;
const ASSASSINATION_CLASSROOM: u32 = 31;
const POKEMON: u32 = 110;
const RAGE_OF_BAHAMUT: u32 = 183;

fn engine() -> &'static DefaultEngine {
    static ENGINE: OnceLock<DefaultEngine> = OnceLock::new();
    ENGINE.get_or_init(|| DefaultEngine::with_defaults(default_catalog().expect("catalog")))
}

fn top(q: &str) -> u32 {
    let response = engine().search(q, 10).expect("search");
    assert!(!response.results.is_empty(), "no results for {q:?}");
    response.results[0].id
}

fn ids(q: &str) -> Vec<u32> {
    engine()
        .search(q, 10)
        .expect("search")
        .results
        .iter()
        .map(|h| h.id)
        .collect()
}

#[test]
fn romaji_fragment_shingeki_finds_attack_on_titan_first() {
    assert_eq!(top("shingeki"), ATTACK_ON_TITAN);
}

#[test]
fn a_similar_less_popular_title_stays_below_the_famous_one() {
    let found = ids("shingeki");
    assert!(found.contains(&RAGE_OF_BAHAMUT));
    assert!(found.iter().position(|&i| i == ATTACK_ON_TITAN) < found.iter().position(|&i| i == RAGE_OF_BAHAMUT));
}

#[test]
fn kanji_fragment_finds_attack_on_titan_through_cjk_bigrams() {
    assert_eq!(top("進撃"), ATTACK_ON_TITAN);
}

#[test]
fn one_kanji_off_is_not_a_match_because_two_kanji_queries_get_no_typo_budget() {
    assert!(!ids("進撃").contains(&RAGE_OF_BAHAMUT));
}

#[test]
fn english_title_with_a_typo_still_finds_attack_on_titan() {
    let response = engine().search("atack on titan", 10).expect("search");
    let first = &response.results[0];
    assert_eq!(first.id, ATTACK_ON_TITAN);
    assert_eq!(first.breakdown.field, Field::English);
    assert_eq!(first.breakdown.edit_distance, 1);
}

#[test]
fn transposed_letters_are_one_typo() {
    assert_eq!(top("attack on tiatn"), ATTACK_ON_TITAN);
}

#[test]
fn demon_slayer_is_found_by_romaji_and_by_kanji() {
    assert_eq!(top("Kimetsu no Yaiba"), DEMON_SLAYER);
    assert_eq!(top("鬼滅"), DEMON_SLAYER);
    assert_eq!(top("鬼滅の刃"), DEMON_SLAYER);
}

#[test]
fn autocomplete_finds_titles_from_the_first_letters_typed() {
    assert_eq!(top("kimet"), DEMON_SLAYER);
    assert_eq!(top("shing"), ATTACK_ON_TITAN);
    assert_eq!(top("attack on ti"), ATTACK_ON_TITAN);
}

#[test]
fn hiragana_query_finds_a_katakana_title() {
    assert_eq!(top("なると"), NARUTO);
    assert_eq!(top("わんぴーす"), ONE_PIECE);
}

#[test]
fn fullwidth_input_from_japanese_keyboards_matches() {
    assert_eq!(top("ＮＡＲＵＴＯ"), NARUTO);
}

#[test]
fn exact_short_title_ranks_above_its_longer_sequel() {
    assert_eq!(top("naruto"), NARUTO);
}

#[test]
fn accent_free_query_matches_accented_title() {
    assert_eq!(top("pokemon"), POKEMON);
}

#[test]
fn every_long_vowel_spelling_finds_the_same_title() {
    for q in ["ansatsu kyoushitsu", "ansatsu kyoshitsu", "ansatsu kyōshitsu", "ansatsu kyooshitsu"] {
        assert_eq!(top(q), ASSASSINATION_CLASSROOM, "{q}");
    }
}

#[test]
fn typo_in_a_long_romaji_word_is_tolerated() {
    assert_eq!(top("fullmetal alchemst"), FULLMETAL);
    assert_eq!(top("hagane no renkinjutsush"), FULLMETAL);
}

#[test]
fn mixed_kanji_and_kana_title_fragment_matches() {
    assert_eq!(top("千と千尋"), SPIRITED_AWAY);
}

#[test]
fn gibberish_returns_nothing_instead_of_random_titles() {
    assert!(ids("zzzzqqq").is_empty());
    assert!(ids("xqzvbn wplm").is_empty());
}

#[test]
fn empty_query_is_an_error_not_the_whole_catalog() {
    assert!(matches!(engine().search("  ", 10), Err(SearchError::EmptyQuery)));
}

#[test]
fn results_are_sorted_by_score_and_respect_the_limit() {
    let response = engine().search("the", 5).expect("search");
    assert!(response.results.len() <= 5);
    assert!(response.results.windows(2).all(|w| w[0].score >= w[1].score));
}

#[test]
fn highlights_rebuild_the_original_title_exactly() {
    let response = engine().search("鬼滅", 1).expect("search");
    for h in &response.results[0].highlights {
        let joined: String = h.segments.iter().map(|s| s.text.as_str()).collect();
        let original = match h.field {
            Field::Romaji => &response.results[0].romaji,
            Field::English => &response.results[0].english,
            Field::Japanese => &response.results[0].japanese,
        };
        assert_eq!(&joined, original);
    }
}

#[test]
fn p99_latency_stays_within_the_search_as_you_type_budget() {
    let budget_us = if cfg!(debug_assertions) { 25_000 } else { 2_000 };
    let latency = measure(engine(), 20).expect("measure");
    assert!(
        latency.p99_us < budget_us,
        "p99 {}us exceeds {}us",
        latency.p99_us,
        budget_us
    );
}
