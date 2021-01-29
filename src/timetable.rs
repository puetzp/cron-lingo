use crate::error::*;
use std::collections::HashMap;
use std::iter::Iterator;
use std::str::FromStr;
use time::{Duration, OffsetDateTime, PrimitiveDateTime, Time};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_timetable() {
        let expression = "at 6, 8, 7 and 14 o'clock on Monday, Thursday and Saturday in even weeks";
        let timetable = Timetable::new(expression).unwrap();
        assert_eq!(timetable.hours, vec!(6, 7, 8, 14));
        assert_eq!(timetable.weekdays, Some(vec!(1, 4, 6)));
        assert_eq!(timetable.weeks, Some(WeekVariant::Even));
    }

    #[test]
    fn test_timetable_without_week_spec() {
        let expression = "at 6, 15 o'clock on Friday";
        let timetable = Timetable::new(expression).unwrap();
        assert_eq!(timetable.hours, vec!(6, 15));
        assert_eq!(timetable.weekdays, Some(vec!(5)));
        assert_eq!(timetable.weeks, None);
    }

    #[test]
    fn test_timetable_hours_only() {
        let expression = "at 6, 23 o'clock";
        let timetable = Timetable::new(expression).unwrap();
        assert_eq!(timetable.hours, vec!(6, 23));
        assert_eq!(timetable.weekdays, None);
        assert_eq!(timetable.weeks, None);
    }

    #[test]
    fn test_timetable_every_hour() {
        let expression = "at every hour";
        let timetable = Timetable::new(expression).unwrap();
        assert_eq!(timetable.hours, (0..=23).collect::<Vec<u8>>());
        assert_eq!(timetable.weekdays, None);
        assert_eq!(timetable.weeks, None);
    }

    #[test]
    fn test_timetable_without_weekday_spec() {
        let expression = "at 6, 23 o'clock in odd weeks";
        let timetable = Timetable::new(expression).unwrap();
        assert_eq!(timetable.hours, vec!(6, 23));
        assert_eq!(timetable.weekdays, None);
        assert_eq!(timetable.weeks, Some(WeekVariant::Odd));
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
    fn test_parse_weekdays_for_duplicate_error() {
        let expression = "at 13 o'clock on Monday and Monday and Friday";
        assert_eq!(
            parse_weekdays(expression).unwrap_err(),
            InvalidExpressionError::DuplicateInput
        );
    }

    #[test]
    fn test_timetable_iteration1() {
        use time::{date, time};
        let timetable = Timetable {
            base: PrimitiveDateTime::new(date!(2021 - 07 - 28), time!(15:00:00)).assume_utc(),
            hours: vec![6, 18],
            weekdays: Some(vec![1, 3]),
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
            timetable
                .into_iter()
                .take(5)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_timetable_iteration2() {
        use time::{date, time};
        let timetable = Timetable {
            base: PrimitiveDateTime::new(date!(2021 - 02 - 16), time!(08:24:47)).assume_utc(),
            hours: vec![12],
            weekdays: Some(vec![0, 5]),
            weeks: Some(WeekVariant::Even),
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 02 - 26), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 02 - 28), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 03 - 12), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 03 - 14), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 03 - 26), time!(12:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 03 - 28), time!(12:00:00)).assume_utc(),
        ];
        assert_eq!(
            timetable
                .into_iter()
                .take(6)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_timetable_iteration3() {
        use time::{date, time};
        let timetable = Timetable {
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
            timetable
                .into_iter()
                .take(3)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }
}

// This HashMap is just used to simplify the parse_weekdays function.
lazy_static::lazy_static! {
    static ref WEEKDAY_MAPPING: HashMap<&'static str, u8> = {
        let mut t = HashMap::new();
        t.insert("Sunday", 0);
        t.insert("Monday", 1);
        t.insert("Tuesday", 2);
        t.insert("Wednesday", 3);
        t.insert("Thursday", 4);
        t.insert("Friday", 5);
        t.insert("Saturday", 6);
        t
    };
}

/// A timetable that is built from an expression and can be iterated over
/// in order to compute the next date(s) that match the specification.
/// This is the only way (at this point) to use `Timetable` in a meaningful
/// way.
///
/// The expression must adhere to a specific syntax. See the module-level
/// documentation for the full range of possibilities.
#[derive(Debug, PartialEq)]
pub struct Timetable {
    base: OffsetDateTime,
    hours: Vec<u8>,
    weekdays: Option<Vec<u8>>,
    weeks: Option<WeekVariant>,
}

