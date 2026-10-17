use crate::ladder::Ladder;
use crate::manifest::RenditionReport;
use crate::packager::INIT_SEGMENT;

const TIMESCALE: u32 = 1000;

pub fn mpd(ladder: &Ladder, renditions: &[RenditionReport], duration: f64) -> String {
    let max_width = renditions.iter().map(|r| r.width).max().unwrap_or(0);
    let max_height = renditions.iter().map(|r| r.height).max().unwrap_or(0);
    let mut reps = String::new();
    for r in renditions {
        reps.push_str(&format!(
            "      <Representation id=\"{}\" bandwidth=\"{}\" width=\"{}\" height=\"{}\" codecs=\"{}\" frameRate=\"{}\"/>\n",
            r.id,
            r.peak_bps(),
            r.width,
            r.height,
            r.codecs,
            ladder.fps
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011" profiles="urn:mpeg:dash:profile:isoff-live:2011,urn:mpeg:dash:profile:cmaf:2019" type="static" mediaPresentationDuration="PT{duration:.3}S" minBufferTime="PT{min_buffer}S">
  <Period id="0" start="PT0S">
    <AdaptationSet id="0" contentType="video" mimeType="video/mp4" segmentAlignment="true" startWithSAP="1" maxWidth="{max_width}" maxHeight="{max_height}" maxFrameRate="{fps}">
      <SegmentTemplate timescale="{TIMESCALE}" duration="{segment}" startNumber="1" initialization="$RepresentationID$/{INIT_SEGMENT}" media="$RepresentationID$/seg_$Number%05d$.m4s"/>
{reps}    </AdaptationSet>
  </Period>
</MPD>
"#,
        min_buffer = ladder.segment_seconds * 2,
        fps = ladder.fps,
        segment = ladder.segment_seconds * TIMESCALE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::SegmentFile;

    fn reports() -> Vec<RenditionReport> {
        Ladder::standard()
            .renditions
            .iter()
            .map(|r| {
                let files = vec![SegmentFile { uri: "seg_00001.m4s".into(), duration: 2.0, bytes: r.bitrate_kbps as u64 * 250 }];
                RenditionReport::build(r, 800, files, vec![0.0])
            })
            .collect()
    }

    #[test]
    fn one_adaptation_set_declares_aligned_segments_starting_with_sap_so_dash_can_switch_seamlessly() {
        let xml = mpd(&Ladder::standard(), &reports(), 30.0);
        assert_eq!(xml.matches("<AdaptationSet").count(), 1);
        assert!(xml.contains("segmentAlignment=\"true\""));
        assert!(xml.contains("startWithSAP=\"1\""));
        assert!(xml.contains("mediaPresentationDuration=\"PT30.000S\""));
    }

    #[test]
    fn template_points_at_the_same_cmaf_files_hls_uses() {
        let xml = mpd(&Ladder::standard(), &reports(), 30.0);
        assert!(xml.contains("initialization=\"$RepresentationID$/init.mp4\""));
        assert!(xml.contains("media=\"$RepresentationID$/seg_$Number%05d$.m4s\""));
        assert!(xml.contains("duration=\"2000\""));
    }

    #[test]
    fn every_rendition_becomes_a_representation_with_its_bandwidth() {
        let xml = mpd(&Ladder::standard(), &reports(), 30.0);
        assert_eq!(xml.matches("<Representation ").count(), 4);
        assert!(xml.contains("id=\"1080p\" bandwidth=\"5000000\" width=\"1920\" height=\"1080\" codecs=\"avc1.640028\""));
    }
}
