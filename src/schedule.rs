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

        let tt = Schedule {
            base: OffsetDateTime::try_now_local().unwrap(),
            hours: parse_hours(expression)?,
            weekdays: parse_weekdays(expression)?,
            weeks: parse_weeks(expression)?,
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
    pub fn skip_outdated(&mut self, skip: bool) {
        self.skip_outdated = skip;
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<time::Weekday> for Weekday {
    fn from(weekday: time::Weekday) -> Self {
        match weekday {
            time::Weekday::Monday => Weekday::Monday,
            time::Weekday::Tuesday => Weekday::Tuesday,
            time::Weekday::Wednesday => Weekday::Wednesday,
            time::Weekday::Thursday => Weekday::Thursday,
            time::Weekday::Friday => Weekday::Friday,
            time::Weekday::Saturday => Weekday::Saturday,
            time::Weekday::Sunday => Weekday::Sunday,
        }
    }
}

impl From<Weekday> for u8 {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Monday => 0,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 2,
            Weekday::Thursday => 3,
            Weekday::Friday => 4,
            Weekday::Saturday => 5,
            Weekday::Sunday => 6,
        }
    }
}

impl From<u8> for Weekday {
    fn from(num: u8) -> Self {
        match num {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            6 => Weekday::Sunday,
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

// Parse the hour spec of an expression and return a sorted list.
// Determine the start end end bounds of the relevant part, parse
// each comma-separated value and add them to a vector.
fn parse_hours(expression: &str) -> Result<Vec<u8>, InvalidExpressionError> {
    let start = match expression.find("at") {
        Some(start_idx) => start_idx,
        None => return Err(InvalidExpressionError::InvalidHourSpec),
    };

    let mut section = if let Some(end_idx) = expression.find("on ") {
        expression[start + 2..end_idx].trim()
    } else if let Some(end_idx) = expression.find("in") {
        expression[start + 2..end_idx].trim()
    } else {
        expression[start + 2..].trim()
    };

    if section == "every hour" {
        return Ok((0..24).collect());
    }

    section = match section.strip_suffix("o'clock") {
        Some(stripped) => stripped,
        None => return Err(InvalidExpressionError::InvalidHourSpec),
    };

    let section = section.replace("and", ",");

    let mut hours = Vec::new();

    for i in section.split(',') {
        let item = i.trim().to_string();

        match item.parse::<u8>() {
            Ok(num) => {
                if hours.contains(&num) {
                    return Err(InvalidExpressionError::DuplicateInput);
                } else if !(0..=23).contains(&num) {
                    return Err(InvalidExpressionError::HoursOutOfBounds(
                        HoursOutOfBoundsError { input: num },
                    ));
                } else {
                    hours.push(num);
                }
            }
            Err(_) => return Err(InvalidExpressionError::ParseHour),
        }
    }

    hours.sort_unstable();

    Ok(hours)
}

// Parse the weekday spec of an epression and return a sorted list.
// Determine the start and end bounds of the relevant part, parse
// each comma-separated value, map it to a corresponding integer
// and add it to a vector.
fn parse_weekdays(expression: &str) -> Result<Option<Vec<Weekday>>, InvalidExpressionError> {
    let start = match expression.find("on ") {
        Some(start_idx) => start_idx,
        None => return Ok(None),
    };

    let section = match expression.find("in") {
        Some(end_idx) => expression[start + 2..end_idx].trim(),
        None => expression[start + 2..].trim(),
    };

    let mut weekdays = Vec::new();

    for item in section.replace("and", ",").split(',') {
        let weekday = match item.trim() {
            "Monday" => Weekday::Monday,
            "Tuesday" => Weekday::Tuesday,
            "Wednesday" => Weekday::Wednesday,
            "Thursday" => Weekday::Thursday,
            "Friday" => Weekday::Friday,
            "Saturday" => Weekday::Saturday,
            "Sunday" => Weekday::Sunday,
            _ => return Err(InvalidExpressionError::UnknownWeekday),
        };

        if !weekdays.contains(&weekday) {
            weekdays.push(weekday);
        } else {
            return Err(InvalidExpressionError::DuplicateInput);
        }
    }

    weekdays.sort_unstable();

    Ok(Some(weekdays))
}

// Parse the week spec of an expression and return a WeekVariant.
// After determining the start and end bounds, the value in between
// is attempted to be matched. If the value is supported, it is mapped
// to a WeekVariant.
fn parse_weeks(expression: &str) -> Result<Option<WeekVariant>, InvalidExpressionError> {
    match expression.find("weeks") {
        Some(end_idx) => {
            let start_idx = match expression.find("in") {
                Some(start_idx) => start_idx,
                None => return Err(InvalidExpressionError::InvalidWeekSpec),
            };

            let section = expression[start_idx + 2..end_idx].trim();

            match section {
                "even" => Ok(Some(WeekVariant::Even)),
                "odd" => Ok(Some(WeekVariant::Odd)),
                _ => Err(InvalidExpressionError::InvalidWeekSpec),
            }
        }
        None => match expression.find("week of the month") {
            Some(end_idx) => {
                let start_idx = match expression.find("in the") {
                    Some(start_idx) => start_idx,
                    None => return Err(InvalidExpressionError::InvalidWeekSpec),
                };

                let section = expression[start_idx + 6..end_idx].trim();

                match section {
                    "first" => Ok(Some(WeekVariant::First)),
                    "second" => Ok(Some(WeekVariant::Second)),
                    "third" => Ok(Some(WeekVariant::Third)),
                    "fourth" => Ok(Some(WeekVariant::Fourth)),
                    _ => Err(InvalidExpressionError::InvalidWeekSpec),
                }
            }
            None => Ok(None),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_expression() {
        let result = Schedule::from_str("").unwrap_err();
        assert_eq!(result, InvalidExpressionError::EmptyExpression);
    }

    #[test]
    fn test_complete_schedule() {
        let expression = "at 6, 8, 7 and 14 o'clock on Monday, Thursday and Saturday in the first week of the month";
        let schedule = Schedule::from_str(expression).unwrap();
        assert_eq!(schedule.hours, vec!(6, 7, 8, 14));
        assert_eq!(
            schedule.weekdays,
            Some(vec!(Weekday::Monday, Weekday::Thursday, Weekday::Saturday))
        );
        assert_eq!(schedule.weeks, Some(WeekVariant::First));
    }

    #[test]
    fn test_schedule_without_week_spec() {
        let expression = "at 6, 15 o'clock on Friday";
        let schedule = Schedule::from_str(expression).unwrap();
        assert_eq!(schedule.hours, vec!(6, 15));
        assert_eq!(schedule.weekdays, Some(vec!(Weekday::Friday)));
        assert_eq!(schedule.weeks, None);
    }

    #[test]
    fn test_schedule_hours_only() {
        let expression = "at 6, 23 o'clock";
        let schedule = Schedule::from_str(expression).unwrap();
        assert_eq!(schedule.hours, vec!(6, 23));
        assert_eq!(schedule.weekdays, None);
        assert_eq!(schedule.weeks, None);
    }

    #[test]
    fn test_schedule_every_hour() {
        let expression = "at every hour";
        let schedule = Schedule::from_str(expression).unwrap();
        assert_eq!(schedule.hours, (0..=23).collect::<Vec<u8>>());
        assert_eq!(schedule.weekdays, None);
        assert_eq!(schedule.weeks, None);
    }

    #[test]
    fn test_schedule_without_weekday_spec() {
        let expression = "at 6, 23 o'clock in odd weeks";
        let schedule = Schedule::from_str(expression).unwrap();
        assert_eq!(schedule.hours, vec!(6, 23));
        assert_eq!(schedule.weekdays, None);
        assert_eq!(schedule.weeks, Some(WeekVariant::Odd));
    }

    #[test]
    fn test_schedule_with_specific_weeks() {
        let expression = "at 6, 23 o'clock in the fourth  week of the month";
        let schedule = Schedule::from_str(expression).unwrap();
        assert_eq!(schedule.hours, vec!(6, 23));
        assert_eq!(schedule.weekdays, None);
        assert_eq!(schedule.weeks, Some(WeekVariant::Fourth));
    }

    #[test]
    fn test_parse_hours_for_out_of_bounds_error() {
        let expression = "at 6, 15, 24 o'clock on Friday";
        assert_eq!(
            parse_hours(expression).unwrap_err(),
            InvalidExpressionError::HoursOutOfBounds(HoursOutOfBoundsError { input: 24 })
        );
    }

    #[test]
    fn test_parse_hours_for_hour_parsing_error() {
        let expression = "at 6, 15, 17 18 o'clock on Monday";
        assert_eq!(
            parse_hours(expression).unwrap_err(),
            InvalidExpressionError::ParseHour
        );
    }

    #[test]
    fn test_parse_weekdays_for_unknown_weekday_error() {
        let expression = "at 6 o'clock on Sunday and Thursday and Fuu in odd weeks";
        assert_eq!(
            parse_weekdays(expression).unwrap_err(),
            InvalidExpressionError::UnknownWeekday
        );
    }

    #[test]
    fn test_parse_weekdays_for_invalid_weekspec_error() {
        let expression = "at 6 o'clock on Sunday and Thursday in the thrd week of the month";
        assert_eq!(
            parse_weeks(expression).unwrap_err(),
            InvalidExpressionError::InvalidWeekSpec
        );
    }

    #[test]
    fn test_parse_weekdays_for_duplicate_error() {
        let expression = "at 13 o'clock on Monday and Monday and Friday";
        assert_eq!(
            parse_weekdays(expression).unwrap_err(),
            InvalidExpressionError::DuplicateInput
        );
    }

    #[test]
    fn test_schedule_iteration_full_spec_even1() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 07 - 28), time!(15:00:00)).assume_utc(),
            hours: vec![6, 18],
            weekdays: Some(vec![Weekday::Monday, Weekday::Wednesday]),
            weeks: Some(WeekVariant::Even),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 07 - 28), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 09), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 09), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 11), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 11), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(5).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_full_spec_even2() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 08 - 10), time!(08:24:47)).assume_utc(),
            hours: vec![12],
            weekdays: Some(vec![Weekday::Friday, Weekday::Sunday]),
            weeks: Some(WeekVariant::Even),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 08 - 13), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 15), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 27), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 29), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 09 - 10), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 09 - 12), time!(12:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(6).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_no_weekdays() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 06 - 15), time!(08:24:47)).assume_utc(),
            hours: vec![6, 12],
            weekdays: None,
            weeks: Some(WeekVariant::Even),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 15), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 16), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 16), time!(12:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(3).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_first_week_no_weekdays() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 07 - 03), time!(08:24:47)).assume_utc(),
            hours: vec![6, 12],
            weekdays: None,
            weeks: Some(WeekVariant::First),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 07 - 03), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 04), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 04), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 05), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 05), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 06), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 06), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 07), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 07), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 01), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 01), time!(12:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(11).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_first_week_only1() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 07 - 03), time!(08:24:47)).assume_utc(),
            hours: vec![6, 12],
            weekdays: Some(vec![Weekday::Wednesday, Weekday::Sunday]),
            weeks: Some(WeekVariant::First),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 07 - 04), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 04), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 07), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 07), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 01), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 01), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 04), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 04), time!(12:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(8).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_first_week_only2() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 09 - 14), time!(09:00:00)).assume_utc(),
            hours: vec![6, 12],
            weekdays: Some(vec![Weekday::Monday, Weekday::Friday]),
            weeks: Some(WeekVariant::First),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 10 - 01), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 10 - 01), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 10 - 04), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 10 - 04), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 11 - 01), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 11 - 01), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 11 - 05), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 11 - 05), time!(12:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(8).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_second_week_only1() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 06 - 27), time!(09:00:00)).assume_utc(),
            hours: vec![9, 23],
            weekdays: Some(vec![Weekday::Monday, Weekday::Wednesday]),
            weeks: Some(WeekVariant::Second),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 07 - 12), time!(09:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 12), time!(23:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 14), time!(09:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 14), time!(23:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 09), time!(09:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 09), time!(23:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 11), time!(09:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 11), time!(23:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(8).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_second_week_only2() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 06 - 01), time!(09:00:00)).assume_utc(),
            hours: vec![9, 23],
            weekdays: Some(vec![Weekday::Monday, Weekday::Wednesday]),
            weeks: Some(WeekVariant::Second),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 09), time!(09:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 09), time!(23:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 14), time!(09:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 14), time!(23:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 12), time!(09:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 12), time!(23:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 14), time!(09:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 14), time!(23:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(8).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_third_week_only1() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 06 - 01), time!(09:00:00)).assume_utc(),
            hours: vec![10],
            weekdays: Some(vec![Weekday::Tuesday]),
            weeks: Some(WeekVariant::Third),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 15), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 20), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 17), time!(10:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(3).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_third_week_only2() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 06 - 25), time!(09:00:00)).assume_utc(),
            hours: vec![10],
            weekdays: Some(vec![Weekday::Tuesday]),
            weeks: Some(WeekVariant::Third),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 07 - 20), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 08 - 17), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 09 - 21), time!(10:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(3).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_fourth_week_only1() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 07 - 28), time!(09:00:00)).assume_utc(),
            hours: vec![10],
            weekdays: Some(vec![Weekday::Thursday]),
            weeks: Some(WeekVariant::Fourth),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 08 - 26), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 09 - 23), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 10 - 28), time!(10:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(3).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_fourth_week_only2() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 05 - 28), time!(09:00:00)).assume_utc(),
            hours: vec![10],
            weekdays: Some(vec![Weekday::Thursday]),
            weeks: Some(WeekVariant::Fourth),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 24), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 22), time!(10:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(2).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_fourth_week_only_no_weekdays() {
        use time::{date, time};
        let schedule = Schedule {
            base: PrimitiveDateTime::new(date!(2021 - 05 - 28), time!(09:00:00)).assume_utc(),
            hours: vec![10],
            weekdays: None,
            weeks: Some(WeekVariant::Fourth),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 05 - 28), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 22), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 23), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 24), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 25), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 26), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 27), time!(10:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 28), time!(10:00:00)).assume_utc(),
        ];
        assert_eq!(
            schedule.iter().take(8).collect::<Vec<OffsetDateTime>>(),
            result
        );
    }
}
