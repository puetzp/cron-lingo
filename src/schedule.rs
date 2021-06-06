use crate::error::*;
use crate::types::*;
use std::iter::Iterator;
use std::str::FromStr;
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time};

/// A schedule that is built from an expression and can be iterated
/// in order to compute the next date(s) that match the specification. By
/// default the computation is based on the current system time, meaning
/// the iterator will never return a date in the past.
///
/// The expression must adhere to a specific syntax. See the module-level
/// documentation for the full range of possibilities.
#[derive(Debug, PartialEq, Clone)]
pub struct Schedule {
    base: OffsetDateTime,
    specs: Vec<DateSpec>,
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

        let mut specs: Vec<DateSpec> = vec![];

        for block in blocks {
            let spec = parse_block(block)?;
            specs.push(spec);
        }

        let tt = Schedule {
            base: OffsetDateTime::try_now_local().unwrap(),
            specs,
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

/*
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
                WeekVariant::None => unreachable!(),
            }
        }

        let next_date_time =
            PrimitiveDateTime::new(next_date, next_time).assume_offset(self.current.offset());

        self.current = next_date_time;

        Some(next_date_time)
    }
}
*/

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
    let (time_block, day_block, week_block) = split_block(block)?;
    let hours = parse_times(time_block)?;

    let days = if let Some(d) = day_block {
        Some(parse_days(d)?)
    } else {
        None
    };

    let weeks = if let Some(w) = week_block {
        match w {
            "in odd weeks" => WeekVariant::Odd,
            "in even weeks" => WeekVariant::Even,
            _ => return Err(InvalidExpressionError::InvalidWeekSpec),
        }
    } else {
        WeekVariant::None
    };

    Ok(DateSpec { hours, days, weeks })
}

// Split a block into two parts (time spec and day spec, if any).
fn split_block(block: &str) -> Result<(&str, Option<&str>, Option<&str>), InvalidExpressionError> {
    let (remainder, weeks) = match block.find("in ") {
        Some(idx) => {
            let (r, w) = block.split_at(idx);
            (r.trim(), Some(w.trim()))
        }
        None => (block, None),
    };

    match remainder.find("on ") {
        Some(idx) => {
            let (mut times, mut days) = remainder.split_at(idx);

            times = times.trim_start_matches("at").trim();

            days = days.trim_start_matches("on").trim();

            // The day specification must be separated from the
            // time spec by an "on", but the weekdays in the day
            // spec itself are only separated by commas/"and"s,
            // so multiple "on"s are invalid.
            if days.contains("on ") {
                return Err(InvalidExpressionError::Syntax);
            }

            Ok((times, Some(days), weeks))
        }
        None => match remainder.find('(') {
            Some(start_idx) => match remainder.find(')') {
                Some(end_idx) => {
                    let times = block[..start_idx].trim_start_matches("at").trim();
                    let days = block[start_idx + 1..end_idx].trim();

                    Ok((times, Some(days), weeks))
                }
                None => return Err(InvalidExpressionError::Syntax),
            },
            None => {
                let times = remainder.trim_start_matches("at").trim();
                Ok((times, None, weeks))
            }
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
        let parts: Vec<&str> = item
            .trim()
            .trim_start_matches("the")
            .trim()
            .split_whitespace()
            .collect();

        let (modifier, day) = if parts.len() == 2 {
            let m = match parts[0] {
                "first" | "1st" => WeekdayModifier::First,
                "second" | "2nd" => WeekdayModifier::Second,
                "third" | "3rd" => WeekdayModifier::Third,
                "fourth" | "4th" => WeekdayModifier::Fourth,
                _ => return Err(InvalidExpressionError::InvalidWeekdayModifier),
            };

            (m, parts[1])
        } else if parts.len() == 1 {
            (WeekdayModifier::None, parts[0])
        } else {
            return Err(InvalidExpressionError::InvalidWeekdaySpec);
        };

        let spec = match day {
            "Monday" => Weekday::Monday(modifier),
            "Tuesday" => Weekday::Tuesday(modifier),
            "Wednesday" => Weekday::Wednesday(modifier),
            "Thursday" => Weekday::Thursday(modifier),
            "Friday" => Weekday::Friday(modifier),
            "Saturday" => Weekday::Saturday(modifier),
            "Sunday" => Weekday::Sunday(modifier),
            _ => return Err(InvalidExpressionError::UnknownWeekday),
        };

        if !days.contains(&spec) {
            days.push(spec);
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
        let result = ("4 PM and 6 PM", Some("Monday and Tuesday"), None);
        assert_eq!(split_block(block).unwrap(), result);
    }

    #[test]
    fn test_split_block_with_week_mod() {
        let block = "at 4 PM and 6 PM on Monday and Tuesday in odd weeks";
        let result = (
            "4 PM and 6 PM",
            Some("Monday and Tuesday"),
            Some("in odd weeks"),
        );
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
        let expression = "the first Monday, Tuesday and the 4th Thursday";
        let result = vec![
            Weekday::Monday(WeekdayModifier::First),
            Weekday::Tuesday(WeekdayModifier::None),
            Weekday::Thursday(WeekdayModifier::Fourth),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_1() {
        let expression = "at 5 PM (Monday and Thursday) in odd weeks";
        let result = DateSpec {
            hours: vec![time!(17:00:00)],
            days: Some(vec![
                Weekday::Monday(WeekdayModifier::None),
                Weekday::Thursday(WeekdayModifier::None),
            ]),
            weeks: WeekVariant::Odd,
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_2() {
        let expression = "at 5 AM  and 6:30 PM (first Monday and Thursday)";
        let result = DateSpec {
            hours: vec![time!(05:00:00), time!(18:30:00)],
            days: Some(vec![
                Weekday::Monday(WeekdayModifier::First),
                Weekday::Thursday(WeekdayModifier::None),
            ]),
            weeks: WeekVariant::None,
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_3() {
        let expression = "at 6:30 AM and 6:30 PM on Monday and Friday in even weeks";
        let result = DateSpec {
            hours: vec![time!(06:30:00), time!(18:30:00)],
            days: Some(vec![
                Weekday::Monday(WeekdayModifier::None),
                Weekday::Friday(WeekdayModifier::None),
            ]),
            weeks: WeekVariant::Even,
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_4() {
        let expression = "at every full hour on Monday";
        assert!(parse_block(expression).is_ok());
    }
    #[test]
    fn test_parse_block_5() {
        let expression = "at 6 PM in even weeks";
        let result = DateSpec {
            hours: vec![time!(18:00:00)],
            days: None,
            weeks: WeekVariant::Even,
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }
}
