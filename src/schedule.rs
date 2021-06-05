use crate::error::*;
use std::convert::TryInto;
use std::iter::Iterator;
use std::str::FromStr;
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time};

/// A schedule that is built from an expression and can be iterated
/// in order to compute the next date(s) that match the specification. The
/// computation is always based on the current system time.
/// This is the only way (at this point) to use `Schedule` in a meaningful
/// way.
///
/// The expression must adhere to a specific syntax. See the module-level
/// documentation for the full range of possibilities.
#[derive(Debug, PartialEq, Clone)]
pub struct Schedule {
    base: OffsetDateTime,
    hours: Vec<u8>,
    weekdays: Option<Vec<Weekday>>,
    weeks: Option<WeekVariant>,
}

impl Schedule {
    #[allow(dead_code)]
    pub fn iter(&self) -> ScheduleIter {
        ScheduleIter {
            schedule: self.clone(),
            current: self.base.clone(),
            skip_outdated: true,
        }
    }
}

impl FromStr for Schedule {
    type Err = InvalidExpressionError;

    /// Attempt to create a new `Schedule` object from an expression.
    ///
    /// ```rust
    /// use cron_lingo::Schedule;
    /// use std::str::FromStr;
    ///
    /// let expr = "at 6 and 18 o'clock on Monday and Thursday in even weeks";
    /// assert!(Schedule::from_str(expr).is_ok());
    /// ```
    fn from_str(expression: &str) -> Result<Self, Self::Err> {
        if expression.is_empty() {
            return Err(InvalidExpressionError::EmptyExpression.into());
        }

        let blocks: Vec<&str> = split_expression(expression);

        //let specifications: Vec<DateSpec> = blocks.iter().map(|x| parse_block(x)).collect();

        let tt = Schedule {
            base: OffsetDateTime::try_now_local().unwrap(),
            hours: vec![],
            weekdays: None,
            weeks: None,
        };
        Ok(tt)
    }
}

/// A wrapper around `Schedule` that keeps track of state during iteration.
#[derive(Clone)]
pub struct ScheduleIter {
    schedule: Schedule,
    current: OffsetDateTime,
    skip_outdated: bool,
}

impl ScheduleIter {
    /// By default the `next` method will not return a date that is
    /// in the past but compute the next future date based on the
    /// current local time instead. This method allows to change the
    /// iterators default behaviour.
    pub fn skip_outdated(mut self, skip: bool) -> ScheduleIter {
        self.skip_outdated = skip;
        self
    }
}

impl Iterator for ScheduleIter {
    type Item = OffsetDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        if self.skip_outdated {
            let now = OffsetDateTime::try_now_local().unwrap();

            if now > self.current {
                self.current = now;
            }
        }

        let (mut next_date, next_time) = match &self.schedule.weekdays {
            Some(weekdays) => {
                let this_weekday = self.current.weekday().number_days_from_monday().into();

                let (next_hour, next_weekday) = if weekdays.iter().any(|&x| x == this_weekday) {
                    match self
                        .schedule
                        .hours
                        .iter()
                        .find(|&&x| x > self.current.hour())
                    {
                        Some(n) => (*n, this_weekday),
                        None => match weekdays.iter().find(|&&x| x > this_weekday) {
                            Some(wd) => (self.schedule.hours[0], *wd),
                            None => (self.schedule.hours[0], weekdays[0]),
                        },
                    }
                } else {
                    match weekdays.iter().find(|&&x| x > this_weekday) {
                        Some(wd) => (self.schedule.hours[0], *wd),
                        None => (self.schedule.hours[0], weekdays[0]),
                    }
                };

                let next_time = Time::try_from_hms(next_hour, 0, 0).unwrap();

                let day_addend = {
                    if this_weekday > next_weekday {
                        7 - (this_weekday - next_weekday)
                    } else if this_weekday == next_weekday
                        && self.current.hour() >= next_time.hour()
                    {
                        7
                    } else {
                        next_weekday - this_weekday
                    }
                };

                let next_date = self.current.date() + Duration::days(day_addend.into());

                (next_date, next_time)
            }
            None => match self
                .schedule
                .hours
                .iter()
                .find(|&&x| x > self.current.hour())
            {
                Some(h) => {
                    let next_time = Time::try_from_hms(*h, 0, 0).unwrap();
                    (self.current.date(), next_time)
                }
                None => {
                    let next_time = Time::try_from_hms(self.schedule.hours[0], 0, 0).unwrap();
                    let next_date = self.current.date() + Duration::day();
                    (next_date, next_time)
                }
            },
        };

