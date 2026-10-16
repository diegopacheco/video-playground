use crate::error::{Error, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRequest {
    pub msn: u64,
    pub part: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStatus {
    Ready,
    Pending,
    TooFar,
}

pub fn parse(query: &HashMap<String, String>) -> Result<Option<BlockRequest>> {
    let msn = query.get("_HLS_msn");
    let part = query.get("_HLS_part");
    match (msn, part) {
        (None, None) => Ok(None),
        (None, Some(_)) => Err(Error::BadRequest("_HLS_part requires _HLS_msn".into())),
        (Some(m), p) => Ok(Some(BlockRequest {
            msn: m.parse().map_err(|_| Error::BadRequest("invalid _HLS_msn".into()))?,
            part: p
                .map(|v| v.parse().map_err(|_| Error::BadRequest("invalid _HLS_part".into())))
                .transpose()?,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn plain_reload_does_not_block() {
        assert_eq!(parse(&query(&[])).unwrap(), None);
    }

    #[test]
    fn msn_and_part_become_a_blocking_request() {
        let req = parse(&query(&[("_HLS_msn", "12"), ("_HLS_part", "3")])).unwrap();
        assert_eq!(req, Some(BlockRequest { msn: 12, part: Some(3) }));
    }

    #[test]
    fn part_without_msn_is_rejected_as_the_spec_mandates_400() {
        assert!(matches!(parse(&query(&[("_HLS_part", "3")])), Err(Error::BadRequest(_))));
    }

    #[test]
    fn non_numeric_directives_are_rejected() {
        assert!(parse(&query(&[("_HLS_msn", "x")])).is_err());
        assert!(parse(&query(&[("_HLS_msn", "1"), ("_HLS_part", "-1")])).is_err());
    }
}
