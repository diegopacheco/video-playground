use std::collections::HashMap;

use serde::Serialize;

use crate::model::Playhead;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Store,
    Buffer,
}

#[derive(Debug, Clone, Serialize)]
pub struct RowItem {
    #[serde(flatten)]
    pub playhead: Playhead,
    pub progress: f64,
    pub source: Source,
}

pub fn continue_watching(stored: Vec<Playhead>, pending: Vec<Playhead>) -> Vec<RowItem> {
    let mut latest: HashMap<String, (Playhead, Source)> = HashMap::new();
    let tagged = stored
        .into_iter()
        .map(|p| (p, Source::Store))
        .chain(pending.into_iter().map(|p| (p, Source::Buffer)));
    for (p, source) in tagged {
        let replace = latest.get(&p.title_id).is_none_or(|(cur, _)| p.wins_over(cur));
        if replace {
            latest.insert(p.title_id.clone(), (p, source));
        }
    }
    let mut row: Vec<RowItem> = latest
        .into_values()
        .filter(|(p, _)| !p.is_finished())
        .map(|(p, source)| RowItem { progress: p.progress(), playhead: p, source })
        .collect();
    row.sort_by(|a, b| {
        b.playhead
            .client_ts_ms
            .cmp(&a.playhead.client_ts_ms)
            .then_with(|| a.playhead.title_id.cmp(&b.playhead.title_id))
    });
    row
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::playhead;

    fn titles(row: &[RowItem]) -> Vec<&str> {
        row.iter().map(|r| r.playhead.title_id.as_str()).collect()
    }

    #[test]
    fn most_recently_watched_title_comes_first() {
        let stored = vec![playhead("p", "old", "tv", 10, 1_000), playhead("p", "new", "tv", 10, 9_000), playhead("p", "mid", "tv", 10, 5_000)];
        assert_eq!(titles(&continue_watching(stored, vec![])), vec!["new", "mid", "old"]);
    }

    #[test]
    fn finished_titles_leave_the_row() {
        let stored = vec![playhead("p", "done", "tv", 950, 9_000), playhead("p", "watching", "tv", 400, 1_000)];
        assert_eq!(titles(&continue_watching(stored, vec![])), vec!["watching"]);
    }

    #[test]
    fn unflushed_heartbeat_is_visible_immediately_for_read_your_writes() {
        let stored = vec![playhead("p", "t", "tv", 100, 1_000)];
        let pending = vec![playhead("p", "t", "phone", 300, 2_000)];
        let row = continue_watching(stored, pending);
        assert_eq!(row[0].playhead.position_secs, 300);
        assert_eq!(row[0].source, Source::Buffer);
    }

    #[test]
    fn stale_buffered_heartbeat_does_not_hide_newer_stored_position() {
        let stored = vec![playhead("p", "t", "phone", 700, 5_000)];
        let pending = vec![playhead("p", "t", "tv", 100, 1_000)];
        let row = continue_watching(stored, pending);
        assert_eq!(row[0].playhead.position_secs, 700);
        assert_eq!(row[0].source, Source::Store);
    }

    #[test]
    fn finishing_on_another_device_removes_title_even_if_store_has_it_in_progress() {
        let stored = vec![playhead("p", "t", "tv", 300, 1_000)];
        let pending = vec![playhead("p", "t", "phone", 990, 2_000)];
        assert!(continue_watching(stored, pending).is_empty());
    }
}
