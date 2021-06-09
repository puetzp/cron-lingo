use cron_lingo::Schedule;
use std::str::FromStr;

#[test]
fn test_schedule() {
    let expr = "at 6 AM on Mondays and Thursdays and at 6 PM on Sundays in even weeks";
    assert!(Schedule::from_str(expr).is_ok());
}
