use cron_lingo::Schedule;
use std::str::FromStr;

#[test]
fn test_schedule_1() {
    let expr = "at 6 AM on Mondays and Thursdays and at 6 PM on Sundays in even weeks";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_2() {
    let expr = "at 1 AM and at 6 PM on Saturdays and Sundays";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_3() {
    let expr = "at 6 AM on Mondays and at 6 PM on Thursdays";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_4() {
    let expr = "at 6 AM, 6 PM (Mondays) and at 8 AM on the first Sunday";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_5() {
    let expr = "at 2 PM (Mondays, Thursdays) in even weeks, at 6 PM on Wednesdays in odd weeks and at 1 AM";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}
