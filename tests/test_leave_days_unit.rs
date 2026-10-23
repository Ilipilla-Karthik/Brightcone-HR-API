use brightcone_hr_api::modules::time::leave_calc::{calculate_leave_days_pure, LeaveCalcError};
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};

// Scenario 1: A single working day (e.g. Wednesday Oct 14, 2026)
#[test]
fn test_scenario_1_single_working_day() {
    let d = NaiveDate::from_ymd_opt(2026, 10, 14).unwrap(); // Wednesday
    let res = calculate_leave_days_pure(
        d,
        d,
        false, // working days policy
        true,  // exclude weekends
        true,  // exclude holidays
        false, // full day
        &HashMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        Some("CL".into()),
    )
    .expect("Should calculate single working day");

    assert_eq!(res.total_calendar_days, 1);
    assert_eq!(res.chargeable_days, 1.0);
    assert_eq!(res.working_days, 1);
    assert_eq!(res.weekend_days, 0);
    assert_eq!(res.holiday_days, 0);
    assert_eq!(res.breakdown[0].is_chargeable, true);
    assert_eq!(res.breakdown[0].chargeable_value, 1.0);
}

// Scenario 2: A date range spanning a weekend (Fri Oct 16 to Mon Oct 19, 2026)
#[test]
fn test_scenario_2_range_spanning_weekend() {
    let start = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(); // Friday
    let end = NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(); // Monday
    let res = calculate_leave_days_pure(
        start,
        end,
        false,
        true,
        true,
        false,
        &HashMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        Some("CL".into()),
    )
    .expect("Should calculate range spanning weekend");

    assert_eq!(res.total_calendar_days, 4);
    assert_eq!(res.weekend_days, 2); // Sat Oct 17, Sun Oct 18
    assert_eq!(res.chargeable_days, 2.0); // Fri Oct 16 and Mon Oct 19
    assert_eq!(res.working_days, 2);
}

// Scenario 3: A range containing a configured public holiday (Mon Jan 26, 2026 Republic Day)
#[test]
fn test_scenario_3_range_with_configured_public_holiday() {
    let start = NaiveDate::from_ymd_opt(2026, 1, 23).unwrap(); // Friday
    let end = NaiveDate::from_ymd_opt(2026, 1, 27).unwrap(); // Tuesday
    let mut holidays = HashMap::new();
    holidays.insert(
        NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
        "Republic Day".to_string(),
    );

    let res = calculate_leave_days_pure(
        start,
        end,
        false,
        true,
        true,
        false,
        &holidays,
        &HashSet::new(),
        &HashSet::new(),
        Some("CL".into()),
    )
    .expect("Should calculate range with holiday");

    assert_eq!(res.total_calendar_days, 5);
    assert_eq!(res.weekend_days, 2); // Sat Jan 24, Sun Jan 25
    assert_eq!(res.holiday_days, 1); // Mon Jan 26 Republic Day
    assert_eq!(res.chargeable_days, 2.0); // Fri Jan 23 and Tue Jan 27
    assert_eq!(res.working_days, 2);
}

// Scenario 4: Start date equal to end date
#[test]
fn test_scenario_4_start_date_equal_to_end_date() {
    let d = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(); // Thursday
    let res = calculate_leave_days_pure(
        d,
        d,
        false,
        true,
        true,
        false,
        &HashMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        None,
    )
    .expect("Should calculate same start and end date");

    assert_eq!(res.total_calendar_days, 1);
    assert_eq!(res.chargeable_days, 1.0);
}

// Scenario 5: End date earlier than start date
#[test]
fn test_scenario_5_end_date_earlier_than_start_date() {
    let start = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
    let err = calculate_leave_days_pure(
        start,
        end,
        false,
        true,
        true,
        false,
        &HashMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        None,
    )
    .expect_err("Should reject end date earlier than start date");

    match err {
        LeaveCalcError::InvalidDateRange { start: s, end: e } => {
            assert_eq!(s, "2026-10-20");
            assert_eq!(e, "2026-10-18");
        }
        _ => panic!("Expected InvalidDateRange variant"),
    }
}

// Scenario 6: Calendar-day leave (e.g. Maternity Leave ML or count_calendar_days)
#[test]
fn test_scenario_6_calendar_day_leave() {
    let start = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(); // Friday
    let end = NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(); // Monday
    let mut holidays = HashMap::new();
    holidays.insert(
        NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(),
        "Regional Holiday".to_string(),
    );

    let res = calculate_leave_days_pure(
        start,
        end,
        true,  // is_calendar_days = true
        false, // do not exclude weekends
        false, // do not exclude holidays
        false,
        &holidays,
        &HashSet::new(),
        &HashSet::new(),
        Some("ML".into()),
    )
    .expect("Should calculate calendar days");

    assert_eq!(res.total_calendar_days, 4);
    assert_eq!(res.chargeable_days, 4.0); // All 4 days are chargeable
    assert_eq!(res.policy_applied, "CALENDAR_DAYS");
    assert_eq!(res.leave_type_code, Some("ML".into()));
}

