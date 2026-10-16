use packager::command;
use packager::encoder::{Encoder, FfmpegEncoder};
use packager::error::Result;
use packager::ladder::{Ladder, Rendition};
use packager::packager::FfmpegCmafPackager;
use packager::pipeline::{DASH_MANIFEST, MASTER_PLAYLIST, Pipeline};
use packager::probe::FfprobeKeyframes;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vp01-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn small_ladder() -> Ladder {
    Ladder {
        fps: 24,
        gop_frames: 48,
        segment_seconds: 2,
        renditions: vec![
            Rendition::new("144p", 256, 144, 200, 30),
            Rendition::new("240p", 426, 240, 400, 30),
            Rendition::new("360p", 640, 360, 800, 30),
        ],
    }
}

fn make_source(dir: &Path) -> PathBuf {
    let source = dir.join("source.mp4");
    let args: Vec<String> = [
        "-y",
        "-v",
        "error",
        "-f",
        "lavfi",
        "-i",
        "testsrc2=size=640x360:rate=30:duration=6,noise=alls=10:allf=t",
        "-c:v",
        "libx264",
        "-preset",
        "ultrafast",
        "-pix_fmt",
        "yuv420p",
    ]
    .iter()
    .map(|s| s.to_string())
    .chain(std::iter::once(source.display().to_string()))
    .collect();
    command::run("ffmpeg", &args).unwrap();
    source
}

fn pipeline(ladder: Ladder, encoder: Box<dyn Encoder>) -> Pipeline {
    Pipeline {
        ladder,
        encoder,
        packager: Box::new(FfmpegCmafPackager),
        probe: Box::new(FfprobeKeyframes),
    }
}

fn first_frame_is_key(init: &Path, segment: &Path, scratch: &Path) -> bool {
    let mut bytes = std::fs::read(init).unwrap();
    bytes.extend(std::fs::read(segment).unwrap());
    std::fs::write(scratch, bytes).unwrap();
    let args: Vec<String> = [
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-read_intervals",
        "%+#1",
        "-show_entries",
        "frame=key_frame",
        "-of",
        "csv=p=0",
    ]
    .iter()
    .map(|s| s.to_string())
    .chain(std::iter::once(scratch.display().to_string()))
    .collect();
    command::run("ffprobe", &args).unwrap().lines().next().map(|l| l.trim().starts_with('1')).unwrap_or(false)
}

#[test]
fn fixed_gop_ladder_has_identical_keyframes_in_every_rendition_and_every_segment_starts_with_one() {
    let dir = temp_dir("aligned");
    let source = make_source(&dir);
    let out = dir.join("out");
    let manifest = pipeline(small_ladder(), Box::new(FfmpegEncoder { preset: "ultrafast".into() }))
        .run(&source, &dir.join("work"), &out)
        .unwrap();

    assert!(manifest.alignment.aligned, "{:?}", manifest.alignment.issues);
    assert!(manifest.alignment.boundaries_on_keyframes, "{:?}", manifest.alignment.issues);
    assert_eq!(manifest.alignment.keyframe_times, vec![0.0, 2.0, 4.0]);

    for r in &manifest.renditions {
        assert_eq!(r.keyframes, manifest.alignment.keyframe_times, "{}", r.id);
        assert_eq!(r.segments.len(), 3, "{}", r.id);
        for s in &r.segments {
            let ok = first_frame_is_key(&out.join(&r.id).join("init.mp4"), &out.join(&s.uri), &dir.join("probe.mp4"));
            assert!(ok, "{} does not begin with a keyframe", s.uri);
        }
    }

    let sizes: Vec<u64> = manifest.renditions.iter().map(|r| r.total_bytes).collect();
    assert!(sizes.windows(2).all(|w| w[0] < w[1]), "higher rungs must cost more bytes: {sizes:?}");
    assert!(out.join(MASTER_PLAYLIST).exists());
    assert!(out.join(DASH_MANIFEST).exists());
}

struct DriftingEncoder {
    inner: FfmpegEncoder,
    drifting_id: String,
    gop_frames: u32,
}

impl Encoder for DriftingEncoder {
    fn encode(&self, source: &Path, rendition: &Rendition, ladder: &Ladder, output: &Path) -> Result<()> {
        let mut ladder = ladder.clone();
        if rendition.id == self.drifting_id {
            ladder.gop_frames = self.gop_frames;
        }
        self.inner.encode(source, rendition, &ladder, output)
    }
}

#[test]
fn one_rendition_with_a_different_gop_is_detected_as_misaligned() {
    let dir = temp_dir("drift");
    let source = make_source(&dir);
    let encoder = DriftingEncoder {
        inner: FfmpegEncoder { preset: "ultrafast".into() },
        drifting_id: "360p".into(),
        gop_frames: 36,
    };
    let manifest = pipeline(small_ladder(), Box::new(encoder))
        .run(&source, &dir.join("work"), &dir.join("out"))
        .unwrap();

    assert!(!manifest.alignment.aligned);
    assert!(manifest.alignment.issues.iter().any(|i| i.starts_with("360p")), "{:?}", manifest.alignment.issues);
}
