use cron_lingo::Schedule;
use std::str::FromStr;

#[test]
fn test_schedule() {
    let expr = "at 17 o'clock on Tuesday and Thursday in odd weeks";
    assert!(Schedule::from_str(expr).is_ok());
}
