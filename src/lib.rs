use std::error::Error;
use std::fmt;
use time::OffsetDateTime;

#[cfg(test)]
mod tests {
    use super::parse_section;
    use super::Component;
    use super::InvalidExpressionError;

    #[test]
    fn test_parse_section_hours1() {
        let expression = "at 6, 7, 8 and 14 o'clock on Monday, Thursday and Saturday in even weeks";
        let result = vec![6, 7, 8, 14];
        assert_eq!(parse_section(expression, Component::Hours).unwrap(), result);
    }

    #[test]
    fn test_parse_section_hours2() {
        let expression = "at 6, 15 o'clock on Friday";
        let result = vec![6, 15];
        assert_eq!(parse_section(expression, Component::Hours).unwrap(), result);
    }

    #[test]
    fn test_parse_section_hours_error1() {
        let expression = "at 6, 15, 24 o'clock on Friday";
        assert_eq!(
            *parse_section(expression, Component::Hours)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_section_hours_error2() {
        let expression = "at 6, 15, 17 18 o'clock on Monday";
        assert_eq!(
            *parse_section(expression, Component::Hours)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_section_weekdays1() {
        let expression = "at 6 o'clock on Sunday, Monday and Thursday in odd weeks";
        let result = vec![0, 1, 4];
        assert_eq!(
            parse_section(expression, Component::Weekdays).unwrap(),
            result
        );
    }

    #[test]
    fn test_parse_section_weekdays2() {
        let expression = "at 13 o'clock on Monday, Friday";
        let result = vec![1, 5];
        assert_eq!(
            parse_section(expression, Component::Weekdays).unwrap(),
            result
        );
    }

    #[test]
    fn test_parse_section_weekdays_error1() {
        let expression = "at 16 o'clock on Monday and Thursd";
        assert_eq!(
            *parse_section(expression, Component::Weekdays)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_section_weekdays_error2() {
        let expression = "at 12 o'clock on Tuesday Saturday";
        assert_eq!(
            *parse_section(expression, Component::Weekdays)
                .unwrap_err()
                .downcast::<InvalidExpressionError>()
                .unwrap(),
            InvalidExpressionError
        );
    }

    #[test]
    fn test_parse_section_weeks() {
        let expression = "at 6 o'clock on Sunday, Monday and Thursday in odd weeks";
        let result = vec![1];
        assert_eq!(parse_section(expression, Component::Weeks).unwrap(), result);
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

enum Component {
    Hours,
    Weekdays,
    Weeks,
}

fn parse_section(expression: &str, component: Component) -> Result<Vec<u8>> {
    let comp_prefix = match component {
        Component::Hours => "at",
        Component::Weekdays => "on",
        Component::Weeks => "in",
    };

    let comp_suffix = match component {
        Component::Hours => "on",
        Component::Weekdays => "in",
        Component::Weeks => "weeks",
    };

    let start = match expression.find(comp_prefix) {
        Some(start_idx) => start_idx,
        None => return Err(InvalidExpressionError.into()),
    };

    let mut section = match expression.find(comp_suffix) {
        Some(end_idx) => expression[start + 2..end_idx].trim(),
        None => expression[start + 2..].trim(),
    };

    section = match component {
        Component::Hours => match section.strip_suffix("o'clock") {
            Some(stripped) => stripped,
            None => return Err(InvalidExpressionError.into()),
        },
        _ => section,
    };

    let section = section.replace("and", ",");

    let mut result = Vec::new();

    for mut item in section.split(",") {
        item = item.trim();

        match component {
            Component::Hours => match item.parse::<u8>() {
                Ok(num) => {
                    if num < 24 {
                        result.push(num);
                    } else {
                        return Err(InvalidExpressionError.into());
                    }
                }
                Err(_) => return Err(InvalidExpressionError.into()),
            },
            Component::Weekdays => match item {
                "Sunday" => result.push(0),
                "Monday" => result.push(1),
                "Tuesday" => result.push(2),
                "Wednesday" => result.push(3),
                "Thursday" => result.push(4),
                "Friday" => result.push(5),
                "Saturday" => result.push(6),
                _ => return Err(InvalidExpressionError.into()),
            },
            Component::Weeks => match item {
                "even" => result.push(0),
                "odd" => result.push(1),
                _ => return Err(InvalidExpressionError.into()),
            },
        }
    }

    Ok(result)
}

fn get_next(expression: &str, base: OffsetDateTime) -> Result<OffsetDateTime> {
    let hours = parse_section(expression, Component::Hours);
    let weekdays = parse_section(expression, Component::Weekdays);
    let weeks = parse_section(expression, Component::Weeks);
    Ok(base)
}

pub fn get_next_date(expression: &str) -> Result<OffsetDateTime> {
    let now = OffsetDateTime::try_now_local()?;
    get_next(expression, now)
}

pub fn get_next_n_dates(_expression: &str, _n: u8) -> Result<Vec<OffsetDateTime>> {
    Ok(vec![])
}
