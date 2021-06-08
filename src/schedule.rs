use crate::error::*;
use crate::types::*;
use std::iter::Iterator;
use std::str::FromStr;
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time, Weekday};

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

impl Iterator for ScheduleIter {
    type Item = OffsetDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        if self.skip_outdated {
            let now = OffsetDateTime::try_now_local().unwrap();

            if now > self.current {
                self.current = now;
            }
        }

        // Create every possible combination of dates for each
        // DateSpec and add them to a vector.
        let mut candidates: Vec<OffsetDateTime> = vec![];

        for spec in self.schedule.specs.clone() {
            candidates.append(&mut compute_dates(self.current, spec));
        }

        // Iterate the vector of dates and find the next date
        // by subtracting the current date from each element
        // in the vector. Return the date that results in the
        // lowest delta.
        let next_date = candidates
            .iter()
            .min_by_key(|d| **d - self.current)
            .unwrap();

        Some(*next_date)
    }
}

fn compute_dates(base: OffsetDateTime, spec: DateSpec) -> Vec<OffsetDateTime> {
    let mut candidates = vec![];
    let today = base.date();
    let offset = base.offset();

    // For each specified time ...
    for time in spec.hours {
        // ... create an OffsetDateTime object for each upcoming weekday ...
        for i in 0..=6 {
            let mut date =
                PrimitiveDateTime::new(today + Duration::days(i), time).assume_offset(offset);

            if date < base {
                date += Duration::week();
            }

            candidates.push(date);
        }

        // ... remove all objects that match none of the desired weekdays (if any)
        // and increment the remaining dates according to the WeekdayModifier (if any) ...
        if let Some(ref days) = spec.days {
            candidates = candidates
                .into_iter()
                .filter(|c| days.iter().any(|x| x.0 == c.weekday()))
                .collect();

            for mut candidate in candidates.iter_mut() {
                let day_spec = days.iter().find(|x| x.0 == candidate.weekday()).unwrap();
                let day = candidate.day();

                match day_spec.1 {
                    WeekdayModifier::First => {
                        if day > 7 {
                            *candidate = wrap_to_next_month(candidate, 7);
                        }
                    }
                    WeekdayModifier::Second => {
                        if day > 14 {
                            *candidate = wrap_to_next_month(candidate, 14);
                        } else if day <= 7 {
                            *candidate += Duration::week();
                        }
                    }
                    WeekdayModifier::Third => {
                        if day > 21 {
                            *candidate = wrap_to_next_month(candidate, 21);
                        } else if day <= 7 {
                            *candidate += Duration::weeks(2);
                        } else if day <= 14 {
                            *candidate += Duration::week();
                        }
                    }
                    WeekdayModifier::Fourth => {
                        if day > 28 {
                            *candidate = wrap_to_next_month(candidate, 28);
                        } else if day <= 7 {
                            *candidate += Duration::weeks(2);
                        } else if day <= 14 {
                            *candidate += Duration::week();
                        } else if day <= 21 {
                            *candidate += Duration::week();
                        }
                    }
                    WeekdayModifier::None => {}
                }
            }
        }
    }

    candidates
}

// Wrap to a specific date (specific weekday and week) in the next month
// based on the weekday of the first day of the next month and an offset.
fn wrap_to_next_month(date: &OffsetDateTime, off: u8) -> OffsetDateTime {
    let offset = date.offset();
    let first_next = get_first_of_next_month(date.date());
    let delta = date.weekday().number_from_monday() - first_next.weekday().number_from_monday();
    let addend = if delta >= 0 { delta } else { off - delta };
    let new_date = first_next + Duration::days(addend.into());
    PrimitiveDateTime::new(new_date, date.time()).assume_offset(offset)
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

    Ok(times)
}

// Parse the weekday spec of an expression and return a sorted list.
fn parse_days(expression: &str) -> Result<Vec<(Weekday, WeekdayModifier)>, InvalidExpressionError> {
    let mut days = Vec::new();

    for item in expression.replace("and", ",").split(',') {
        let parts: Vec<&str> = item
            .trim()
            .trim_start_matches("the")
            .trim()
            .split_whitespace()
            .collect();

        let (modifier, raw_day) = if parts.len() == 2 {
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

        let day = match raw_day {
            "Monday" => Weekday::Monday,
            "Tuesday" => Weekday::Tuesday,
            "Wednesday" => Weekday::Wednesday,
            "Thursday" => Weekday::Thursday,
            "Friday" => Weekday::Friday,
            "Saturday" => Weekday::Saturday,
            "Sunday" => Weekday::Sunday,
            _ => return Err(InvalidExpressionError::UnknownWeekday),
        };

        let spec = (day, modifier);

        if !days.contains(&spec) {
            days.push(spec);
        } else {
            return Err(InvalidExpressionError::DuplicateInput);
        }
    }

    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{date, time};

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
            (Weekday::Monday, WeekdayModifier::None),
            (Weekday::Tuesday, WeekdayModifier::None),
            (Weekday::Thursday, WeekdayModifier::None),
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
            (Weekday::Monday, WeekdayModifier::First),
            (Weekday::Tuesday, WeekdayModifier::None),
            (Weekday::Thursday, WeekdayModifier::Fourth),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_1() {
        let expression = "at 5 PM (Monday and Thursday) in odd weeks";
        let result = DateSpec {
            hours: vec![time!(17:00:00)],
            days: Some(vec![
                (Weekday::Monday, WeekdayModifier::None),
                (Weekday::Thursday, WeekdayModifier::None),
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
                (Weekday::Monday, WeekdayModifier::First),
                (Weekday::Thursday, WeekdayModifier::None),
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
                (Weekday::Monday, WeekdayModifier::None),
                (Weekday::Friday, WeekdayModifier::None),
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

    #[test]
    fn test_compute_dates_1() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(12:00:00), time!(18:00:00)],
            days: None,
            weeks: WeekVariant::None,
        };
        let result = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 11), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 05), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 06), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 07), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 08), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 09), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 05), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 06), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 07), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 08), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 09), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_compute_dates_2() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(18:00:00)],
            days: Some(vec![
                (Weekday::Monday, WeekdayModifier::None),
                (Weekday::Thursday, WeekdayModifier::None),
            ]),
            weeks: WeekVariant::None,
        };
        let result = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 07), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_compute_dates_3() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(18:00:00)],
            days: Some(vec![
                (Weekday::Monday, WeekdayModifier::Second),
                (Weekday::Thursday, WeekdayModifier::None),
            ]),
            weeks: WeekVariant::None,
        };
        let result = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 14), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_compute_dates_4() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(12:00:00), time!(18:00:00)],
            days: Some(vec![
                (Weekday::Friday, WeekdayModifier::First),
                (Weekday::Thursday, WeekdayModifier::None),
            ]),
            weeks: WeekVariant::None,
        };
        let result = vec![
            PrimitiveDateTime::new(date!(2021 - 07 - 02), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_compute_dates_5() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 12), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(06:00:00), time!(12:00:00), time!(18:00:00)],
            days: Some(vec![
                (Weekday::Friday, WeekdayModifier::First),
                (Weekday::Thursday, WeekdayModifier::None),
                (Weekday::Monday, WeekdayModifier::Third),
            ]),
            weeks: WeekVariant::None,
        };
        let result = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 21), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 17), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 02), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 21), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 17), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 02), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 21), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 17), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 07 - 02), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }
}
