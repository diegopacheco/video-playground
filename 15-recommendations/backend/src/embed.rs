use std::collections::{HashMap, HashSet};

pub const DIM: usize = 256;

const GENRE_WEIGHT: f32 = 3.0;
const TAG_WEIGHT: f32 = 2.0;
const WORD_WEIGHT: f32 = 1.0;

const STOPWORDS: [&str; 24] = [
    "the", "and", "for", "with", "they", "their", "along", "way", "meet", "hoping", "fights", "into",
    "from", "that", "this", "who", "one", "out", "all", "its", "are", "was", "his", "her",
];

pub struct Doc<'a> {
    pub genres: &'a [String],
    pub tags: &'a [String],
    pub synopsis: &'a str,
}

pub struct Embedder {
    df: HashMap<String, usize>,
    docs: usize,
}

pub fn tokens(doc: &Doc) -> Vec<(String, f32)> {
    let mut out: Vec<(String, f32)> = Vec::new();
    for g in doc.genres {
        out.push((format!("g:{}", g.to_lowercase()), GENRE_WEIGHT));
    }
    for t in doc.tags {
        out.push((format!("t:{}", t.to_lowercase()), TAG_WEIGHT));
    }
    for w in doc.synopsis.split(|c: char| !c.is_alphanumeric()) {
        let w = w.to_lowercase();
        if w.len() >= 3 && !STOPWORDS.contains(&w.as_str()) {
            out.push((format!("w:{w}"), WORD_WEIGHT));
        }
    }
    out
}

fn fnv1a(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}

pub fn normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        v.iter_mut().for_each(|x| *x /= norm);
    }
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

pub fn to_pgvector(v: &[f32]) -> String {
    let parts: Vec<String> = v.iter().map(|x| format!("{x:.6}")).collect();
    format!("[{}]", parts.join(","))
}

impl Embedder {
    pub fn fit(docs: &[Doc]) -> Self {
        let mut df = HashMap::new();
        for doc in docs {
            let unique: HashSet<String> = tokens(doc).into_iter().map(|(t, _)| t).collect();
            for t in unique {
                *df.entry(t).or_insert(0) += 1;
            }
        }
        Self { df, docs: docs.len() }
    }

    fn idf(&self, token: &str) -> f32 {
        let df = self.df.get(token).copied().unwrap_or(0);
        ((self.docs as f32 + 1.0) / (df as f32 + 1.0)).ln() + 1.0
    }

    pub fn embed(&self, doc: &Doc) -> Vec<f32> {
        let mut tf: HashMap<String, f32> = HashMap::new();
        for (t, w) in tokens(doc) {
            *tf.entry(t).or_insert(0.0) += w;
        }
        let mut v = vec![0.0f32; DIM];
        for (t, w) in tf {
            let h = fnv1a(&t);
            let sign = if h >> 63 == 1 { -1.0 } else { 1.0 };
            v[(h % DIM as u64) as usize] += sign * w * self.idf(&t);
        }
        normalize(&mut v);
        v
    }

    pub fn embed_genres(&self, genres: &[String]) -> Vec<f32> {
        self.embed(&Doc { genres, tags: &[], synopsis: "" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn vectors_are_unit_length_so_dot_product_is_cosine() {
        let g = s(&["Action"]);
        let t = s(&["swordplay"]);
        let e = Embedder::fit(&[Doc { genres: &g, tags: &t, synopsis: "a swordsman" }]);
        let v = e.embed(&Doc { genres: &g, tags: &t, synopsis: "a lone swordsman seeks revenge" });
        assert_eq!(v.len(), DIM);
        assert!((cosine(&v, &v) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn titles_sharing_genres_and_tags_are_closer_than_unrelated_ones() {
        let (ga, ta) = (s(&["Action", "Supernatural"]), s(&["swordplay", "yokai"]));
        let (gb, tb) = (s(&["Action", "Supernatural"]), s(&["swordplay", "exorcist"]));
        let (gc, tc) = (s(&["Romance", "Slice of Life"]), s(&["cafe", "confession"]));
        let a = Doc { genres: &ga, tags: &ta, synopsis: "a fighter battles yokai" };
        let b = Doc { genres: &gb, tags: &tb, synopsis: "an exorcist battles evil spirits" };
        let c = Doc { genres: &gc, tags: &tc, synopsis: "a florist opens a quiet cafe" };
        let e = Embedder::fit(&[Doc { ..a }, Doc { ..b }, Doc { ..c }]);
        let (va, vb, vc) = (e.embed(&a), e.embed(&b), e.embed(&c));
        assert!(cosine(&va, &vb) > cosine(&va, &vc) + 0.3);
    }

    #[test]
    fn rare_tokens_weigh_more_than_common_ones() {
        let common = s(&["Drama"]);
        let rare = s(&["Mecha"]);
        let docs = [
            Doc { genres: &common, tags: &[], synopsis: "" },
            Doc { genres: &common, tags: &[], synopsis: "" },
            Doc { genres: &rare, tags: &[], synopsis: "" },
        ];
        let e = Embedder::fit(&docs);
        assert!(e.idf("g:mecha") > e.idf("g:drama"));
    }

    #[test]
    fn pgvector_literal_is_bracketed_and_comma_separated() {
        assert_eq!(to_pgvector(&[0.5, -1.0]), "[0.500000,-1.000000]");
    }
}
