mod common;

use common::temp_dir;
use per_title::config::Title;
use per_title::encoder::{Encoder, X264Cbr};
use per_title::metric::{QualityMetric, Ssim, Vmaf};
use per_title::model::{Rendition, Resolution};
use per_title::source::{LavfiSource, SourceGenerator};

#[test]
fn real_ffmpeg_gives_higher_quality_for_more_bits_on_textured_content() {
    let dir = temp_dir("ffmpeg");
    std::fs::create_dir_all(&dir).expect("dir");
    let source = dir.join("src.mp4");
    let title = Title {
        id: "t".into(),
        name: "t".into(),
        description: String::new(),
        graph: "testsrc2=s=320x180:r=24:d=1,noise=alls=10:allf=t,format=yuv420p".into(),
    };
    LavfiSource.generate(&title, &source).expect("source");
    let display = Resolution::new(320, 180);
    let encoder = X264Cbr {
        preset: "ultrafast".into(),
    };
    let mut scores = Vec::new();
    for kbps in [60, 1500] {
        let r = Rendition {
            resolution: display,
            kbps,
        };
        let out = dir.join(r.file_name());
        let bytes = encoder.encode(&source, &r, &out).expect("encode");
        assert!(bytes > 0);
        let vmaf = Vmaf { threads: 2 }.measure(&source, &out, display).expect("vmaf");
        let ssim = Ssim.measure(&source, &out, display).expect("ssim");
        assert!((0.0..=100.0).contains(&vmaf));
        assert!((0.0..=1.0).contains(&ssim));
        scores.push((vmaf, ssim));
    }
    assert!(scores[1].0 > scores[0].0, "vmaf {:?}", scores);
    assert!(scores[1].1 > scores[0].1, "ssim {:?}", scores);
}
