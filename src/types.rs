use std::convert::TryInto;
use time::{Date, Time};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum WeekVariant {
    Even,
    Odd,
    First,
    Second,
    Third,
    Fourth,
    None,
}

impl WeekVariant {
    fn contains(self, date: Date) -> bool {
        match self {
            Self::Even => date.week() % 2 == 0,
            Self::Odd => date.week() % 2 != 0,
            Self::First => {
                let first_day = Date::try_from_ymd(date.year(), date.month(), 1).unwrap();
                (date - first_day).whole_days() < 7
            }
            Self::Second => {
                let first_day = Date::try_from_ymd(date.year(), date.month(), 1).unwrap();
                let delta = (date - first_day).whole_days();
                (7..14).contains(&delta)
            }
            Self::Third => {
                let first_day = Date::try_from_ymd(date.year(), date.month(), 1).unwrap();
                let delta = (date - first_day).whole_days();
                (14..21).contains(&delta)
            }
            Self::Fourth => {
                let first_day = Date::try_from_ymd(date.year(), date.month(), 1).unwrap();
                let delta = (date - first_day).whole_days();
                (21..28).contains(&delta)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum WeekdayModifier {
    First,
    Second,
    Third,
    Fourth,
    None,
}

/*
impl WeekVariant {
    fn contains(self, date: Date) -> bool {
        match self {
            Self::First => {
                let first_day = Date::try_from_ymd(date.year(), date.month(), 1).unwrap();
                (date - first_day).whole_days() < 7
            }
            Self::Second => {
                let first_day = Date::try_from_ymd(date.year(), date.month(), 1).unwrap();
                let delta = (date - first_day).whole_days();
                (7..14).contains(&delta)
            }
            Self::Third => {
                let first_day = Date::try_from_ymd(date.year(), date.month(), 1).unwrap();
                let delta = (date - first_day).whole_days();
                (14..21).contains(&delta)
            }
            Self::Fourth => {
                let first_day = Date::try_from_ymd(date.year(), date.month(), 1).unwrap();
                let delta = (date - first_day).whole_days();
                (21..28).contains(&delta)
            }
        }
    }
}
*/

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DateSpec {
    pub hours: Vec<Time>,
    pub days: Option<Vec<Weekday>>,
    pub weeks: WeekVariant,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Weekday {
    Monday(WeekdayModifier),
    Tuesday(WeekdayModifier),
    Wednesday(WeekdayModifier),
    Thursday(WeekdayModifier),
    Friday(WeekdayModifier),
    Saturday(WeekdayModifier),
    Sunday(WeekdayModifier),
}

impl From<time::Weekday> for Weekday {
    fn from(weekday: time::Weekday) -> Self {
        match weekday {
            time::Weekday::Monday => Weekday::Monday(WeekdayModifier::None),
            time::Weekday::Tuesday => Weekday::Tuesday(WeekdayModifier::None),
            time::Weekday::Wednesday => Weekday::Wednesday(WeekdayModifier::None),
            time::Weekday::Thursday => Weekday::Thursday(WeekdayModifier::None),
            time::Weekday::Friday => Weekday::Friday(WeekdayModifier::None),
            time::Weekday::Saturday => Weekday::Saturday(WeekdayModifier::None),
            time::Weekday::Sunday => Weekday::Sunday(WeekdayModifier::None),
        }
    }
}

impl From<Weekday> for u8 {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Monday(_) => 0,
            Weekday::Tuesday(_) => 1,
            Weekday::Wednesday(_) => 2,
            Weekday::Thursday(_) => 3,
            Weekday::Friday(_) => 4,
            Weekday::Saturday(_) => 5,
            Weekday::Sunday(_) => 6,
        }
    }
}

impl From<u8> for Weekday {
    fn from(num: u8) -> Self {
        match num {
            0 => Weekday::Monday(WeekdayModifier::None),
            1 => Weekday::Tuesday(WeekdayModifier::None),
            2 => Weekday::Wednesday(WeekdayModifier::None),
            3 => Weekday::Thursday(WeekdayModifier::None),
            4 => Weekday::Friday(WeekdayModifier::None),
            5 => Weekday::Saturday(WeekdayModifier::None),
            6 => Weekday::Sunday(WeekdayModifier::None),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Sub for Weekday {
    type Output = i8;

    fn sub(self, other: Self) -> i8 {
        (u8::from(self) - u8::from(other)).try_into().unwrap()
    }
}

impl std::ops::Add for Weekday {
    type Output = u8;

    fn add(self, other: Self) -> u8 {
        u8::from(self) + u8::from(other)
    }
}
