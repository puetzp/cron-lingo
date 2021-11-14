use crate::error::*;
use crate::types::{ParsedBlock, WeekVariant, WeekdayModifier};
use time::{Time, Weekday};

const TIME_FORMAT_NO_MINUTES: &[time::format_description::FormatItem] =
    time::macros::format_description!("[hour padding:none repr:12] [period case:upper]");
const TIME_FORMAT_WITH_MINUTES: &[time::format_description::FormatItem] =
    time::macros::format_description!("[hour padding:none repr:12]:[minute] [period case:upper]");

pub(crate) fn parse(expression: &str) -> Result<Vec<ParsedBlock>, Error> {
    let chars: Vec<char> = expression.chars().collect();
    let mut position: usize = 0;

    if chars.is_empty() {
        return Err(Error::EmptyExpression);
    }

    let mut tokens = vec![];

    while position < chars.len() {
        tokens.push(match_block(&mut position, &chars)?);
    }

    Ok(tokens)
}

fn match_block(position: &mut usize, chars: &[char]) -> Result<ParsedBlock, Error> {
    eat_keyword("at", position, chars)?;
    eat_whitespace(position, chars)?;
    let times = match_times(position, chars)?;

    let days = if *position < chars.len() {
        if is_block_end(&position, &chars) {
            eat_delimitation(position, chars)?;
            None
        } else {
            Some(match_weekdays(position, chars)?)
        }
    } else {
        None
    };

    let weeks = if *position < chars.len() {
        if is_block_end(&position, &chars) {
            eat_delimitation(position, chars)?;
            None
        } else {
            eat_whitespace(position, chars)?;
            Some(match_week(position, chars)?)
        }
    } else {
        None
    };

    let spec = ParsedBlock { times, days, weeks };

    Ok(spec)
}

fn is_block_end(position: &usize, chars: &[char]) -> bool {
    expect_sequence(", at", &position, &chars) || expect_sequence(" and at", &position, &chars)
}

fn expect_sequence(sequence: &str, position: &usize, chars: &[char]) -> bool {
    let end_pos = *position + sequence.len();
    match chars.get(*position..end_pos) {
        Some(c) => c.iter().collect::<String>().as_str() == sequence,
        None => false,
    }
}

fn eat_delimitation(position: &mut usize, chars: &[char]) -> Result<(), Error> {
    match chars.get(*position) {
        Some(ch) => {
            if *ch == ',' {
                *position += 1;
                eat_whitespace(position, chars)?;
            } else {
                eat_whitespace(position, chars)?;
                eat_keyword("and", position, chars)?;
                eat_whitespace(position, chars)?;
            }
        }
        None => return Err(Error::UnexpectedEndOfInput),
    }

    Ok(())
}

fn eat_keyword(keyword: &str, position: &mut usize, chars: &[char]) -> Result<(), Error> {
    let end_pos = *position + keyword.len();

    let word: String = chars
        .get(*position..end_pos)
        .ok_or(Error::UnexpectedEndOfInput)?
        .iter()
        .collect();

    if word.as_str() == keyword {
        *position = end_pos;
    } else {
        let err = SyntaxError {
            position: *position,
            expected: format!("'{}'", keyword),
        };
        return Err(Error::Syntax(err));
    }

    Ok(())
}

fn eat_modifier(position: &mut usize, chars: &[char]) -> Result<WeekdayModifier, Error> {
    if eat_keyword("1st", position, chars).is_ok() {
        return Ok(WeekdayModifier::First);
    }

    if eat_keyword("first", position, chars).is_ok() {
        return Ok(WeekdayModifier::First);
    }

    if eat_keyword("2nd", position, chars).is_ok() {
        return Ok(WeekdayModifier::Second);
    }

    if eat_keyword("second", position, chars).is_ok() {
        return Ok(WeekdayModifier::Second);
    }

    if eat_keyword("3rd", position, chars).is_ok() {
        return Ok(WeekdayModifier::Third);
    }

    if eat_keyword("third", position, chars).is_ok() {
        return Ok(WeekdayModifier::Third);
    }

    if eat_keyword("4th", position, chars).is_ok() {
        return Ok(WeekdayModifier::Fourth);
    }

    if eat_keyword("fourth", position, chars).is_ok() {
        return Ok(WeekdayModifier::Fourth);
    }

    if eat_keyword("last", position, chars).is_ok() {
        return Ok(WeekdayModifier::Last);
    }

    let err = SyntaxError {
        position: *position,
        expected:
            "one of '1st', 'first', '2nd', 'second', '3rd', 'third', '4th', 'fourth' or 'last'"
                .to_string(),
    };

    Err(Error::Syntax(err))
}

