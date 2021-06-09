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
    }

    // ... remove all objects that match none of the desired weekdays (if any)
    // and increment the remaining dates according to the optional WeekdayModifier
    // and WeekVariant.
    if let Some(ref days) = spec.days {
        let weeks = spec.weeks.clone();

        candidates
            .iter_mut()
            .filter(|c| days.iter().any(|x| x.0 == c.weekday()))
            .for_each(|candidate| {
                let day_modifier = days.iter().find(|x| x.0 == candidate.weekday()).unwrap().1;

                while !check_date_validity(candidate, day_modifier, weeks) {
                    *candidate += Duration::week();
                }
            });
    }

    // ... and return the filtered date candidates of this DateSpec.
    candidates
}

// Takes a date and checks its bounds according to optional WeekdayModifiers
// and/or WeekVariants. Returns false if the date does not match the specified rules.
fn check_date_validity(
    date: &OffsetDateTime,
    weekday_mod: Option<WeekdayModifier>,
    week_mod: Option<WeekVariant>,
) -> bool {
    let is_correct_day = match weekday_mod {
        Some(modifier) => {
            let day = date.day();

            match modifier {
                WeekdayModifier::First => day <= 7,
                WeekdayModifier::Second => day > 7 && day <= 14,
                WeekdayModifier::Third => day > 14 && day <= 21,
                WeekdayModifier::Fourth => day > 21 && day <= 28,
            }
        }
        None => true,
    };

    let is_correct_week = match week_mod {
        Some(modifier) => {
            let week = date.week();

            match modifier {
                WeekVariant::Even => week % 2 == 0,
                WeekVariant::Odd => week % 2 != 0,
            }
        }
        None => true,
    };

    is_correct_day && is_correct_week
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
            "in odd weeks" => Some(WeekVariant::Odd),
            "in even weeks" => Some(WeekVariant::Even),
            _ => return Err(InvalidExpressionError::InvalidWeekSpec),
        }
    } else {
        None
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
fn parse_days(
    expression: &str,
) -> Result<Vec<(Weekday, Option<WeekdayModifier>)>, InvalidExpressionError> {
    let mut days = Vec::new();

    for item in expression.replace("and", ",").split(',') {
        let spec = match item.trim().trim_start_matches("the").trim() {
            "Mondays" => (Weekday::Monday, None),
            "first Monday" | "1st Monday" => (Weekday::Monday, Some(WeekdayModifier::First)),
            "second Monday" | "2nd Monday" => (Weekday::Monday, Some(WeekdayModifier::Second)),
            "third Monday" | "3rd Monday" => (Weekday::Monday, Some(WeekdayModifier::Third)),
            "fourth Monday" | "4th Monday" => (Weekday::Monday, Some(WeekdayModifier::Fourth)),
            "Tuesdays" => (Weekday::Tuesday, None),
            "first Tuesday" | "1st Tuesday" => (Weekday::Tuesday, Some(WeekdayModifier::First)),
            "second Tuesday" | "2nd Tuesday" => (Weekday::Tuesday, Some(WeekdayModifier::Second)),
            "third Tuesday" | "3rd Tuesday" => (Weekday::Tuesday, Some(WeekdayModifier::Third)),
            "fourth Tuesday" | "4th Tuesday" => (Weekday::Tuesday, Some(WeekdayModifier::Fourth)),
            "Wednesdays" => (Weekday::Wednesday, None),
            "first Wednesday" | "1st Wednesday" => {
                (Weekday::Wednesday, Some(WeekdayModifier::First))
            }
            "second Wednesday" | "2nd Wednesday" => {
                (Weekday::Wednesday, Some(WeekdayModifier::Second))
            }
            "third Wednesday" | "3rd Wednesday" => {
                (Weekday::Wednesday, Some(WeekdayModifier::Third))
            }
            "fourth Wednesday" | "4th Wednesday" => {
                (Weekday::Wednesday, Some(WeekdayModifier::Fourth))
            }
            "Thursdays" => (Weekday::Thursday, None),
            "first Thursday" | "1st Thursday" => (Weekday::Thursday, Some(WeekdayModifier::First)),
            "second Thursday" | "2nd Thursday" => {
                (Weekday::Thursday, Some(WeekdayModifier::Second))
            }
            "third Thursday" | "3rd Thursday" => (Weekday::Thursday, Some(WeekdayModifier::Third)),
            "fourth Thursday" | "4th Thursday" => {
                (Weekday::Thursday, Some(WeekdayModifier::Fourth))
            }
            "Fridays" => (Weekday::Friday, None),
            "first Friday" | "1st Friday" => (Weekday::Friday, Some(WeekdayModifier::First)),
            "second Friday" | "2nd Friday" => (Weekday::Friday, Some(WeekdayModifier::Second)),
            "third Friday" | "3rd Friday" => (Weekday::Friday, Some(WeekdayModifier::Third)),
            "fourth Friday" | "4th Friday" => (Weekday::Friday, Some(WeekdayModifier::Fourth)),
            "Saturdays" => (Weekday::Saturday, None),
            "first Saturday" | "1st Saturday" => (Weekday::Saturday, Some(WeekdayModifier::First)),
            "second Saturday" | "2nd Saturday" => {
                (Weekday::Saturday, Some(WeekdayModifier::Second))
            }
            "third Saturday" | "3rd Saturday" => (Weekday::Saturday, Some(WeekdayModifier::Third)),
            "fourth Saturday" | "4th Saturday" => {
                (Weekday::Saturday, Some(WeekdayModifier::Fourth))
            }
            "Sundays" => (Weekday::Sunday, None),
            "first Sunday" | "1st Sunday" => (Weekday::Sunday, Some(WeekdayModifier::First)),
            "second Sunday" | "2nd Sunday" => (Weekday::Sunday, Some(WeekdayModifier::Second)),
            "third Sunday" | "3rd Sunday" => (Weekday::Sunday, Some(WeekdayModifier::Third)),
            "fourth Sunday" | "4th Sunday" => (Weekday::Sunday, Some(WeekdayModifier::Fourth)),
            _ => return Err(InvalidExpressionError::InvalidWeekdaySpec),
        };

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
        let expression = "at 4 PM on Mondays, at 6 PM on Thursdays and at 3 AM";
        let result = vec!["at 4 PM on Mondays", "6 PM on Thursdays", "3 AM"];
        assert_eq!(split_expression(expression), result);
    }

    #[test]
    fn test_split_block() {
        let block = "at 4 PM and 6 PM on Mondays and Tuesdays";
        let result = ("4 PM and 6 PM", Some("Mondays and Tuesdays"), None);
        assert_eq!(split_block(block).unwrap(), result);
    }

    #[test]
    fn test_split_block_with_week_mod() {
        let block = "at 4 PM and 6 PM on Mondays and Tuesdays in odd weeks";
        let result = (
            "4 PM and 6 PM",
            Some("Mondays and Tuesdays"),
            Some("in odd weeks"),
        );
        assert_eq!(split_block(block).unwrap(), result);
    }

    #[test]
    fn test_split_block_for_error() {
        let block = "at 4 PM and 6 PM on Mondays and on Tuesdays";
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
        let expression = "Mondays, Tuesdays and Thursdays";
        let result = vec![
            (Weekday::Monday, None),
            (Weekday::Tuesday, None),
            (Weekday::Thursday, None),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_days_for_duplicate_error() {
        let expression = "Mondays, Mondays and Thursdays";
        assert_eq!(
            parse_days(expression).unwrap_err(),
            InvalidExpressionError::DuplicateInput
        );
    }

    #[test]
    fn test_parse_days_with_modifiers() {
        let expression = "the first Monday, Tuesdays and the 4th Thursday";
        let result = vec![
            (Weekday::Monday, Some(WeekdayModifier::First)),
            (Weekday::Tuesday, None),
            (Weekday::Thursday, Some(WeekdayModifier::Fourth)),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_1() {
        let expression = "at 5 PM (Mondays and Thursdays) in odd weeks";
        let result = DateSpec {
            hours: vec![time!(17:00:00)],
            days: Some(vec![(Weekday::Monday, None), (Weekday::Thursday, None)]),
            weeks: Some(WeekVariant::Odd),
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_2() {
        let expression = "at 5 AM  and 6:30 PM (first Monday and Thursdays)";
        let result = DateSpec {
            hours: vec![time!(05:00:00), time!(18:30:00)],
            days: Some(vec![
                (Weekday::Monday, Some(WeekdayModifier::First)),
                (Weekday::Thursday, None),
            ]),
            weeks: None,
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_3() {
        let expression = "at 6:30 AM and 6:30 PM on Mondays and Fridays in even weeks";
        let result = DateSpec {
            hours: vec![time!(06:30:00), time!(18:30:00)],
            days: Some(vec![(Weekday::Monday, None), (Weekday::Friday, None)]),
            weeks: Some(WeekVariant::Even),
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_4() {
        let expression = "at every full hour on Mondays";
        assert!(parse_block(expression).is_ok());
    }

    #[test]
    fn test_parse_block_5() {
        let expression = "at 6 PM in even weeks";
        let result = DateSpec {
            hours: vec![time!(18:00:00)],
            days: None,
            weeks: Some(WeekVariant::Even),
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_compute_dates_1() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(12:00:00), time!(18:00:00)],
            days: None,
            weeks: None,
        };
        let mut result = vec![
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
        assert_eq!(compute_dates(base, spec).sort(), result.sort());
    }

    #[test]
    fn test_compute_dates_2() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(18:00:00)],
            days: Some(vec![(Weekday::Monday, None), (Weekday::Thursday, None)]),
            weeks: None,
        };
        let mut result = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 07), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(compute_dates(base, spec).sort(), result.sort());
    }

    #[test]
    fn test_compute_dates_3() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(18:00:00)],
            days: Some(vec![
                (Weekday::Monday, Some(WeekdayModifier::Second)),
                (Weekday::Thursday, None),
            ]),
            weeks: None,
        };
        let mut result = vec![
            PrimitiveDateTime::new(date!(2021 - 06 - 14), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(compute_dates(base, spec).sort(), result.sort());
    }

    #[test]
    fn test_compute_dates_4() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(12:00:00), time!(18:00:00)],
            days: Some(vec![
                (Weekday::Friday, Some(WeekdayModifier::First)),
                (Weekday::Thursday, None),
            ]),
            weeks: None,
        };
        let mut result = vec![
            PrimitiveDateTime::new(date!(2021 - 07 - 02), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 04), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 06 - 10), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(compute_dates(base, spec).sort(), result.sort());
    }

    #[test]
    fn test_compute_dates_5() {
        let base = PrimitiveDateTime::new(date!(2021 - 06 - 12), time!(13:38:00)).assume_utc();
        let spec = DateSpec {
            hours: vec![time!(06:00:00), time!(12:00:00), time!(18:00:00)],
            days: Some(vec![
                (Weekday::Friday, Some(WeekdayModifier::First)),
                (Weekday::Thursday, None),
                (Weekday::Monday, Some(WeekdayModifier::Third)),
            ]),
            weeks: None,
        };
        let mut result = vec![
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
        assert_eq!(compute_dates(base, spec).sort(), result.sort());
    }
}
