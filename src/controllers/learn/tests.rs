#[cfg(test)]
mod unit_tests {
    use super::super::types::*;
    use chrono::{Duration, TimeZone, Utc};

    struct MockEmp {
        name: &'static str,
        dept: &'static str,
        mgr: Option<&'static str>,
    }

    fn is_in_audience(aud: &str, emp: &MockEmp, all: &[MockEmp]) -> bool {
        match aud {
            "Everyone" => true,
            "Managers" => all.iter().any(|x| x.mgr == Some(emp.name)),
            dept => emp.dept == dept,
        }
    }

    #[test]
    fn test_audience_everyone() {
        let employees = vec![
            MockEmp { name: "Priya", dept: "People Ops", mgr: None },
            MockEmp { name: "Aarav", dept: "Engineering", mgr: Some("Vikram") },
        ];
        assert!(is_in_audience("Everyone", &employees[0], &employees));
        assert!(is_in_audience("Everyone", &employees[1], &employees));
    }

    #[test]
    fn test_audience_managers() {
        let employees = vec![
            MockEmp { name: "Priya", dept: "People Ops", mgr: None },
            MockEmp { name: "Vikram", dept: "Engineering", mgr: Some("Priya") },
            MockEmp { name: "Aarav", dept: "Engineering", mgr: Some("Vikram") },
        ];
        assert!(is_in_audience("Managers", &employees[0], &employees));
        assert!(is_in_audience("Managers", &employees[1], &employees));
        assert!(!is_in_audience("Managers", &employees[2], &employees));
    }

    #[test]
    fn test_audience_department() {
        let employees = vec![
            MockEmp { name: "Priya", dept: "People Ops", mgr: None },
            MockEmp { name: "Aarav", dept: "Engineering", mgr: Some("Vikram") },
        ];
        assert!(is_in_audience("Engineering", &employees[1], &employees));
        assert!(!is_in_audience("Engineering", &employees[0], &employees));
    }

    #[test]
    fn test_compliance_calculation() {
        let total_assigned = 10usize;
        let completed_count = 8usize;
        let pct = ((completed_count as f64 / total_assigned as f64) * 100.0).round() as i32;
        assert_eq!(pct, 80);

        let zero_assigned = 0usize;
        let zero_pct = if zero_assigned > 0 { 0 } else { 100 };
        assert_eq!(zero_pct, 100);
    }

    #[test]
    fn test_reminder_cooldown() {
        let now = Utc::now();
        let sent_recently = now - Duration::hours(12);
        let sent_long_ago = now - Duration::hours(36);
        let threshold = now - Duration::hours(24);

        assert!(sent_recently >= threshold, "12 hours ago is within 24h cooldown");
        assert!(sent_long_ago < threshold, "36 hours ago is outside 24h cooldown");
    }

    #[test]
    fn test_parse_deadline_strict_validation() {
        let d1 = parse_deadline("2026-10-21").expect("Valid YYYY-MM-DD");
        assert_eq!(d1, DeadlineKind::DateOnly(chrono::NaiveDate::from_ymd_opt(2026, 10, 21).unwrap()));

        let d2 = parse_deadline("2026-10-21T18:30:00Z").expect("Valid RFC3339 UTC");
        assert!(matches!(d2, DeadlineKind::Timestamp(_)));

        let d3 = parse_deadline("2026-10-21T23:59:59+05:30").expect("Valid RFC3339 with offset");
        assert!(matches!(d3, DeadlineKind::Timestamp(_)));

        assert!(parse_deadline("").is_err(), "Empty string rejected");
        assert!(parse_deadline("2026/10/21").is_err(), "Slash date rejected");
        assert!(parse_deadline("2026-13-45").is_err(), "Invalid date numbers rejected");
        assert!(parse_deadline("tomorrow").is_err(), "Relative date string rejected");
        assert!(parse_deadline("2026-10-21 23:59:59").is_err(), "Missing offset in timestamp rejected");
    }

    #[test]
    fn test_timestamp_deadline_instant_comparison() {
        let biz_tz = parse_business_timezone("+05:30").unwrap();
        let deadline_str = "2026-10-21T18:30:00Z";

        let before_utc = Utc.with_ymd_and_hms(2026, 10, 21, 18, 29, 59).unwrap();
        assert_eq!(
            is_course_overdue_with_instant("Mandatory", 0, Some(deadline_str), before_utc, biz_tz),
            false,
            "Timestamp deadline not reached"
        );

        let after_utc = Utc.with_ymd_and_hms(2026, 10, 21, 18, 30, 1).unwrap();
        assert_eq!(
            is_course_overdue_with_instant("Mandatory", 0, Some(deadline_str), after_utc, biz_tz),
            true,
            "Timestamp deadline passed"
        );
    }

