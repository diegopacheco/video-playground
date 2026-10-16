use crate::config::{AnalysisConfig, Title};
use crate::encoder::Encoder;
use crate::error::{AppError, Result};
use crate::hull::{Axis, convex_hull};
use crate::ladder::{build_ladder, compare_generic, total_savings};
use crate::metric::QualityMetric;
use crate::model::{Analysis, Point, Rendition, TitleReport};
use crate::source::SourceGenerator;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub struct Pipeline {
    pub config: AnalysisConfig,
    pub source: Arc<dyn SourceGenerator>,
    pub encoder: Arc<dyn Encoder>,
    pub metrics: Vec<Arc<dyn QualityMetric>>,
    pub work_dir: PathBuf,
}

struct Job {
    title: usize,
    rendition: Rendition,
}

impl Pipeline {
    pub fn total_jobs(&self) -> usize {
        self.config.titles.len() * self.config.grid().len()
    }

    fn source_path(&self, title: &Title) -> PathBuf {
        self.work_dir.join("sources").join(format!("{}.mp4", title.id))
    }

    fn measure(&self, source: &Path, title: &Title, rendition: &Rendition) -> Result<Point> {
        let dir = self.work_dir.join("encodes").join(&title.id);
        std::fs::create_dir_all(&dir)?;
        let output = dir.join(rendition.file_name());
        let bytes = self.encoder.encode(source, rendition, &output)?;
        let bitrate_kbps = bytes as f64 * 8.0 / self.config.duration_secs / 1000.0;
        let mut scores = BTreeMap::new();
        for metric in &self.metrics {
            let value = metric.measure(source, &output, self.config.source)?;
            scores.insert(metric.name().to_string(), value);
        }
        Ok(Point {
            resolution: rendition.resolution,
            target_kbps: rendition.kbps,
            bitrate_kbps,
            scores,
        })
    }

    fn measure_all(&self, jobs: &[Job], progress: &(dyn Fn(usize, usize) + Sync)) -> Result<Vec<Point>> {
        let next = AtomicUsize::new(0);
        let done = AtomicUsize::new(0);
        let failed = AtomicBool::new(false);
        let results: Mutex<Vec<Option<Result<Point>>>> = Mutex::new((0..jobs.len()).map(|_| None).collect());
        let sources: Vec<PathBuf> = self.config.titles.iter().map(|t| self.source_path(t)).collect();
        std::thread::scope(|scope| {
            for _ in 0..self.config.workers.max(1) {
                scope.spawn(|| {
                    loop {
                        let i = next.fetch_add(1, Ordering::SeqCst);
                        if i >= jobs.len() || failed.load(Ordering::SeqCst) {
                            break;
                        }
                        let job = &jobs[i];
                        let title = &self.config.titles[job.title];
                        let result = self.measure(&sources[job.title], title, &job.rendition);
                        if result.is_err() {
                            failed.store(true, Ordering::SeqCst);
                        }
                        if let Ok(mut r) = results.lock() {
                            r[i] = Some(result);
                        }
                        progress(done.fetch_add(1, Ordering::SeqCst) + 1, jobs.len());
                    }
                });
            }
        });
        let results = results
            .into_inner()
            .map_err(|_| AppError::Ffmpeg("worker poisoned the result lock".into()))?;
        let mut points = Vec::with_capacity(jobs.len());
        for r in results {
            match r {
                Some(Ok(p)) => points.push(p),
                Some(Err(e)) => return Err(e),
                None => {}
            }
        }
        if points.len() != jobs.len() {
            return Err(AppError::Ffmpeg("analysis stopped before every encode finished".into()));
        }
        Ok(points)
    }

    fn report(&self, title: &Title, points: Vec<Point>) -> TitleReport {
        let metric = self.config.ladder_metric.as_str();
        let axis_of = |m: &Arc<dyn QualityMetric>| Axis {
            metric: m.name(),
            min_gain: m.min_gain(),
        };
        let hulls: BTreeMap<String, Vec<Point>> = self
            .metrics
            .iter()
            .map(|m| (m.name().to_string(), convex_hull(&points, axis_of(m))))
            .collect();
        let axis = self
            .metrics
            .iter()
            .find(|m| m.name() == metric)
            .map(axis_of)
            .unwrap_or(Axis { metric, min_gain: 0.0 });
        let hull = hulls.get(metric).cloned().unwrap_or_default();
        let ladder = build_ladder(&hull, metric, &self.config.targets);
        let generic = compare_generic(&points, &hull, axis, &self.config.generic_ladder);
        let savings = total_savings(&generic);
        TitleReport {
            id: title.id.clone(),
            name: title.name.clone(),
            description: title.description.clone(),
            points,
            hulls,
            ladder,
            generic,
            savings,
        }
    }

    pub fn run(&self, progress: &(dyn Fn(usize, usize) + Sync)) -> Result<Analysis> {
        self.config.validate()?;
        if !self.metrics.iter().any(|m| m.name() == self.config.ladder_metric) {
            return Err(AppError::Config(format!(
                "ladder metric {} is not measured",
                self.config.ladder_metric
            )));
        }
        let started = Instant::now();
        std::fs::create_dir_all(self.work_dir.join("sources"))?;
        for title in &self.config.titles {
            self.source.generate(title, &self.source_path(title))?;
        }
        let grid = self.config.grid();
        let jobs: Vec<Job> = (0..self.config.titles.len())
            .flat_map(|t| grid.iter().map(move |r| Job { title: t, rendition: *r }))
            .collect();
        let points = self.measure_all(&jobs, progress)?;
        let per_title = grid.len();
        let titles = self
            .config
            .titles
            .iter()
            .enumerate()
            .map(|(i, t)| self.report(t, points[i * per_title..(i + 1) * per_title].to_vec()))
            .collect();
        Ok(Analysis {
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            elapsed_secs: started.elapsed().as_secs_f64(),
            encoder: self.encoder.name(),
            metrics: self.metrics.iter().map(|m| m.name().to_string()).collect(),
            ladder_metric: self.config.ladder_metric.clone(),
            targets: self.config.targets.clone(),
            titles,
        })
    }
}
