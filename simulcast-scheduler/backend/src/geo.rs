use std::net::{IpAddr, Ipv4Addr};

pub const UNKNOWN_COUNTRY: &str = "ZZ";

pub const COUNTRIES: &[(&str, &str)] = &[
    ("US", "America/New_York"),
    ("BR", "America/Sao_Paulo"),
    ("GB", "Europe/London"),
    ("DE", "Europe/Berlin"),
    ("IN", "Asia/Kolkata"),
    ("JP", "Asia/Tokyo"),
    ("KR", "Asia/Seoul"),
    ("CN", "Asia/Shanghai"),
    ("AU", "Australia/Sydney"),
];

pub fn timezone_for(country: &str) -> &'static str {
    COUNTRIES
        .iter()
        .find(|(code, _)| *code == country)
        .map(|(_, tz)| *tz)
        .unwrap_or("UTC")
}

pub trait GeoResolver: Send + Sync {
    fn resolve(&self, country_header: Option<&str>, ip: Option<IpAddr>) -> String;
}

pub struct IpRange {
    pub start: Ipv4Addr,
    pub end: Ipv4Addr,
    pub country: String,
}

pub struct HeaderThenIpResolver {
    ranges: Vec<IpRange>,
}

impl HeaderThenIpResolver {
    pub fn new(ranges: Vec<IpRange>) -> Self {
        Self { ranges }
    }

    pub fn with_default_table() -> Self {
        let ranges = COUNTRIES
            .iter()
            .enumerate()
            .map(|(i, (code, _))| {
                let octet = (i + 1) as u8;
                IpRange {
                    start: Ipv4Addr::new(10, octet, 0, 0),
                    end: Ipv4Addr::new(10, octet, 255, 255),
                    country: (*code).to_string(),
                }
            })
            .collect();
        Self::new(ranges)
    }

    fn lookup_ip(&self, ip: IpAddr) -> Option<String> {
        let IpAddr::V4(v4) = ip else { return None };
        let n = u32::from(v4);
        self.ranges
            .iter()
            .find(|r| u32::from(r.start) <= n && n <= u32::from(r.end))
            .map(|r| r.country.clone())
    }
}

fn valid_code(header: &str) -> Option<String> {
    let code = header.trim().to_ascii_uppercase();
    (code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase())).then_some(code)
}

impl GeoResolver for HeaderThenIpResolver {
    fn resolve(&self, country_header: Option<&str>, ip: Option<IpAddr>) -> String {
        country_header
            .and_then(valid_code)
            .or_else(|| ip.and_then(|addr| self.lookup_ip(addr)))
            .unwrap_or_else(|| UNKNOWN_COUNTRY.to_string())
    }
}