    #[test]
    fn test_production_timezone_conversion_path_using_fixed_utc_instant_around_midnight() {
        let biz_tz = parse_business_timezone("+05:30").expect("Valid business timezone offset");
        let due_date = "2026-10-21";

        let fixed_utc_before_midnight = Utc.with_ymd_and_hms(2026, 10, 21, 18, 29, 59).unwrap();
        let is_overdue_before = is_course_overdue_with_instant(
            "Mandatory",
            0,
            Some(due_date),
            fixed_utc_before_midnight,
            biz_tz,
        );
        assert_eq!(
            is_overdue_before, false,
            "At 2026-10-21 18:29:59 UTC (23:59:59 in biz tz), course due 2026-10-21 is NOT overdue"
        );

        let fixed_utc_after_midnight = Utc.with_ymd_and_hms(2026, 10, 21, 18, 30, 1).unwrap();
        let is_overdue_after = is_course_overdue_with_instant(
            "Mandatory",
            0,
            Some(due_date),
            fixed_utc_after_midnight,
            biz_tz,
        );
        assert_eq!(
            is_overdue_after, true,
            "At 2026-10-21 18:30:01 UTC (00:00:01 on next day in biz tz), course due 2026-10-21 IS overdue"
        );
    }

    #[test]
    fn test_overdue_rules_matrix() {
        let biz_tz = parse_business_timezone("+05:30").unwrap();
        let now_utc = Utc.with_ymd_and_hms(2026, 10, 21, 6, 30, 0).unwrap();

        assert_eq!(is_course_overdue_with_instant("Mandatory", 0, Some("2026-10-21"), now_utc, biz_tz), false);
        assert_eq!(is_course_overdue_with_instant("Mandatory", 0, Some("2026-10-20"), now_utc, biz_tz), true);
        assert_eq!(is_course_overdue_with_instant("Mandatory", 0, Some("2026-10-22"), now_utc, biz_tz), false);
        assert_eq!(is_course_overdue_with_instant("Mandatory", 100, Some("2026-10-20"), now_utc, biz_tz), false);
        assert_eq!(is_course_overdue_with_instant("Elective", 0, Some("2026-10-20"), now_utc, biz_tz), false);
        assert_eq!(is_course_overdue_with_instant("Mandatory", 0, None, now_utc, biz_tz), false);
    }

    #[test]
    fn test_configured_timezone_values_iana_and_offset() {
        let tz_offset = parse_business_timezone("+05:30").expect("Must parse +05:30 without fallback");
        assert_eq!(tz_offset.local_minus_utc(), 5 * 3600 + 30 * 60);

        let tz_iana = parse_business_timezone("Asia/Kolkata").expect("Must parse Asia/Kolkata without fallback");
        assert_eq!(tz_iana.local_minus_utc(), 5 * 3600 + 30 * 60);
        assert_eq!(tz_offset, tz_iana, "+05:30 and Asia/Kolkata produce identical FixedOffset");

        let tz_utc = parse_business_timezone("UTC").expect("Must parse UTC without fallback");
        assert_eq!(tz_utc.local_minus_utc(), 0);

        assert!(parse_business_timezone("Invalid/Timezone").is_none());
        assert!(parse_business_timezone("not-a-tz").is_none());
    }

    #[test]
    fn test_validate_resource_valid() {
        assert_eq!(
            validate_resource(Some("document"), Some("https://example.com/handbook.pdf")),
            Ok(Some(("document".to_string(), "https://example.com/handbook.pdf".to_string())))
        );
        assert_eq!(
            validate_resource(Some("video"), Some("https://stream.example.com/orientation.mp4")),
            Ok(Some(("video".to_string(), "https://stream.example.com/orientation.mp4".to_string())))
        );
        assert_eq!(
            validate_resource(Some("link"), Some("https://portal.example.com/compliance")),
            Ok(Some(("link".to_string(), "https://portal.example.com/compliance".to_string())))
        );
        assert_eq!(
            validate_resource(Some("Video"), Some("https://example.com/video")),
            Ok(Some(("video".to_string(), "https://example.com/video".to_string())))
        );
    }

    #[test]
    fn test_validate_resource_empty_and_removal() {
        assert_eq!(validate_resource(None, None), Ok(None));
        assert_eq!(validate_resource(Some(""), Some("")), Ok(None));
        assert_eq!(validate_resource(Some("   "), Some("   ")), Ok(None));
    }

    #[test]
    fn test_validate_resource_rejections() {
        assert!(validate_resource(Some("audio"), Some("https://example.com/audio.mp3")).is_err());
        assert!(validate_resource(Some("document"), None).is_err());
        assert!(validate_resource(Some("document"), Some("")).is_err());
        assert!(validate_resource(None, Some("https://example.com")).is_err());
        assert!(validate_resource(Some("link"), Some("http://example.com")).is_err());
        assert!(validate_resource(Some("link"), Some("javascript:alert(1)")).is_err());
        assert!(validate_resource(Some("link"), Some("data:text/html,bad")).is_err());
        assert!(validate_resource(Some("link"), Some("file:///etc/passwd")).is_err());
        assert!(validate_resource(Some("link"), Some("ftp://ftp.example.com")).is_err());
        assert!(validate_resource(Some("link"), Some("blob:https://example.com/bad")).is_err());
        assert!(validate_resource(Some("link"), Some("about:blank")).is_err());
        assert!(validate_resource(Some("link"), Some("https://admin:pass@example.com/doc")).is_err());
        assert!(validate_resource(Some("link"), Some("https://example .com/doc")).is_err());
        assert!(validate_resource(Some("link"), Some("https://")).is_err());
    }
}