fn eat_weekday(position: &mut usize, chars: &[char], specific: bool) -> Result<Weekday, Error> {
    let day;

    if eat_keyword("Monday", position, chars).is_ok() {
        day = Weekday::Monday;
    } else if eat_keyword("Tuesday", position, chars).is_ok() {
        day = Weekday::Tuesday;
    } else if eat_keyword("Wednesday", position, chars).is_ok() {
        day = Weekday::Wednesday;
    } else if eat_keyword("Thursday", position, chars).is_ok() {
        day = Weekday::Thursday;
    } else if eat_keyword("Friday", position, chars).is_ok() {
        day = Weekday::Friday;
    } else if eat_keyword("Saturday", position, chars).is_ok() {
        day = Weekday::Saturday;
    } else if eat_keyword("Sunday", position, chars).is_ok() {
        day = Weekday::Sunday;
    } else {
        let err = SyntaxError {
            position: *position,
            expected: "one of 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday' or 'Sunday'".to_string()
        };
        return Err(Error::Syntax(err));
    }

    if !specific {
        match chars.get(*position) {
            Some(c) => {
                if *c == 's' {
                    *position += 1;
                    return Ok(day);
                } else {
                    let err = SyntaxError {
                        position: *position,
                        expected: "one of 'Mondays', 'Tuesdays', 'Wednesdays', 'Thursdays', 'Fridays', 'Saturdays' or 'Sundays'".to_string()
                    };
                    return Err(Error::Syntax(err));
                }
            }
            None => return Err(Error::UnexpectedEndOfInput),
        }
    } else {
        return Ok(day);
    }
}

fn eat_whitespace(position: &mut usize, chars: &[char]) -> Result<(), Error> {
    match chars.get(*position) {
        Some(ch) => {
            if ch.is_whitespace() {
                *position += 1;
                Ok(())
            } else {
                let err = SyntaxError {
                    position: *position,
                    expected: "a whitespace".to_string(),
                };
                Err(Error::Syntax(err))
            }
        }
        None => Err(Error::UnexpectedEndOfInput),
    }
}