        if let Some(week) = &self.schedule.weeks {
            match week {
                WeekVariant::Even | WeekVariant::Odd => {
                    if !week.contains(next_date) {
                        next_date += Duration::week();
                    }
                }
                WeekVariant::First => {
                    if !week.contains(next_date) {
                        let base = get_first_of_next_month(next_date);
                        next_date = compute_next_date(base, &self.schedule.weekdays);
                    }
                }
                WeekVariant::Second => {
                    if !week.contains(next_date) {
                        let end_of_first =
                            Date::try_from_ymd(next_date.year(), next_date.month(), 7).unwrap();

                        if end_of_first < next_date {
                            let base = get_first_of_next_month(next_date) + Duration::days(7);
                            next_date = compute_next_date(base, &self.schedule.weekdays);
                        } else {
                            let base = Date::try_from_ymd(next_date.year(), next_date.month(), 1)
                                .unwrap()
                                + Duration::days(7);
                            next_date = compute_next_date(base, &self.schedule.weekdays);
                        }
                    }
                }
                WeekVariant::Third => {
                    if !week.contains(next_date) {
                        let end_of_second =
                            Date::try_from_ymd(next_date.year(), next_date.month(), 14).unwrap();

                        if end_of_second < next_date {
                            let base = get_first_of_next_month(next_date) + Duration::days(14);
                            next_date = compute_next_date(base, &self.schedule.weekdays);
                        } else {
                            let base = Date::try_from_ymd(next_date.year(), next_date.month(), 1)
                                .unwrap()
                                + Duration::days(14);
                            next_date = compute_next_date(base, &self.schedule.weekdays);
                        }
                    }
                }
                WeekVariant::Fourth => {
                    if !week.contains(next_date) {
                        let end_of_third =
                            Date::try_from_ymd(next_date.year(), next_date.month(), 21).unwrap();

                        if end_of_third < next_date {
                            let base = get_first_of_next_month(next_date) + Duration::days(21);
                            next_date = compute_next_date(base, &self.schedule.weekdays);
                        } else {
                            let base = Date::try_from_ymd(next_date.year(), next_date.month(), 1)
                                .unwrap()
                                + Duration::days(21);
                            next_date = compute_next_date(base, &self.schedule.weekdays);
                        }
                    }
                }
            }
        }

        let next_date_time =
            PrimitiveDateTime::new(next_date, next_time).assume_offset(self.current.offset());

        self.current = next_date_time;

        Some(next_date_time)
    }
}

fn compute_next_date(base: Date, weekdays: &Option<Vec<Weekday>>) -> Date {
    let base_weekday: Weekday = base.weekday().into();

    match weekdays {
        Some(weekdays) => match weekdays.iter().find(|&&wd| wd >= base_weekday) {
            Some(wd) => {
                let delta = *wd - base_weekday;
                base + Duration::days(delta.into())
            }
            None => {
                let delta = 7 - (base_weekday - weekdays[0]);
                base + Duration::days(delta.into())
            }
        },
        None => base,
    }
}

