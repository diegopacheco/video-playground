pub fn damerau_levenshtein(a: &[char], b: &[char]) -> usize {
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let width = m + 1;
    let mut d = vec![0usize; (n + 1) * width];
    for i in 0..=n {
        d[i * width] = i;
    }
    for (j, cell) in d.iter_mut().enumerate().take(m + 1) {
        *cell = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (d[(i - 1) * width + j] + 1)
                .min(d[i * width + j - 1] + 1)
                .min(d[(i - 1) * width + j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(d[(i - 2) * width + j - 2] + 1);
            }
            d[i * width + j] = best;
        }
    }
    d[n * width + m]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dl(a: &str, b: &str) -> usize {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        damerau_levenshtein(&a, &b)
    }

    #[test]
    fn a_swapped_pair_is_one_typo_not_two() {
        assert_eq!(dl("tiatn", "titan"), 1);
        assert_eq!(dl("naurto", "naruto"), 1);
    }

    #[test]
    fn missing_inserted_and_replaced_letters_cost_one_each() {
        assert_eq!(dl("atack", "attack"), 1);
        assert_eq!(dl("attackk", "attack"), 1);
        assert_eq!(dl("bleech", "bleach"), 1);
        assert_eq!(dl("", "abc"), 3);
        assert_eq!(dl("same", "same"), 0);
    }

    #[test]
    fn works_on_kanji_characters() {
        assert_eq!(dl("進撃", "神撃"), 1);
    }
}
