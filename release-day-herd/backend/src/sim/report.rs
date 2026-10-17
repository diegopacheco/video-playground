use serde::Serialize;

pub const BUCKET_MS: u64 = 100;
const BUCKET_US: u64 = BUCKET_MS * 1_000;
const BUCKETS_PER_SECOND: usize = 10;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Bucket {
    pub t_ms: u64,
    pub api_requests: u64,
    pub api_rejected: u64,
    pub origin_requests: u64,
    pub origin_rejected: u64,
    pub first_frames: u64,
    pub admitted: u64,
    pub retries: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SimReport {
    pub strategy: String,
    pub bucket_ms: u64,
    pub users: u64,
    pub succeeded: u64,
    pub abandoned: u64,
    pub success_rate: f64,
    pub peak_origin_rps: u64,
    pub peak_api_rps: u64,
    pub max_admitted_per_s: u64,
    pub origin_requests: u64,
    pub api_requests: u64,
    pub retries: u64,
    pub wasted_retries: u64,
    pub prewarmed_objects: u64,
    pub ttff_p50_ms: Option<u64>,
    pub ttff_p95_ms: Option<u64>,
    pub ttff_p99_ms: Option<u64>,
    pub series: Vec<Bucket>,
}

#[derive(Debug, Default)]
pub struct Recorder {
    series: Vec<Bucket>,
    ttff_ms: Vec<u64>,
    abandoned: u64,
    retries: u64,
    wasted_retries: u64,
}

impl Recorder {
    fn at(&mut self, t_us: u64) -> &mut Bucket {
        let slot = (t_us / BUCKET_US) as usize;
        if self.series.len() <= slot {
            let start = self.series.len() as u64;
            self.series.extend((start..=slot as u64).map(|s| Bucket { t_ms: s * BUCKET_MS, ..Bucket::default() }));
        }
        &mut self.series[slot]
    }

    pub fn api(&mut self, t_us: u64, accepted: bool) {
        let b = self.at(t_us);
        b.api_requests += 1;
        b.api_rejected += u64::from(!accepted);
    }

    pub fn origin(&mut self, t_us: u64, accepted: bool) {
        let b = self.at(t_us);
        b.origin_requests += 1;
        b.origin_rejected += u64::from(!accepted);
    }

    pub fn admitted(&mut self, t_us: u64) {
        self.at(t_us).admitted += 1;
    }

    pub fn retry(&mut self, t_us: u64) {
        self.retries += 1;
        self.at(t_us).retries += 1;
    }

    pub fn wasted_retry(&mut self) {
        self.wasted_retries += 1;
    }

    pub fn first_frame(&mut self, t_us: u64, ttff_us: u64) {
        self.at(t_us).first_frames += 1;
        self.ttff_ms.push(ttff_us / 1_000);
    }

    pub fn abandon(&mut self) {
        self.abandoned += 1;
    }

    pub fn finish(mut self, strategy: &str, users: u64, prewarmed_objects: u64) -> SimReport {
        self.ttff_ms.sort_unstable();
        let succeeded = self.ttff_ms.len() as u64;
        let seconds = per_second(&self.series);
        let peak = |f: fn(&Bucket) -> u64| seconds.iter().map(f).max().unwrap_or(0);
        SimReport {
            strategy: strategy.to_string(),
            bucket_ms: BUCKET_MS,
            users,
            succeeded,
            abandoned: self.abandoned,
            success_rate: succeeded as f64 / users.max(1) as f64,
            peak_origin_rps: peak(|b| b.origin_requests),
            peak_api_rps: peak(|b| b.api_requests),
            max_admitted_per_s: peak(|b| b.admitted),
            origin_requests: self.series.iter().map(|b| b.origin_requests).sum(),
            api_requests: self.series.iter().map(|b| b.api_requests).sum(),
            retries: self.retries,
            wasted_retries: self.wasted_retries,
            prewarmed_objects,
            ttff_p50_ms: percentile(&self.ttff_ms, 50),
            ttff_p95_ms: percentile(&self.ttff_ms, 95),
            ttff_p99_ms: percentile(&self.ttff_ms, 99),
            series: self.series,
        }
    }
}

pub fn per_second(series: &[Bucket]) -> Vec<Bucket> {
    series
        .chunks(BUCKETS_PER_SECOND)
        .enumerate()
        .map(|(i, chunk)| {
            chunk.iter().fold(Bucket { t_ms: i as u64 * 1_000, ..Bucket::default() }, |acc, b| Bucket {
                t_ms: acc.t_ms,
                api_requests: acc.api_requests + b.api_requests,
                api_rejected: acc.api_rejected + b.api_rejected,
                origin_requests: acc.origin_requests + b.origin_requests,
                origin_rejected: acc.origin_rejected + b.origin_rejected,
                first_frames: acc.first_frames + b.first_frames,
                admitted: acc.admitted + b.admitted,
                retries: acc.retries + b.retries,
            })
        })
        .collect()
}

pub fn percentile(sorted: &[u64], p: u64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = (p as usize * sorted.len()).div_ceil(100).max(1);
    Some(sorted[rank - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_use_nearest_rank() {
        let v: Vec<u64> = (1..=100).collect();
        assert_eq!(percentile(&v, 50), Some(50));
        assert_eq!(percentile(&v, 99), Some(99));
        assert_eq!(percentile(&[7], 99), Some(7));
        assert_eq!(percentile(&[], 50), None);
    }

    #[test]
    fn rejected_requests_still_count_as_load() {
        let mut r = Recorder::default();
        r.origin(10, false);
        r.origin(20, true);
        let report = r.finish("x", 1, 0);
        assert_eq!(report.peak_origin_rps, 2);
        assert_eq!(report.series[0].origin_rejected, 1);
    }

    #[test]
    fn peak_rps_sums_a_whole_second_not_one_fine_bucket() {
        let mut r = Recorder::default();
        for i in 0..10 {
            r.origin(i * BUCKET_US + 1, true);
        }
        r.origin(1_000_001, true);
        let report = r.finish("x", 1, 0);
        assert_eq!(report.series.len(), 11);
        assert_eq!(report.peak_origin_rps, 10);
    }
}
