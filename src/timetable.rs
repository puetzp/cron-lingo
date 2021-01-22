use crate::error::InvalidExpressionError;
use std::error::Error;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hours1() {
        let expression = "at 6, 7, 8 and 14 o'clock on Monday, Thursday and Saturday in even weeks";
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
    fn test_parse_weekdays1() {
        let expression = "at 6 o'clock on Sunday, Monday and Thursday in odd weeks";
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
            "at 6 o'clock on Sunday, Monday and Thursday in the first and third week of the month";
        let result = WeekVariant::Multiple(vec!["first".to_string(), "third".to_string()]);
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
    Multiple(Vec<String>),
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
                if num < 24 {
                    hours.push(num);
                } else {
                    return Err(InvalidExpressionError.into());
                }
            }
            Err(_) => return Err(InvalidExpressionError.into()),
        }
    }

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
            "Sunday" => weekdays.push(0),
            "Monday" => weekdays.push(1),
            "Tuesday" => weekdays.push(2),
            "Wednesday" => weekdays.push(3),
            "Thursday" => weekdays.push(4),
            "Friday" => weekdays.push(5),
            "Saturday" => weekdays.push(6),
            _ => return Err(InvalidExpressionError.into()),
        }
    }

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
                        "first" | "second" | "third" | "fourth" | "last" => {
                            weeks.push(item.to_string());
                        }
                        _ => return Err(InvalidExpressionError.into()),
                    }
                }
                Ok(WeekVariant::Multiple(weeks))
            }
            None => return Err(InvalidExpressionError.into()),
        },
    }
}