// Scenario 7: A range containing both weekends and configured holidays
#[test]
fn test_scenario_7_range_with_both_weekends_and_configured_holidays() {
    let start = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(); // Thursday
    let end = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(); // Tuesday
    let mut holidays = HashMap::new();
    holidays.insert(
        NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(),
        "Festival of Lights".to_string(),
    );

    let res = calculate_leave_days_pure(
        start,
        end,
        false,
        true,
        true,
        false,
        &holidays,
        &HashSet::new(),
        &HashSet::new(),
        Some("EL".into()),
    )
    .expect("Should calculate both weekends and holidays");

    assert_eq!(res.total_calendar_days, 6);
    assert_eq!(res.weekend_days, 2); // Sat Oct 17, Sun Oct 18
    assert_eq!(res.holiday_days, 1); // Mon Oct 19
    assert_eq!(res.chargeable_days, 3.0); // Thu Oct 15, Fri Oct 16, Tue Oct 20
    assert_eq!(res.working_days, 3);
}

// Scenario 8: A range with zero chargeable working days (only weekend)
#[test]
fn test_scenario_8_zero_chargeable_working_days() {
    let start = NaiveDate::from_ymd_opt(2026, 10, 17).unwrap(); // Saturday
    let end = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap(); // Sunday
    let res = calculate_leave_days_pure(
        start,
        end,
        false,
        true,
        true,
        false,
        &HashMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        Some("CL".into()),
    )
    .expect("Should calculate zero chargeable working days");

    assert_eq!(res.total_calendar_days, 2);
    assert_eq!(res.weekend_days, 2);
    assert_eq!(res.chargeable_days, 0.0);
    assert_eq!(res.working_days, 0);
}

// Scenario 9: Additional cases: Roster schedule overrides, half-day, holiday on weekend
#[test]
fn test_scenario_9_additional_roster_overrides_and_half_day() {
    // 9a: Holiday falling on a Sunday (must not double-subtract)
    let start = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(); // Sunday (Karnataka Rajyotsava)
    let end = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(); // Monday
    let mut holidays = HashMap::new();
    holidays.insert(
        NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
        "Karnataka Rajyotsava".to_string(),
    );

    let res = calculate_leave_days_pure(
        start,
        end,
        false,
        true,
        true,
        false,
        &holidays,
        &HashSet::new(),
        &HashSet::new(),
        Some("CL".into()),
    )
    .unwrap();

    assert_eq!(res.total_calendar_days, 2);
    assert_eq!(res.weekend_days, 1); // Sunday
    assert_eq!(res.holiday_days, 1); // Sunday
    assert_eq!(res.chargeable_days, 1.0); // Monday only

    // 9b: Custom Roster: Employee is scheduled to work on Saturday, scheduled off on Wednesday
    let start2 = NaiveDate::from_ymd_opt(2026, 10, 14).unwrap(); // Wed
    let end2 = NaiveDate::from_ymd_opt(2026, 10, 17).unwrap(); // Sat

    let mut scheduled_offs = HashSet::new();
    scheduled_offs.insert(NaiveDate::from_ymd_opt(2026, 10, 14).unwrap()); // Scheduled off Wed

    let mut scheduled_working = HashSet::new();
    scheduled_working.insert(NaiveDate::from_ymd_opt(2026, 10, 17).unwrap()); // Scheduled to work Sat

    let res2 = calculate_leave_days_pure(
        start2,
        end2,
        false,
        true,
        true,
        false,
        &HashMap::new(),
        &scheduled_offs,
        &scheduled_working,
        Some("CL".into()),
    )
    .unwrap();

    // Thu Oct 15: working
    // Fri Oct 16: working
    // Sat Oct 17: working (scheduled working overrides weekend)
    // Wed Oct 14: scheduled off
    assert_eq!(res2.chargeable_days, 3.0);

    // 9c: Half day on a single working day
    let thur = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
    let half_res = calculate_leave_days_pure(
        thur,
        thur,
        false,
        true,
        true,
        true, // is_half_day = true
        &HashMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        Some("CL".into()),
    )
    .unwrap();
    assert_eq!(half_res.chargeable_days, 0.5);
}
