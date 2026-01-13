use chrono::Duration;
use regex::Regex;

pub fn parse_duration(s: &str) -> Option<Duration> {
    let re = Regex::new(r"(\d+)([dhms])").unwrap();
    let mut total_seconds = 0i64;
    let mut found = false;

    for cap in re.captures_iter(s) {
        found = true;
        let value: i64 = cap[1].parse().ok()?;
        let unit = &cap[2];

        total_seconds += match unit {
            "d" => value * 24 * 3600,
            "h" => value * 3600,
            "m" => value * 60,
            "s" => value,
            _ => 0,
        };
    }

    if found {
        Some(Duration::seconds(total_seconds))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("10m30s"), Some(Duration::seconds(630)));
        assert_eq!(parse_duration("1h30m"), Some(Duration::seconds(5400)));
        assert_eq!(parse_duration("1d"), Some(Duration::seconds(86400)));
        assert_eq!(parse_duration("invalid"), None);
    }
}