impl Timetable {
    /// Attempt to create a new `Timetable` object from an expression.
    ///
    /// ```rust
    /// use cron_lingo::Timetable;
    ///
    /// let expr = "at 6 and 18 o'clock on Monday and Thursday in even weeks";
    /// assert!(Timetable::new(expr).is_ok());
    /// ```
    pub fn new(expression: &str) -> Result<Self, InvalidExpressionError> {
        Timetable::from_str(expression)
    }
}

impl FromStr for Timetable {
    type Err = InvalidExpressionError;

    fn from_str(expression: &str) -> Result<Self, Self::Err> {
        let tt = Timetable {
            base: OffsetDateTime::try_now_local().unwrap(),
            hours: parse_hours(expression)?,
            weekdays: parse_weekdays(expression)?,
            weeks: parse_weeks(expression)?,
        };
        Ok(tt)
    }
}

impl Iterator for Timetable {
    type Item = OffsetDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        let now = OffsetDateTime::try_now_local().unwrap();

        if now > self.base {
            self.base = now;
        }

        let (mut next_date, next_time) = match &self.weekdays {
            Some(weekdays) => {
                let this_weekday = self.base.weekday().number_days_from_sunday();

                let (next_hour, next_weekday) = if weekdays.iter().any(|&x| x == this_weekday) {
                    match self.hours.iter().find(|&&x| x > self.base.hour()) {
                        Some(n) => (*n, this_weekday),
                        None => match weekdays.iter().find(|&&x| x > this_weekday) {
                            Some(wd) => (self.hours[0], *wd),
                            None => (self.hours[0], weekdays[0]),
                        },
                    }
                } else {
                    match weekdays.iter().find(|&&x| x > this_weekday) {
                        Some(wd) => (self.hours[0], *wd),
                        None => (self.hours[0], weekdays[0]),
                    }
                };

                let next_time = Time::try_from_hms(next_hour, 0, 0).unwrap();

                let day_addend = {
                    if this_weekday > next_weekday {
                        7 - this_weekday + next_weekday
                    } else {
                        next_weekday - this_weekday
                    }
                };

                let next_date = self.base.date() + Duration::days(day_addend.into());

                (next_date, next_time)
            }
            None => match self.hours.iter().find(|&&x| x > self.base.hour()) {
                Some(h) => {
                    let next_time = Time::try_from_hms(*h, 0, 0).unwrap();
                    (self.base.date(), next_time)
                }
                None => {
                    let next_time = Time::try_from_hms(self.hours[0], 0, 0).unwrap();
                    let next_date = self.base.date() + Duration::day();
                    (next_date, next_time)
                }
            },
        };

        if let Some(weeks) = &self.weeks {
            match weeks {
                WeekVariant::Even => {
                    if !next_date.week() % 2 == 0 {
                        next_date += Duration::week();
                    }
                }
                WeekVariant::Odd => {
                    if next_date.week() % 2 == 0 {
                        next_date += Duration::week();
                    }
                }
            }
        }

        let next_date_time =
            PrimitiveDateTime::new(next_date, next_time).assume_offset(self.base.offset());

        self.base = next_date_time;

        Some(next_date_time)
    }
}

#[derive(Debug, PartialEq)]
enum WeekVariant {
    Even,
    Odd,
}

#[derive(Debug, PartialEq)]
enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

// Parse the hour spec of an expression and return a sorted list.
// Determine the start end end bounds of the relevant part, parse
// each comma-separated value and add them to a vector.
fn parse_hours(expression: &str) -> Result<Vec<u8>, InvalidExpressionError> {
    let start = match expression.find("at") {
        Some(start_idx) => start_idx,
        None => return Err(InvalidExpressionError::InvalidHourSpec),
    };

    let mut section = if let Some(end_idx) = expression.find("on") {
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
    let start = match expression.find("on") {
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
            "Sunday" => Weekday::Sunday,
            "Monday" => Weekday::Monday,
            "Tuesday" => Weekday::Tuesday,
            "Wednesday" => Weekday::Wednesday,
            "Thursday" => Weekday::Thursday,
            "Friday" => Weekday::Friday,
            "Saturday" => Weekday::Saturday,
            _ => return Err(InvalidExpressionError::UnknownWeekday),
        };

        if !weekdays.contains(weekday) {
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
        None => Ok(None),
    }
}
