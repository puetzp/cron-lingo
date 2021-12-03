use cron_lingo::Schedule;
use std::str::FromStr;

#[test]
fn test_empty_expression() {
    let result = Schedule::from_str("").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::EmptyExpression);
}

#[test]
fn test_unexpected_end_of_input() {
    let result = Schedule::from_str("at").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at ").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 08").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 8:").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 8:0").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 8:00 ").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 8:00 AM ").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 8:00 AM on").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 6 AM on Mondays and").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 6 AM on Mondays and Thursdays ").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 6 AM (").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 6 AM (Mondays").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 6 AM (Mondays and").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 6 AM (Mondays and Thursdays").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);

    let result = Schedule::from_str("at 6 AM (Mondays and Thursdays) ").unwrap_err();
    assert_eq!(result, cron_lingo::error::Error::UnexpectedEndOfInput);
}

#[test]
fn test_schedule_1() {
    let expr = "at 6 AM (Mondays and Thursdays)";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_2() {
    let expr = "at 6 PM on Saturdays and Sundays";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_3() {
    let expr = "at 6 AM on the last Monday";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_4() {
    let expr = "at 6 AM, 6 PM (Mondays) in even weeks";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_5() {
    let expr = "at 2 PM (Mondays, Thursdays, Saturdays and last Sunday) in even weeks";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_6() {
    let expr = "at 6:30 AM on the first Monday and 4th Saturday";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_7() {
    let expr = "at 8:15 AM (Fridays and the first Saturday) in even weeks";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_8() {
    let expr = "at 6 PM on Sundays in even weeks";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_9() {
    let expr = "at 6 PM on the 1st Saturday and Sundays";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_10() {
    let expr = "at 6 PM on Thursdays in odd weeks";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_11() {
    let expr = "at 8 AM and 8 PM on the first Sunday";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_12() {
    let expr = "at 1 AM, 5:30 AM, 12 PM, 5:30 PM and 11:59 PM";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_schedule_13() {
    let expr = "at 6:30 PM on Thursdays";
    let result = Schedule::from_str(expr);
    assert!(result.is_ok(), "{:?}", result);
}
