use time::{Time, Weekday};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum WeekVariant {
    Even,
    Odd,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum WeekdayModifier {
    First,
    Second,
    Third,
    Fourth,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DateSpec {
    pub hours: Vec<Time>,
    pub days: Option<Vec<(Weekday, Option<WeekdayModifier>)>>,
    pub weeks: Option<WeekVariant>,
}
