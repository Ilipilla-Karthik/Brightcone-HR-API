/// Employee lookups and date helpers shared by the record rules and leave flow.
use chrono::Datelike;

/// Count weekdays (Mon-Fri) between two dates inclusive, skipping Sat (6) and Sun (0).
#[must_use]
pub fn weekdays(from: &str, to: &str) -> i32 {
    let d = chrono::NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let e = chrono::NaiveDate::parse_from_str(to, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    if e < d {
        return 0;
    }
    let mut n = 0;
    let mut cur = d;
    while cur <= e {
        let w = cur.weekday().num_days_from_monday(); // 0=Mon, 4=Fri, 5=Sat, 6=Sun
        if w != 5 && w != 6 {
            n += 1;
        }
        cur += chrono::Duration::try_days(1).unwrap_or(chrono::Duration::zero());
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekdays() {
        // Mon Jan 1 2024 to Fri Jan 5 2024 = 5 weekdays
        assert_eq!(weekdays("2024-01-01", "2024-01-05"), 5);
    }

    #[test]
    fn test_weekend_dates() {
        // Sat Jan 6 to Sun Jan 7 = 0 weekdays
        assert_eq!(weekdays("2024-01-06", "2024-01-07"), 0);
    }

    #[test]
    fn test_mixed_range() {
        // Fri Jan 5 to Mon Jan 8 = 2 weekdays (Fri + Mon)
        assert_eq!(weekdays("2024-01-05", "2024-01-08"), 2);
    }
}
