//use std::convert::TryInto;
use time::{Time, Weekday};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum WeekVariant {
    Even,
    Odd,
    None,
}
/*
impl WeekVariant {
    fn contains(self, date: Date) -> bool {
        match self {
            Self::Even => date.week() % 2 == 0,
            Self::Odd => date.week() % 2 != 0,
            Self::None => true,
        }
    }
}
*/
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum WeekdayModifier {
    First,
    Second,
    Third,
    Fourth,
    None,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DateSpec {
    pub hours: Vec<Time>,
    pub days: Option<Vec<(Weekday, WeekdayModifier)>>,
    pub weeks: WeekVariant,
}
