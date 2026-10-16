use crate::AbrError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Trace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bandwidth_kbps: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub duration_s: usize,
    pub mean_kbps: f64,
    pub min_kbps: f64,
    pub max_kbps: f64,
    pub bandwidth_kbps: Vec<f64>,
}

const BUNDLED: [(&str, &str, &str, &str); 4] = [
    (
        "3g",
        "3G mobile",
        "Low bandwidth around 0.8 Mbps with random walk noise and occasional deep fades",
        include_str!("../traces/3g.csv"),
    ),
    (
        "4g-variable",
        "4G variable",
        "LTE with regime changes between 1 and 17 Mbps every half minute and strong jitter",
        include_str!("../traces/4g-variable.csv"),
    ),
    (
        "wifi-stable",
        "WiFi stable",
        "Home WiFi steady around 6.5 Mbps with small jitter",
        include_str!("../traces/wifi-stable.csv"),
    ),
    (
        "dropouts",
        "Drop-outs",
        "About 4 Mbps with near total outages of 8 to 25 seconds roughly every minute",
        include_str!("../traces/dropouts.csv"),
    ),
];

impl Trace {
    pub fn new(id: &str, bandwidth_kbps: Vec<f64>) -> Result<Trace, AbrError> {
        if !bandwidth_kbps.iter().any(|b| *b > 0.0) {
            return Err(AbrError::EmptyTrace(id.to_string()));
        }
        Ok(Trace {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            bandwidth_kbps,
        })
    }

    pub fn parse(id: &str, csv: &str) -> Result<Trace, AbrError> {
        let mut values = Vec::new();
        for (number, line) in csv.lines().enumerate().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let fail = |reason: &str| AbrError::TraceParse {
                trace: id.to_string(),
                line: number + 1,
                reason: reason.to_string(),
            };
            let field = line.split(',').nth(1).ok_or_else(|| fail("missing bandwidth column"))?;
            let value: f64 = field.trim().parse().map_err(|_| fail("bandwidth is not a number"))?;
            if !value.is_finite() || value < 0.0 {
                return Err(fail("bandwidth must be a non negative number"));
            }
            values.push(value);
        }
        Trace::new(id, values)
    }

    pub fn bandwidth_at(&self, time_s: f64) -> f64 {
        self.bandwidth_kbps[(time_s.max(0.0).floor() as usize) % self.bandwidth_kbps.len()]
    }

    pub fn download_time(&self, start_s: f64, kbits: f64) -> f64 {
        let mut remaining = kbits;
        let mut now = start_s;
        while remaining > 0.0 {
            let slot_end = now.floor() + 1.0;
            let bandwidth = self.bandwidth_at(now);
            let available = bandwidth * (slot_end - now);
            if bandwidth > 0.0 && available >= remaining {
                now += remaining / bandwidth;
                break;
            }
            remaining -= available;
            now = slot_end;
        }
        now - start_s
    }

    pub fn summary(&self) -> TraceSummary {
        let values = &self.bandwidth_kbps;
        TraceSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            duration_s: values.len(),
            mean_kbps: values.iter().sum::<f64>() / values.len() as f64,
            min_kbps: values.iter().cloned().fold(f64::INFINITY, f64::min),
            max_kbps: values.iter().cloned().fold(0.0, f64::max),
            bandwidth_kbps: values.clone(),
        }
    }
}

pub fn bundled() -> Result<Vec<Trace>, AbrError> {
    BUNDLED
        .iter()
        .map(|(id, name, description, csv)| {
            let mut trace = Trace::parse(id, csv)?;
            trace.name = name.to_string();
            trace.description = description.to_string();
            Ok(trace)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_spans_slots_and_waits_through_zero_bandwidth_outage() {
        let trace = Trace::new("t", vec![1000.0, 0.0, 1000.0]).unwrap();
        let time = trace.download_time(0.0, 1500.0);
        assert!((time - 2.5).abs() < 1e-9, "outage second must add a full second, got {time}");
    }

    #[test]
    fn download_starting_mid_slot_only_uses_remaining_part_of_that_second() {
        let trace = Trace::new("t", vec![1000.0, 2000.0]).unwrap();
        let time = trace.download_time(0.5, 1500.0);
        assert!((time - 1.0).abs() < 1e-9, "0.5s at 1000 then 0.5s at 2000, got {time}");
    }

    #[test]
    fn trace_loops_so_long_sessions_never_run_out_of_network() {
        let trace = Trace::new("t", vec![1000.0, 3000.0]).unwrap();
        assert_eq!(trace.bandwidth_at(4.2), 1000.0);
        assert_eq!(trace.bandwidth_at(5.9), 3000.0);
    }

    #[test]
    fn all_zero_trace_is_rejected_because_downloads_would_never_finish() {
        assert_eq!(Trace::new("dead", vec![0.0, 0.0]).unwrap_err(), AbrError::EmptyTrace("dead".into()));
    }

    #[test]
    fn malformed_csv_reports_the_line() {
        let err = Trace::parse("bad", "time_s,bandwidth_kbps\n0,100\n1,abc\n").unwrap_err();
        assert!(matches!(err, AbrError::TraceParse { line: 3, .. }));
    }

    #[test]
    fn bundled_traces_cover_ten_minutes_each() {
        let traces = bundled().unwrap();
        assert_eq!(traces.len(), 4);
        assert!(traces.iter().all(|t| t.bandwidth_kbps.len() == 600));
        let dropouts = traces.iter().find(|t| t.id == "dropouts").unwrap();
        assert!(dropouts.bandwidth_kbps.contains(&0.0), "drop-out trace must contain real outages");
    }
}