fn match_times(position: &mut usize, chars: &[char]) -> Result<Vec<Time>, Error> {
    let mut tokens = vec![];

    tokens.push(match_time(position, chars)?);

    // Check for more occurrences of time tokens.
    loop {
        if is_block_end(&position, &chars) {
            break;
        } else {
            match chars.get(*position) {
                Some(ch) => {
                    if *ch == ',' {
                        *position += 1;
                        eat_whitespace(position, chars)?;
                        tokens.push(match_time(position, chars)?);
                        continue;
                    } else if ch.is_whitespace() {
                        if expect_sequence(" and", &position, &chars) {
                            eat_whitespace(position, chars)?;
                            eat_keyword("and", position, chars)?;
                            eat_whitespace(position, chars)?;
                            tokens.push(match_time(position, chars)?);
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        let err = SyntaxError {
                            position: *position,
                            expected: "either ',' or a whitespace".to_string(),
                        };
                        return Err(Error::Syntax(err));
                    }
                }
                None => break,
            }
        }
    }

    Ok(tokens)
}

fn match_time(position: &mut usize, chars: &[char]) -> Result<Time, Error> {
    // First character must be a number.
    let hour = chars
        .get(*position)
        .ok_or_else(|| {
            let err = SyntaxError {
                position: *position,
                expected: "a number between 1 and 12 with optional zero-padding".to_string(),
            };
            Error::Syntax(err)
        })?
        .clone();

    if !hour.is_numeric() {
        let err = SyntaxError {
            position: *position,
            expected: "a number between 1 and 12 with optional zero-padding".to_string(),
        };
        return Err(Error::Syntax(err));
    }

    *position += 1;

    // Next character may be the next part of a 2-digit number, a colon,
    // or a whitespace.
    let next = chars
        .get(*position)
        .ok_or_else(|| {
            let err = SyntaxError {
                position: *position,
                expected: "one of a number between 0 and 2, a colon or a whitespace".to_string(),
            };
            Error::Syntax(err)
        })?
        .clone();

    *position += 1;

    if next.is_whitespace() {
        let end_pos = *position + 2;

        let mut time: String = chars
            .get(*position..end_pos)
            .ok_or_else(|| {
                let err = SyntaxError {
                    position: *position,
                    expected: "either 'AM' or 'PM'".to_string(),
                };
                Error::Syntax(err)
            })?
            .iter()
            .collect();

        *position = end_pos;

        time.insert(0, ' ');
        time.insert(0, hour);

        let parsed = Time::parse(&time, &TIME_FORMAT_NO_MINUTES).map_err(Error::TimeParse)?;

        Ok(parsed)
    } else if next == ':' {
        let mut complete = String::new();
        complete.push(hour);
        complete.push(next);

        let end_pos = *position + 5;

        for c in chars.get(*position..end_pos).ok_or_else(|| {
            let err = SyntaxError {
                position: *position,
                expected: "a number between 00 and 59 followed by either 'AM' or 'PM'".to_string(),
            };
            Error::Syntax(err)
        })? {
            complete.push(*c);
        }

        *position = end_pos;

        let parsed = Time::parse(&complete, &TIME_FORMAT_WITH_MINUTES).map_err(Error::TimeParse)?;

        Ok(parsed)
    } else if next.is_numeric() {
        let mut complete = String::new();
        complete.push(hour);
        complete.push(next);

        let next = chars
            .get(*position)
            .ok_or_else(|| {
                let err = SyntaxError {
                    position: *position,
                    expected: "either a ':' or a whitespace".to_string(),
                };
                Error::Syntax(err)
            })?
            .clone();

        *position += 1;

        if next.is_whitespace() {
            let end_pos = *position + 2;

            let time: String = chars
                .get(*position..end_pos)
                .ok_or_else(|| {
                    let err = SyntaxError {
                        position: *position,
                        expected: "either 'AM' or 'PM'".to_string(),
                    };
                    Error::Syntax(err)
                })?
                .iter()
                .collect();

            *position = end_pos;

            complete.push(' ');
            complete.push_str(&time);

            let parsed =
                Time::parse(&complete, &TIME_FORMAT_NO_MINUTES).map_err(Error::TimeParse)?;

            Ok(parsed)
        } else if next == ':' {
            complete.push(next);

            let end_pos = *position + 5;

            for c in chars.get(*position..end_pos).ok_or_else(|| {
                let err = SyntaxError {
                    position: *position,
                    expected: "a number between 00 and 59 followed by either 'AM' or 'PM'"
                        .to_string(),
                };
                Error::Syntax(err)
            })? {
                complete.push(*c);
            }

            *position = end_pos;

            let parsed =
                Time::parse(&complete, &TIME_FORMAT_WITH_MINUTES).map_err(Error::TimeParse)?;

            Ok(parsed)
        } else {
            let err = SyntaxError {
                position: *position,
                expected: "either ',' or a whitespace".to_string(),
            };
            Err(Error::Syntax(err))
        }
    } else {
        let err = SyntaxError {
            position: *position,
            expected: "one of a number between 0 and 2, ',' or a whitespace".to_string(),
        };
        Err(Error::Syntax(err))
    }
}

fn match_weekdays(
    position: &mut usize,
    chars: &[char],
) -> Result<Vec<(Weekday, Option<WeekdayModifier>)>, Error> {
    let mut tokens = vec![];

    eat_whitespace(position, chars)?;

    let has_braces = match chars.get(*position) {
        Some(c) => {
            if *c == '(' {
                *position += 1;
                true
            } else {
                eat_keyword("on", position, chars)?;
                eat_whitespace(position, chars)?;
                false
            }
        }
        None => {
            let err = SyntaxError {
                position: *position,
                expected: "either '(' or 'on'".to_string(),
            };
            return Err(Error::Syntax(err));
        }
    };

    tokens.push(match_weekday(position, chars)?);

    loop {
        match chars.get(*position) {
            Some(ch) => {
                if *ch == ',' {
                    *position += 1;
                    eat_whitespace(position, chars)?;
                    tokens.push(match_weekday(position, chars)?);
                    continue;
                } else if ch.is_whitespace() {
                    if expect_sequence(" and", &position, &chars) {
                        eat_whitespace(position, chars)?;
                        eat_keyword("and", position, chars)?;
                        eat_whitespace(position, chars)?;
                        tokens.push(match_weekday(position, chars)?);
                        continue;
                    } else {
                        break;
                    }
                } else if *ch == ')' {
                    break;
                } else {
                    let expected = if has_braces {
                        "either ',', ')' or a whitespace".to_string()
                    } else {
                        "either ',' or a whitespace".to_string()
                    };
                    let err = SyntaxError {
                        position: *position,
                        expected,
                    };
                    return Err(Error::Syntax(err));
                }
            }
            None => break,
        }
    }

    if has_braces {
        match chars.get(*position) {
            Some(c) => {
                if *c == ')' {
                    *position += 1;
                } else {
                    let err = SyntaxError {
                        position: *position,
                        expected: "a ')'".to_string(),
                    };
                    return Err(Error::Syntax(err));
                }
            }
            None => {
                let err = SyntaxError {
                    position: *position,
                    expected: "a ')'".to_string(),
                };
                return Err(Error::Syntax(err));
            }
        }
    }

    Ok(tokens)
}

fn match_weekday(
    position: &mut usize,
    chars: &[char],
) -> Result<(Weekday, Option<WeekdayModifier>), Error> {
    let next = chars
        .get(*position)
        .ok_or(Error::UnexpectedEndOfInput)?
        .clone();

    let mut modifier = None;

    if next.is_numeric() {
        modifier = Some(eat_modifier(position, chars)?);
        eat_whitespace(position, chars)?;
    } else if next.is_alphabetic() && next.is_lowercase() {
        if eat_keyword("the", position, chars).is_ok() {
            eat_whitespace(position, chars)?;
        }
        modifier = Some(eat_modifier(position, chars)?);
        eat_whitespace(position, chars)?;
    }

    let day = if modifier.is_some() {
        eat_weekday(position, chars, true)?
    } else {
        eat_weekday(position, chars, false)?
    };

    return Ok((day, modifier));
}

fn match_week(position: &mut usize, chars: &[char]) -> Result<WeekVariant, Error> {
    if eat_keyword("in even weeks", position, chars).is_ok() {
        return Ok(WeekVariant::Even);
    } else if eat_keyword("in odd weeks", position, chars).is_ok() {
        return Ok(WeekVariant::Odd);
    } else {
        let err = SyntaxError {
            position: *position,
            expected: "either 'in even weeks' or 'in odd weeks'".to_string(),
        };
        return Err(Error::Syntax(err));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::time;

    #[test]
    fn test_parse_single_block() {
        let spec = vec![ParsedBlock {
            times: vec![time!(07:30:00), time!(17:00:00), time!(04:00:00)],
            days: Some(vec![
                (Weekday::Monday, None),
                (Weekday::Wednesday, None),
                (Weekday::Friday, Some(WeekdayModifier::Last)),
            ]),
            weeks: Some(WeekVariant::Odd),
        }];
        assert_eq!(
            parse("at 07:30 AM, 5 PM and 4 AM on Mondays and Wednesdays and the last Friday in odd weeks"),
            Ok(spec)
        );
    }

    #[test]
    fn test_parse_multiple_blocks() {
        let spec = vec![
            ParsedBlock {
                times: vec![time!(07:00:00)],
                days: Some(vec![(Weekday::Monday, None)]),
                weeks: None,
            },
            ParsedBlock {
                times: vec![time!(07:00:00)],
                days: Some(vec![(Weekday::Thursday, None)]),
                weeks: Some(WeekVariant::Odd),
            },
        ];
        assert_eq!(
            parse("at 07:00 AM (Mondays) and at 07:00 AM (Thursdays) in odd weeks"),
            Ok(spec)
        );
    }
}
