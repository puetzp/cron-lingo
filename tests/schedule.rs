use cron_lingo::Schedule;
use std::str::FromStr;

#[test]
fn test_empty_expression() {
    let result = Schedule::from_str("").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::EmptyExpression);
}

#[test]
fn test_unexpected_end_of_input() {
    let expr = "at 6 AM on Mondays and";
    let result = Schedule::from_str(expr).unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);
}

#[test]
fn test_schedule_1() {
    let expr = "at 6 AM on Mondays and Thursdays plus at 6 PM on Sundays in even weeks";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_2() {
    let expr = "at 1 AM plus at 6 PM on Saturdays and Sundays";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_3() {
    let expr = "at 6 AM on Mondays plus at 6 PM on Thursdays";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_4() {
    let expr = "at 6 AM, 6 PM (Mondays) plus at 8 AM on the first Sunday";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_5() {
    let expr = "at 2 PM (Mondays, Thursdays) in even weeks plus at 6 PM on Wednesdays in odd weeks plus at 1 AM";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_6() {
    let expr = "at 6:30 AM on Mondays plus at 6 PM on Thursdays";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_7() {
    let expr = "at 8:15 AM (Fridays and the first Saturday) in even weeks";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}
