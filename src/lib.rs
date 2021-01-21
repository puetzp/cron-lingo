use std::error::Error;
use std::fmt;
use time::OffsetDateTime;

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
    fn test_parse_weeks() {
        let expression = "at 6 o'clock on Sunday, Monday and Thursday in odd weeks";
        let result = String::from("odd");
        assert_eq!(parse_weeks(expression).unwrap(), result);
    }
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq)]
struct InvalidExpressionError;

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid expression")
    }
}

impl Error for InvalidExpressionError {}

fn parse_hours(expression: &str) -> Result<Vec<u8>> {
    let start = match expression.find("at") {
        Some(start_idx) => start_idx,
        None => return Err(InvalidExpressionError.into()),
    };

    let mut section = match expression.find("on") {
        Some(end_idx) => expression[start + 2..end_idx].trim(),
        None => expression[start + 2..].trim(),
    };

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

fn parse_weekdays(expression: &str) -> Result<Vec<u8>> {
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

fn parse_weeks(expression: &str) -> Result<String> {
    let start = match expression.find("in") {
        Some(start_idx) => start_idx,
        None => return Err(InvalidExpressionError.into()),
    };

    let result = match expression.find("weeks") {
        Some(end_idx) => expression[start + 2..end_idx].trim(),
        None => return Err(InvalidExpressionError.into()),
    };

    match result {
        "even" => Ok(result.to_string()),
        "odd" => Ok(result.to_string()),
        _ => return Err(InvalidExpressionError.into()),
    }
}

fn get_next(expression: &str, base: OffsetDateTime) -> Result<OffsetDateTime> {
    let hours = parse_hours(expression);
    let weekdays = parse_weekdays(expression);
    let weeks = parse_weeks(expression);
    Ok(base)
}

pub fn get_next_date(expression: &str) -> Result<OffsetDateTime> {
    let now = OffsetDateTime::try_now_local()?;
    get_next(expression, now)
}

pub fn get_next_n_dates(_expression: &str, _n: u8) -> Result<Vec<OffsetDateTime>> {
    Ok(vec![])
}