fn get_first_of_next_month(date: Date) -> Date {
    match Date::try_from_ymd(date.year(), date.month() + 1, 1) {
        Ok(d) => d,
        Err(_) => Date::try_from_ymd(date.year() + 1, 1, 1).unwrap(),
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum WeekVariant {
    Even,
    Odd,
    First,
    Second,
    Third,
    Fourth,
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
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
enum WeekdayModifier {
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

struct DateSpec {
    hours: Vec<Time>,
    days: Option<Vec<Weekday>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Weekday {
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

// Split an expression into multiple distinct blocks that may be separated
// by either ".. and at .." or ".., at ..".
fn split_expression(expression: &str) -> Vec<&str> {
    expression
        .split("and at")
        .map(|x| x.split(", at").collect::<Vec<&str>>())
        .flatten()
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
}

// Parse a block (e.g. "at 4 AM and 4 PM on Monday and Thursday") to a DateSpec
// object.
fn parse_block(block: &str) -> Result<DateSpec, InvalidExpressionError> {
    // First check for the existence of a pattern that separates
    // weekdays from time specifications.
    let (time_block, day_block) = split_block(block)?;
    let hours = parse_times(time_block)?;

    let days = if let Some(d) = day_block {
        Some(parse_days(d)?)
    } else {
        None
    };

    Ok(DateSpec { hours, days })
}

// Split a block into two parts (time spec and day spec, if any).
fn split_block(block: &str) -> Result<(&str, Option<&str>), InvalidExpressionError> {
    match block.find("on ") {
        Some(idx) => {
            let (mut times, mut days) = block.split_at(idx);

            times = times.trim_start_matches("at").trim();

            days = days.trim_start_matches("on").trim();

            // The day specification must be separated from the
            // time spec by an "on", but the weekdays in the day
            // spec itself are only separated by commas/"and"s,
            // so multiple "on"s are invalid.
            if days.contains("on ") {
                return Err(InvalidExpressionError::Syntax);
            }

            Ok((times, Some(days)))
        }
        None => match block.find('(') {
            Some(start_idx) => match block.find(')') {
                Some(end_idx) => {
                    let times = block[..start_idx].trim();
                    let days = block[start_idx + 1..end_idx].trim();

                    Ok((times, Some(days)))
                }
                None => return Err(InvalidExpressionError::Syntax),
            },
            None => Ok((block, None)),
        },
    }
}

// Parse the hour spec of an expression and return a sorted list.
fn parse_times(expression: &str) -> Result<Vec<Time>, InvalidExpressionError> {
    if expression == "every full hour" {
        let mut full_hours = vec![];

        for i in 0..24 {
            full_hours.push(Time::try_from_hms(i, 0, 0).unwrap());
        }
        return Ok(full_hours);
    }

    let expression = expression.replace("and", ",");

    let mut times = Vec::new();

    for item in expression.split(',').map(|x| x.trim()) {
        let time = match Time::parse(item, "%-I %P") {
            Ok(t) => t,
            Err(_) => Time::parse(item, "%-I:%-M %P")
                .map_err(|source| InvalidExpressionError::InvalidHourSpec(source))?,
        };

        if times.contains(&time) {
            return Err(InvalidExpressionError::DuplicateInput);
        } else {
            times.push(time);
        }
    }

    times.sort_unstable();

    Ok(times)
}

// Parse the weekday spec of an expression and return a sorted list.
fn parse_days(expression: &str) -> Result<Vec<Weekday>, InvalidExpressionError> {
    let mut days = Vec::new();

    for item in expression.replace("and", ",").split(',') {
        let (item, modifier) = if item.starts_with("the first") {
            (item.trim_start_matches("the first"), WeekdayModifier::First)
        } else if item.starts_with("the second") {
            (
                item.trim_start_matches("the second"),
                WeekdayModifier::Second,
            )
        } else if item.starts_with("the third") {
            (item.trim_start_matches("the third"), WeekdayModifier::Third)
        } else if item.starts_with("the fourth") {
            (
                item.trim_start_matches("the fourth"),
                WeekdayModifier::Fourth,
            )
        } else {
            (item, WeekdayModifier::None)
        };

        let day = match item.trim() {
            "Monday" => Weekday::Monday(modifier),
            "Tuesday" => Weekday::Tuesday(modifier),
            "Wednesday" => Weekday::Wednesday(modifier),
            "Thursday" => Weekday::Thursday(modifier),
            "Friday" => Weekday::Friday(modifier),
            "Saturday" => Weekday::Saturday(modifier),
            "Sunday" => Weekday::Sunday(modifier),
            _ => return Err(InvalidExpressionError::UnknownWeekday),
        };

        if !days.contains(&day) {
            days.push(day);
        } else {
            return Err(InvalidExpressionError::DuplicateInput);
        }
    }

    days.sort_unstable();

    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::time;

    #[test]
    fn test_empty_expression() {
        let result = Schedule::from_str("").unwrap_err();
        assert_eq!(result, InvalidExpressionError::EmptyExpression);
    }

    #[test]
    fn test_split_expression() {
        let expression = "at 4 PM on Monday, at 6 PM on Thursday and at 3 AM";
        let result = vec!["at 4 PM on Monday", "6 PM on Thursday", "3 AM"];
        assert_eq!(split_expression(expression), result);
    }

    #[test]
    fn test_split_block() {
        let block = "at 4 PM and 6 PM on Monday and Tuesday";
        let result = ("4 PM and 6 PM", Some("Monday and Tuesday"));
        assert_eq!(split_block(block).unwrap(), result);
    }

    #[test]
    fn test_split_block_for_error() {
        let block = "at 4 PM and 6 PM on Monday and on Tuesday";
        assert_eq!(
            split_block(block).unwrap_err(),
            InvalidExpressionError::Syntax
        );
    }

    #[test]
    fn test_parse_times() {
        let expression = "1 AM, 5 AM, 4 PM, 5 PM and 6 PM";
        let result = vec![
            time!(01:00:00),
            time!(05:00:00),
            time!(16:00:00),
            time!(17:00:00),
            time!(18:00:00),
        ];
        assert_eq!(parse_times(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_times_every_hour() {
        let expression = "every full hour";
        let result = vec![
            time!(00:00:00),
            time!(01:00:00),
            time!(02:00:00),
            time!(03:00:00),
            time!(04:00:00),
            time!(05:00:00),
            time!(06:00:00),
            time!(07:00:00),
            time!(08:00:00),
            time!(09:00:00),
            time!(10:00:00),
            time!(11:00:00),
            time!(12:00:00),
            time!(13:00:00),
            time!(14:00:00),
            time!(15:00:00),
            time!(16:00:00),
            time!(17:00:00),
            time!(18:00:00),
            time!(19:00:00),
            time!(20:00:00),
            time!(21:00:00),
            time!(22:00:00),
            time!(23:00:00),
        ];
        assert_eq!(parse_times(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_times_for_parse_error() {
        let expression = "1 AM and 5:30";
        assert_eq!(
            parse_times(expression).unwrap_err(),
            InvalidExpressionError::InvalidHourSpec(time::ParseError::UnexpectedEndOfString)
        );
    }

    #[test]
    fn test_parse_times_for_duplicate_error() {
        let expression = "1 AM and 1 AM and 5 PM";
        assert_eq!(
            parse_times(expression).unwrap_err(),
            InvalidExpressionError::DuplicateInput
        );
    }

    #[test]
    fn test_parse_days() {
        let expression = "Monday, Tuesday and Thursday";
        let result = vec![
            Weekday::Monday(WeekdayModifier::None),
            Weekday::Tuesday(WeekdayModifier::None),
            Weekday::Thursday(WeekdayModifier::None),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_days_for_duplicate_error() {
        let expression = "Monday, Monday and Thursday";
        assert_eq!(
            parse_days(expression).unwrap_err(),
            InvalidExpressionError::DuplicateInput
        );
    }

    #[test]
    fn test_parse_days_with_modifiers() {
        let expression = "the first Monday, Tuesday and the fourth Thursday";
        let result = vec![
            Weekday::Monday(WeekdayModifier::First),
            Weekday::Tuesday(WeekdayModifier::None),
            Weekday::Thursday(WeekdayModifier::Fourth),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }
}
