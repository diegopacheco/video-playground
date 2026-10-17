use crate::error::{AppError, Result};
use crate::model::{Rendition, Resolution};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Title {
    pub id: String,
    pub name: String,
    pub description: String,
    pub graph: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisConfig {
    pub source: Resolution,
    pub fps: u32,
    pub duration_secs: f64,
    pub resolutions: Vec<Resolution>,
    pub bitrates_kbps: Vec<u32>,
    pub targets: Vec<f64>,
    pub ladder_metric: String,
    pub generic_ladder: Vec<Rendition>,
    pub workers: usize,
    pub titles: Vec<Title>,
}

impl AnalysisConfig {
    pub fn grid(&self) -> Vec<Rendition> {
        self.resolutions
            .iter()
            .flat_map(|r| {
                self.bitrates_kbps.iter().map(|k| Rendition {
                    resolution: *r,
                    kbps: *k,
                })
            })
            .collect()
    }

    pub fn validate(&self) -> Result<()> {
        if self.resolutions.is_empty() || self.bitrates_kbps.is_empty() {
            return Err(AppError::Config("grid must not be empty".into()));
        }
        if self.titles.is_empty() {
            return Err(AppError::Config("at least one title is required".into()));
        }
        let grid = self.grid();
        for rung in &self.generic_ladder {
            if !grid.contains(rung) {
                return Err(AppError::Config(format!(
                    "generic rung {} @ {} kbps is not in the encoding grid",
                    rung.resolution.label(),
                    rung.kbps
                )));
            }
        }
        if self.targets.windows(2).any(|w| w[0] >= w[1]) {
            return Err(AppError::Config("targets must be strictly increasing".into()));
        }
        Ok(())
    }
}

const SOURCE: Resolution = Resolution::new(640, 360);
const FPS: u32 = 24;
const DURATION: f64 = 4.0;

fn anime_graph() -> String {
    let mut buildings = String::new();
    for i in 0..8u32 {
        let x = i * 160 + 20;
        let h = 80 + (i * 37) % 120;
        let y = 250 - h;
        let color = format!("0x{:02x}{:02x}{:02x}", 120 + i * 15, 90 + i * 10, 160 - i * 8);
        buildings.push_str(&format!(
            ",drawbox=x={x}:y={y}:w=110:h={h}:color={color}:t=fill,drawbox=x={x}:y={y}:w=110:h={h}:color=black:t=3"
        ));
        for k in 0..(h - 20) / 30 {
            for j in 0..2 {
                let wx = x + 15 + j * 50;
                let wy = y + 15 + k * 30;
                buildings.push_str(&format!(
                    ",drawbox=x={wx}:y={wy}:w=30:h=18:color=0xfff3b0:t=fill,drawbox=x={wx}:y={wy}:w=30:h=18:color=black:t=2"
                ));
            }
        }
    }
    format!(
        "color=c=0x9ad0f5:s=1280x{h}:r={FPS}:d={DURATION}[sky];\
color=c=0xf4a261:s=120x100:r={FPS}:d={DURATION}[hero];\
color=c=0xe63946:s=60x60:r={FPS}:d={DURATION}[ball];\
[sky]drawbox=x=0:y=250:w=1280:h=110:color=0x6cbf5a:t=fill{buildings},crop={w}:{h}:x='t*150':y=0,\
drawbox=x=500:y=30:w=70:h=70:color=0xffe066:t=fill[bg];\
[hero]drawbox=x=0:y=0:w=120:h=100:color=black:t=4,drawbox=x=20:y=20:w=25:h=20:color=white:t=fill,\
drawbox=x=70:y=20:w=25:h=20:color=white:t=fill[c];\
[bg][c]overlay=x='40+90*t':y='150+8*sin(t*9)'[a];\
[a][ball]overlay=x='560-50*t':y='200-30*abs(sin(t*4))',format=yuv420p",
        w = SOURCE.width,
        h = SOURCE.height
    )
}

fn live_action_graph() -> String {
    let frames = (FPS as f64 * DURATION) as u32;
    format!(
        "mandelbrot=s={w}x{h}:r={FPS}:maxiter=120:start_scale=2.5:end_scale=0.6:end_pts={frames}[m];\
life=s={w}x{h}:r={FPS}:mold=10:ratio=0.3:death_color=#202020:life_color=#c0c0c0[l];\
[m][l]blend=all_mode=overlay:all_opacity=0.15,noise=alls=4:allf=t,gblur=sigma=0.6,trim=duration={DURATION},format=yuv420p",
        w = SOURCE.width,
        h = SOURCE.height
    )
}

pub fn default_titles() -> Vec<Title> {
    vec![
        Title {
            id: "anime".into(),
            name: "Anime-like".into(),
            description: "Flat colors, hard black outlines, a slow pan and two simple moving shapes".into(),
            graph: anime_graph(),
        },
        Title {
            id: "live-action".into(),
            name: "Live-action-like".into(),
            description: "Zooming fractal texture blended with cellular motion and temporal film grain".into(),
            graph: live_action_graph(),
        },
    ]
}

fn rung(width: u32, height: u32, kbps: u32) -> Rendition {
    Rendition {
        resolution: Resolution::new(width, height),
        kbps,
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            source: SOURCE,
            fps: FPS,
            duration_secs: DURATION,
            resolutions: vec![
                Resolution::new(320, 180),
                Resolution::new(480, 270),
                Resolution::new(640, 360),
            ],
            bitrates_kbps: vec![30, 45, 60, 100, 160, 250, 400, 650, 1000, 1600, 2600, 4000],
            targets: vec![60.0, 75.0, 85.0, 93.0],
            ladder_metric: "vmaf".into(),
            generic_ladder: vec![
                rung(320, 180, 250),
                rung(480, 270, 650),
                rung(640, 360, 1600),
                rung(640, 360, 2600),
            ],
            workers: 3,
            titles: default_titles(),
        }
    }
}
