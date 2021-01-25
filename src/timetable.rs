use crate::error::InvalidExpressionError;
use std::error::Error;
use std::ops::Sub;
use std::str::FromStr;
use time::{Duration, OffsetDateTime, PrimitiveDateTime, Time};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hours1() {
        let expression = "at 6, 8, 7 and 14 o'clock on Monday, Thursday and Saturday in even weeks";
        let result = vec![6, 7, 8, 14];
        assert_eq!(parse_hours(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_hours2() {
        let expression = "at 6, 15 o'clock on Friday";
        let result = vec![6, 15];
        assert_eq!(parse_hours(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_hours3() {
        let expression = "at every hour on Friday and Saturday";
        let result: Vec<u8> = (0..24).collect();
        assert_eq!(parse_hours(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_hours4() {
        let expression = "at 12 o'clock";
        let result = vec![12];
        assert_eq!(parse_hours(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_hours5() {
        let expression = "at every hour";
        let result: Vec<u8> = (0..24).collect();
        assert_eq!(parse_hours(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_hours_for_error1() {
        let expression = "at 6, 15, 24 o'clock on Friday";
        assert_eq!(
            *parse_hours(expression)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_hours_for_error2() {
        let expression = "at 6, 15, 17 18 o'clock on Monday";
        assert_eq!(
            *parse_hours(expression)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_hours_for_error3() {
        let expression = "at 6, 6, 15, 17 18 o'clock";
        assert_eq!(
            *parse_hours(expression)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_weekdays1() {
        let expression = "at 6 o'clock on Sunday and Thursday and Monday in odd weeks";
        let result = vec![0, 1, 4];
        assert_eq!(parse_weekdays(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_weekdays2() {
        let expression = "at 13 o'clock on Monday, Friday";
        let result = vec![1, 5];
        assert_eq!(parse_weekdays(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_weekdays_for_error1() {
        let expression = "at 16 o'clock on Monday and Thursd";
        assert_eq!(
            *parse_weekdays(expression)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_weekdays_for_error2() {
        let expression = "at 12 o'clock on Tuesday Saturday";
        assert_eq!(
            *parse_weekdays(expression)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_weeks1() {
        let expression = "at 6 o'clock on Sunday, Monday and Thursday in odd weeks";
        assert_eq!(parse_weeks(expression).unwrap(), WeekVariant::Odd);
    }

    #[test]
    fn test_parse_weeks2() {
        let expression =
            "at 6 o'clock on Sunday, Monday and Thursday in the third and first week of the month";
        let result = WeekVariant::Multiple(vec![Week::First, Week::Third]);
        assert_eq!(parse_weeks(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_weeks_for_error() {
        let expression =
            "at 6 o'clock on Sunday, Monday and Thursday in the first, third and odd week of the month";
        assert_eq!(
            *parse_weeks(expression)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_compute_next_date1() {
        use time::{date, time};
        let base = PrimitiveDateTime::new(date!(2020 - 04 - 14), time!(05:00:00)).assume_utc();
        let timetable = Timetable {
            hours: vec![3, 9, 15, 21],
            weekdays: vec![1, 4],
            weeks: WeekVariant::Even,
        };
        let result = PrimitiveDateTime::new(date!(2020 - 04 - 16), time!(03:00:00)).assume_utc();
        assert_eq!(timetable.compute_next_date(base).unwrap(), result);
    }

    #[test]
    fn test_compute_next_date2() {
        use time::{date, time};
        let base = PrimitiveDateTime::new(date!(2020 - 04 - 30), time!(22:00:00)).assume_utc();
        let timetable = Timetable {
            hours: vec![3, 9, 15, 21],
            weekdays: vec![1, 3],
            weeks: WeekVariant::Odd,
        };
        let result = PrimitiveDateTime::new(date!(2020 - 05 - 04), time!(03:00:00)).assume_utc();
        assert_eq!(timetable.compute_next_date(base).unwrap(), result);
    }

    #[test]
    fn test_compute_next_date3() {
        use time::{date, time};
        let base = PrimitiveDateTime::new(date!(2021 - 01 - 14), time!(16:00:00)).assume_utc();
        let timetable = Timetable {
            hours: vec![6, 18],
            weekdays: vec![1, 4],
            weeks: WeekVariant::Multiple(vec![Week::First, Week::Third]),
        };
        let result = PrimitiveDateTime::new(date!(2021 - 01 - 14), time!(18:00:00)).assume_utc();
        assert_eq!(timetable.compute_next_date(base).unwrap(), result);
    }

    #[test]
    fn test_compute_next_date4() {
        use time::{date, time};
        let base = PrimitiveDateTime::new(date!(2021 - 02 - 04), time!(12:00:00)).assume_utc();
        let timetable = Timetable {
            hours: vec![6, 18],
            weekdays: vec![3, 5],
            weeks: WeekVariant::Multiple(vec![Week::First]),
        };
        let result = PrimitiveDateTime::new(date!(2021 - 02 - 05), time!(06:00:00)).assume_utc();
        assert_eq!(timetable.compute_next_date(base).unwrap(), result);
    }

    #[test]
    fn test_compute_next_date5() {
        use time::{date, time};
        let base = PrimitiveDateTime::new(date!(2021 - 02 - 08), time!(12:00:00)).assume_utc();
        let timetable = Timetable {
            hours: vec![6, 18],
            weekdays: vec![3, 5],
            weeks: WeekVariant::Multiple(vec![Week::First]),
        };
        let result = PrimitiveDateTime::new(date!(2021 - 03 - 03), time!(06:00:00)).assume_utc();
        assert_eq!(timetable.compute_next_date(base).unwrap(), result);
    }

    #[test]
    fn test_compute_next_date6() {
        use time::{date, time};
        let base = PrimitiveDateTime::new(date!(2021 - 03 - 06), time!(12:00:00)).assume_utc();
        let timetable = Timetable {
            hours: vec![6, 18],
            weekdays: vec![5],
            weeks: WeekVariant::Multiple(vec![Week::Fourth]),
        };
        let result = PrimitiveDateTime::new(date!(2021 - 03 - 26), time!(06:00:00)).assume_utc();
        assert_eq!(timetable.compute_next_date(base).unwrap(), result);
    }

    #[test]
    fn test_compute_next_date7() {
        use time::{date, time};
        let base = PrimitiveDateTime::new(date!(2021 - 05 - 24), time!(12:00:00)).assume_utc();
        let timetable = Timetable {
            hours: vec![6, 18],
            weekdays: vec![5],
            weeks: WeekVariant::Multiple(vec![Week::Second]),
        };
        let result = PrimitiveDateTime::new(date!(2021 - 06 - 11), time!(06:00:00)).assume_utc();
        assert_eq!(timetable.compute_next_date(base).unwrap(), result);
    }
}

#[derive(Debug, PartialEq)]
pub struct Timetable {
    hours: Vec<u8>,
    weekdays: Vec<u8>,
    weeks: WeekVariant,
}

impl FromStr for Timetable {
    type Err = InvalidExpressionError;

    fn from_str(expression: &str) -> Result<Self, Self::Err> {
        let tt = Timetable {
            hours: parse_hours(expression)?,
            weekdays: parse_weekdays(expression)?,
            weeks: parse_weeks(expression)?,
        };
        Ok(tt)
    }
}

#[derive(Debug, PartialEq)]
enum WeekVariant {
    Even,
    Odd,
    Multiple(Vec<Week>),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Week {
    First,
    Second,
    Third,
    Fourth,
}

impl Sub for &Week {
    type Output = i8;
    fn sub(self, other: &Week) -> Self::Output {
        let self_num = match self {
            Week::First => 1,
            Week::Second => 2,
            Week::Third => 3,
            Week::Fourth => 4,
        };

        match other {
            Week::First => self_num - 1,
            Week::Second => self_num - 2,
            Week::Third => self_num - 3,
            Week::Fourth => self_num - 4,
        }
    }
}

impl Timetable {
    pub fn compute_next_date(self, base: OffsetDateTime) -> Result<OffsetDateTime, Box<dyn Error>> {
        let this_weekday = base.weekday().number_days_from_sunday();

        let (next_hour, next_weekday) = if self.weekdays.iter().any(|&x| x == this_weekday) {
            match self.hours.iter().find(|&&x| x > base.hour()) {
                Some(n) => (*n, this_weekday),
                None => match self.weekdays.iter().find(|&&x| x > this_weekday) {
                    Some(wd) => (self.hours[0], *wd),
                    None => (self.hours[0], self.weekdays[0]),
                },
            }
        } else {
            match self.weekdays.iter().find(|&&x| x > this_weekday) {
                Some(wd) => (self.hours[0], *wd),
                None => (self.hours[0], self.weekdays[0]),
            }
        };

        let next_time = Time::try_from_hms(next_hour, 0, 0)?;

        let day_addend = {
            if this_weekday > next_weekday {
                7 - this_weekday + next_weekday
            } else {
                next_weekday - this_weekday
            }
        };

        let mut next_date = base.date() + Duration::days(day_addend.into());

        match self.weeks {
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
            WeekVariant::Multiple(weeks) => {
                let days_from_sunday = next_date.weekday().number_days_from_sunday();

                let this_week = if days_from_sunday >= next_date.day() {
                    Week::First
                } else {
                    let last_sunday = (next_date - Duration::days(days_from_sunday as i64)).day();

                    if last_sunday <= 7 {
                        Week::Second
                    } else if last_sunday > 7 && last_sunday <= 14 {
                        Week::Third
                    } else {
                        Week::Fourth
                    }
                };

                if !weeks.contains(&this_week) {
                    match weeks.iter().find(|&x| x > &this_week) {
                        Some(w) => {
                            let delta_to_next = w - &this_week;
                            next_date += Duration::weeks(delta_to_next as i64);
                        }
                        None => {
                            let delta_to_next = 4 + (&weeks[0] - &this_week);
                            let tmp_date = next_date + Duration::weeks(delta_to_next as i64);

                            if tmp_date.month() == next_date.month() {
                                next_date = tmp_date + Duration::week();
                            } else {
                                next_date = tmp_date;
                            }
                        }
                    };
                }
            }
        }

        let next_date_time =
            PrimitiveDateTime::new(next_date, next_time).assume_offset(base.offset());

        Ok(next_date_time)
    }
}

fn parse_hours(expression: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let start = match expression.find("at") {
        Some(start_idx) => start_idx,
        None => return Err(InvalidExpressionError.into()),
    };

    let mut section = match expression.find("on") {
        Some(end_idx) => expression[start + 2..end_idx].trim(),
        None => expression[start + 2..].trim(),
    };

    if section == "every hour" {
        return Ok((0..24).collect());
    }

    section = match section.strip_suffix("o'clock") {
        Some(stripped) => stripped,
        None => return Err(InvalidExpressionError.into()),
    };

    let section = section.replace("and", ",");

    let mut hours = Vec::new();

    for mut item in section.split(",") {
        item = item.trim();

        match item.parse::<u8>() {
            Ok(num) => {
                if num < 24 && !hours.contains(&num) {
                    hours.push(num);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            Err(_) => return Err(InvalidExpressionError.into()),
        }
    }

    hours.sort_unstable();

    Ok(hours)
}

fn parse_weekdays(expression: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let start = match expression.find("on") {
        Some(start_idx) => start_idx,
        None => return Err(InvalidExpressionError.into()),
    };

    let section = match expression.find("in") {
        Some(end_idx) => expression[start + 2..end_idx].trim(),
        None => expression[start + 2..].trim(),
    };

    let mut weekdays = Vec::new();

    for mut item in section.replace("and", ",").split(",") {
        item = item.trim();

        match item {
            "Sunday" => {
                if !weekdays.contains(&0) {
                    weekdays.push(0);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            "Monday" => {
                if !weekdays.contains(&1) {
                    weekdays.push(1);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            "Tuesday" => {
                if !weekdays.contains(&2) {
                    weekdays.push(2);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            "Wednesday" => {
                if !weekdays.contains(&3) {
                    weekdays.push(3);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            "Thursday" => {
                if !weekdays.contains(&4) {
                    weekdays.push(4);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            "Friday" => {
                if !weekdays.contains(&5) {
                    weekdays.push(5);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            "Saturday" => {
                if !weekdays.contains(&6) {
                    weekdays.push(6);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            _ => return Err(InvalidExpressionError.into()),
        }
    }

    weekdays.sort_unstable();

    Ok(weekdays)
}

fn parse_weeks(expression: &str) -> Result<WeekVariant, Box<dyn Error>> {
    match expression.find("weeks") {
        Some(end_idx) => {
            let start_idx = match expression.find("in") {
                Some(start_idx) => start_idx,
                None => return Err(InvalidExpressionError.into()),
            };

            let section = expression[start_idx + 2..end_idx].trim();

            match section {
                "even" => Ok(WeekVariant::Even),
                "odd" => Ok(WeekVariant::Odd),
                _ => return Err(InvalidExpressionError.into()),
            }
        }
        None => match expression.find("week of the month") {
            Some(end_idx) => {
                let start_idx = match expression.find("in the") {
                    Some(start_idx) => start_idx,
                    None => return Err(InvalidExpressionError.into()),
                };

                let section = expression[start_idx + 6..end_idx].trim();

                let mut weeks = Vec::new();

                for mut item in section.replace("and", ",").split(",") {
                    item = item.trim();

                    match item {
                        "first" => weeks.push(Week::First),
                        "second" => weeks.push(Week::Second),
                        "third" => weeks.push(Week::Third),
                        "fourth" => weeks.push(Week::Fourth),
                        _ => return Err(InvalidExpressionError.into()),
                    }
                }
                weeks.sort();
                Ok(WeekVariant::Multiple(weeks))
            }
            None => return Err(InvalidExpressionError.into()),
        },
    }
}
