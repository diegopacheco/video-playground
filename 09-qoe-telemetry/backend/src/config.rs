use std::env;
use std::str::FromStr;

pub struct Config {
    pub port: u16,
    pub brokers: String,
    pub topic: String,
    pub group: String,
    pub window_ms: i64,
    pub lateness_ms: i64,
}

fn var<T: FromStr>(name: &str, default: T) -> T {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: var("BACKEND_PORT", 8109),
            brokers: var("KAFKA_BROKERS", "localhost:6109".to_string()),
            topic: var("KAFKA_TOPIC", "qoe-beacons".to_string()),
            group: var("KAFKA_GROUP", "qoe-aggregator".to_string()),
            window_ms: var("WINDOW_MS", 10_000),
            lateness_ms: var("LATENESS_MS", 2_000),
        }
    }
}
