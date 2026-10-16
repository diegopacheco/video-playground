use std::sync::Arc;

use serde::Serialize;

use crate::ads::{AdDecisionService, BreakInfo, Creative, EPSILON};
use crate::error::{Result, SsaiError};
use crate::playlist::{Entry, Playlist};
use crate::users::UserProfile;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StitchedBreak {
    pub id: String,
    pub planned: f64,
    pub replaced: f64,
    pub filled: f64,
    pub creatives: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stitched {
    pub playlist: Playlist,
    pub breaks: Vec<StitchedBreak>,
}

pub struct UriScheme {
    pub content_base: String,
    pub media_base: String,
    pub tracking_base: String,
}

pub struct ManifestStitcher {
    decision: Arc<dyn AdDecisionService>,
    uris: UriScheme,
}

struct OpenBreak {
    index: usize,
    planned: f64,
    replacing: bool,
}

impl ManifestStitcher {
    pub fn new(decision: Arc<dyn AdDecisionService>, uris: UriScheme) -> ManifestStitcher {
        ManifestStitcher { decision, uris }
    }

    pub fn stitch(&self, content: &Playlist, user: &UserProfile, session: &str) -> Result<Stitched> {
        let mut entries = Vec::new();
        let mut breaks: Vec<StitchedBreak> = Vec::new();
        let mut open: Option<OpenBreak> = None;
        for entry in &content.entries {
            match entry {
                Entry::CueOut { id, duration, .. } => {
                    if open.is_some() {
                        return Err(SsaiError::InvalidPlaylist(format!("cue out {id} inside another break")));
                    }
                    entries.push(entry.clone());
                    let pod = self.decision.decide(user, &BreakInfo { id: id.clone(), duration: *duration })?;
                    let replacing = !pod.is_empty();
                    if replacing {
                        entries.extend(self.ad_entries(&pod, user, session, id));
                        entries.push(Entry::Discontinuity);
                    }
                    breaks.push(StitchedBreak {
                        id: id.clone(),
                        planned: *duration,
                        replaced: 0.0,
                        filled: pod.iter().map(Creative::duration).sum(),
                        creatives: pod.iter().map(|c| c.id.clone()).collect(),
                    });
                    open = Some(OpenBreak { index: breaks.len() - 1, planned: *duration, replacing });
                }
                Entry::CueIn { .. } => {
                    open = None;
                    entries.push(entry.clone());
                }
                Entry::Segment { duration, uri } => match open.as_mut() {
                    Some(current) => {
                        let summary = &mut breaks[current.index];
                        summary.replaced += duration;
                        if !current.replacing {
                            entries.push(self.content_segment(*duration, uri));
                        }
                        if summary.replaced + EPSILON >= current.planned {
                            open = None;
                        }
                    }
                    None => entries.push(self.content_segment(*duration, uri)),
                },
                Entry::Discontinuity | Entry::Tag(_) => entries.push(entry.clone()),
            }
        }
        let playlist = Playlist { header: content.header.clone(), entries, ended: content.ended };
        Ok(Stitched { playlist, breaks })
    }

    fn content_segment(&self, duration: f64, uri: &str) -> Entry {
        let absolute = uri.starts_with('/') || uri.contains("://");
        let uri = if absolute { uri.to_string() } else { format!("{}/{uri}", self.uris.content_base) };
        Entry::Segment { duration, uri }
    }

    fn ad_entries(&self, pod: &[Creative], user: &UserProfile, session: &str, break_id: &str) -> Vec<Entry> {
        let mut out = Vec::new();
        for creative in pod {
            out.push(Entry::Discontinuity);
            for (index, segment) in creative.segments.iter().enumerate() {
                let uri = if creative.tracked {
                    format!("{}/{}/{session}/{break_id}/{}/{index}.ts", self.uris.tracking_base, user.id, creative.id)
                } else {
                    format!("{}/{}/{}", self.uris.media_base, creative.id, segment.file)
                };
                out.push(Entry::Segment { duration: segment.duration, uri });
            }
        }
        out
    }
}
