use per_title::config::AnalysisConfig;
use per_title::encoder::X264Cbr;
use per_title::metric::{parse_ssim, parse_vmaf};
use per_title::model::{Rendition, Resolution};
use std::path::Path;

#[test]
fn vmaf_score_is_read_from_the_libvmaf_summary_line() {
    let out = "frame=   96 fps=0.0 q=-0.0 Lsize=N/A\n[Parsed_libvmaf_1 @ 0x1] VMAF score: 87.654321\n";
    assert_eq!(parse_vmaf(out).ok(), Some(87.654321));
}

#[test]
fn ssim_uses_the_all_planes_value_not_the_luma_one() {
    let out = "[Parsed_ssim_1 @ 0x1] SSIM Y:0.991 (20.4) U:0.97 (15.2) V:0.96 (14.0) All:0.983012 (17.7)\n";
    assert_eq!(parse_ssim(out).ok(), Some(0.983012));
}

#[test]
fn missing_score_is_an_error_instead_of_a_silent_zero() {
    assert!(parse_vmaf("no score here").is_err());
    assert!(parse_ssim("SSIM nothing").is_err());
}

#[test]
fn default_config_is_valid_and_small_enough_to_run_in_minutes() {
    let c = AnalysisConfig::default();
    assert!(c.validate().is_ok());
    let jobs = c.grid().len() * c.titles.len();
    assert!(jobs <= 100, "{jobs} encodes would make the analysis too slow");
    assert!(c.duration_secs <= 5.0);
}

#[test]
fn generic_rung_outside_the_grid_is_rejected_because_its_quality_would_be_unknown() {
    let mut c = AnalysisConfig::default();
    c.generic_ladder.push(Rendition {
        resolution: Resolution::new(1920, 1080),
        kbps: 5800,
    });
    assert!(c.validate().is_err());
}

#[test]
fn x264_is_driven_in_cbr_so_measured_bitrate_matches_the_rung_budget() {
    let r = Rendition {
        resolution: Resolution::new(480, 270),
        kbps: 650,
    };
    let args = X264Cbr::default().arguments(Path::new("in.mp4"), &r, Path::new("out.mp4"));
    let joined = args.join(" ");
    assert!(joined.contains("-b:v 650k -minrate 650k -maxrate 650k"));
    assert!(joined.contains("nal-hrd=cbr"));
    assert!(joined.contains("scale=480:270"));
}
