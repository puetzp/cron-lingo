use cron_lingo::Timetable;

#[test]
fn test_timetable() {
    let expr = "at 17 o'clock on Tuesday and Thursday in odd weeks";
    assert!(Timetable::new(expr).is_ok());
}
