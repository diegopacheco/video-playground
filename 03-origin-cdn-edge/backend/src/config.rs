use crate::error::CdnError;
use std::str::FromStr;

pub fn env_or<T: FromStr>(name: &str, default: T) -> Result<T, CdnError> {
    match std::env::var(name) {
        Ok(raw) => raw
            .trim()
            .parse::<T>()
            .map_err(|_| CdnError::Config(format!("{name}={raw} is not valid"))),
        Err(_) => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_variable_falls_back_to_default() {
        assert_eq!(env_or("VP03_UNSET_VARIABLE", 42u64), Ok(42));
    }
}
