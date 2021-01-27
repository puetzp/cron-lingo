use crate::error::InvalidExpressionError;
use std::error::Error;
use std::iter::Iterator;
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
        assert!(parse_hours(expression).is_err());
    }

    #[test]
    fn test_parse_hours_for_error2() {
        let expression = "at 6, 15, 17 18 o'clock on Monday";
        assert!(parse_hours(expression).is_err());
    }

    #[test]
    fn test_parse_hours_for_error3() {
        let expression = "at 6, 6, 15, 17 18 o'clock";
        assert!(parse_hours(expression).is_err());
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
        assert!(parse_weekdays(expression).is_err());
    }

    #[test]
    fn test_parse_weekdays_for_error2() {
        let expression = "at 12 o'clock on Tuesday Saturday";
        assert!(parse_weekdays(expression).is_err());
    }

    #[test]
    fn test_parse_weeks() {
        let expression = "at 6 o'clock on Sunday, Monday and Thursday in odd weeks";
        assert_eq!(parse_weeks(expression).unwrap(), WeekVariant::Odd);
    }

    #[test]
    fn test_parse_weeks_for_error() {
        let expression = "at 6 o'clock on Sunday, Monday and Thursday in even and odd weeks";
        assert!(parse_weeks(expression).is_err());
    }

    #[test]
    fn test_timetable1() {
        use time::{date, time};
        let mut timetable = Timetable {
            base: Some(PrimitiveDateTime::new(date!(2020 - 04 - 14), time!(05:00:00)).assume_utc()),
            hours: vec![3, 9, 15, 21],
            weekdays: vec![1, 4],
            weeks: WeekVariant::Even,
        };
        let result = PrimitiveDateTime::new(date!(2020 - 04 - 16), time!(03:00:00)).assume_utc();
        assert_eq!(timetable.next().unwrap(), result);
    }

    #[test]
    fn test_timetable2() {
        use time::{date, time};
        let mut timetable = Timetable {
            base: Some(PrimitiveDateTime::new(date!(2020 - 04 - 30), time!(22:00:00)).assume_utc()),
            hours: vec![3, 9, 15, 21],
            weekdays: vec![1, 3],
            weeks: WeekVariant::Odd,
        };
        let result = PrimitiveDateTime::new(date!(2020 - 05 - 04), time!(03:00:00)).assume_utc();
        assert_eq!(timetable.next().unwrap(), result);
    }

    #[test]
    fn test_timetable3() {
        use time::{date, time};
        let timetable = Timetable {
            base: Some(PrimitiveDateTime::new(date!(2021 - 01 - 27), time!(15:00:00)).assume_utc()),
            hours: vec![6, 18],
            weekdays: vec![1, 3],
            weeks: WeekVariant::Even,
        };
        let result: Vec<OffsetDateTime> = vec![
            PrimitiveDateTime::new(date!(2021 - 01 - 27), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 02 - 08), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 02 - 08), time!(18:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 02 - 10), time!(06:00:00)).assume_utc(),
            PrimitiveDateTime::new(date!(2021 - 02 - 10), time!(18:00:00)).assume_utc(),
        ];
        assert_eq!(
            timetable
                .into_iter()
                .take(5)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }
}

#[derive(Debug, PartialEq)]
pub struct Timetable {
    base: Option<OffsetDateTime>,
    hours: Vec<u8>,
    weekdays: Vec<u8>,
    weeks: WeekVariant,
}

impl Timetable {
    pub fn new(expression: &str) -> Result<Self, Box<dyn Error>> {
        let mut timetable = Timetable::from_str(expression)?;
        timetable.base = Some(OffsetDateTime::try_now_local()?);
        Ok(timetable)
    }
}

impl FromStr for Timetable {
    type Err = InvalidExpressionError;

    fn from_str(expression: &str) -> Result<Self, Self::Err> {
        let tt = Timetable {
            base: None,
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
        let base = match self.base {
            Some(date) => date,
            None => OffsetDateTime::try_now_local().unwrap(),
        };

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

        let next_time = Time::try_from_hms(next_hour, 0, 0).unwrap();

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
        }

        let next_date_time =
            PrimitiveDateTime::new(next_date, next_time).assume_offset(base.offset());

        self.base = Some(next_date_time);

        Some(next_date_time)
    }
}

#[derive(Debug, PartialEq)]
enum WeekVariant {
    Even,
    Odd,
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

    for mut item in section.split(',') {
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

    for mut item in section.replace("and", ",").split(',') {
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
                _ => Err(InvalidExpressionError.into()),
            }
        }
        None => Err(InvalidExpressionError.into()),
    }
}
