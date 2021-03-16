use cron_lingo::Timetable;
use std::str::FromStr;

#[test]
fn test_timetable() {
    let expr = "at 17 o'clock on Tuesday and Thursday in odd weeks";
    assert!(Timetable::from_str(expr).is_ok());
}
