use crate::error::{AppError, Result};
use std::process::Command;

pub fn run(args: &[String]) -> Result<String> {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-nostdin")
        .args(args)
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        Ok(stderr)
    } else {
        let tail: Vec<&str> = stderr.lines().rev().take(5).collect();
        Err(AppError::Ffmpeg(tail.into_iter().rev().collect::<Vec<_>>().join(" | ")))
    }
}

pub fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}
