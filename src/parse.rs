use crate::error::*;
use crate::types::{WeekVariant, WeekdayModifier};
use std::fmt;
use time::Weekday;

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum Token {
    // A time object in 12-hour format.
    Time(time::Time),
    // A specific weekday like "Monday".
    Day((Weekday, Option<WeekdayModifier>)),
    // A modifier narrowing down the ordinal week number, one of "even" or "odd".
    Week(WeekVariant),
}

const TIME_FORMAT_NO_MINUTES: &[time::format_description::FormatItem] =
    time::macros::format_description!("[hour padding:none repr:12] [period case:upper]");
const TIME_FORMAT_WITH_MINUTES: &[time::format_description::FormatItem] =
    time::macros::format_description!("[hour padding:none repr:12]:[minute] [period case:upper]");

pub(crate) fn parse(expression: &str) -> Result<Vec<Vec<Token>>, InvalidExpressionError> {
    let chars: Vec<char> = expression.chars().collect();
    let mut position: usize = 0;

    if chars.is_empty() {
        return Err(InvalidExpressionError::EmptyExpression);
    }

    let tokens = match_blocks(&mut position, &chars)?;

    Ok(tokens)
}

fn match_blocks(
    position: &mut usize,
    chars: &[char],
) -> Result<Vec<Vec<Token>>, InvalidExpressionError> {
    let mut tokens: Vec<Vec<Token>> = Vec::new();

    while *position < chars.len() {
        tokens.push(match_block(position, chars)?);
    }

    Ok(tokens)
}

fn match_block(position: &mut usize, chars: &[char]) -> Result<Vec<Token>, InvalidExpressionError> {
    let mut tokens = Vec::new();

    eat_keyword("at", position, chars)?;
    eat_whitespace(position, chars)?;
    tokens.extend(match_times(position, chars)?);

    if *position < chars.len() {
        if is_block_end(&position, &chars) {
            eat_delimitation(position, chars)?;
            return Ok(tokens);
        } else {
            tokens.extend(match_weekdays(position, chars)?);
        }
    }

    if *position < chars.len() {
        if is_block_end(&position, &chars) {
            eat_delimitation(position, chars)?;
            return Ok(tokens);
        } else {
            eat_whitespace(position, chars)?;
            tokens.push(match_week(position, chars)?);
        }
    }

    Ok(tokens)
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

fn eat_delimitation(position: &mut usize, chars: &[char]) -> Result<(), InvalidExpressionError> {
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
        None => return Err(InvalidExpressionError::Syntax),
    }

    Ok(())
}

fn eat_keyword(
    keyword: &str,
    position: &mut usize,
    chars: &[char],
) -> Result<(), InvalidExpressionError> {
    let end_pos = *position + keyword.len();

    let word: String = chars
        .get(*position..end_pos)
        .ok_or(InvalidExpressionError::Syntax)?
        .iter()
        .collect();

    if word.as_str() == keyword {
        *position = end_pos;
    } else {
        return Err(InvalidExpressionError::Syntax);
    }

    Ok(())
}

fn eat_modifier(
    position: &mut usize,
    chars: &[char],
) -> Result<WeekdayModifier, InvalidExpressionError> {
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

    Err(InvalidExpressionError::Syntax)
}

fn eat_weekday(
    position: &mut usize,
    chars: &[char],
    specific: bool,
) -> Result<Weekday, InvalidExpressionError> {
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
        return Err(InvalidExpressionError::Syntax);
    }

    if !specific {
        match chars.get(*position) {
            Some(c) => {
                if *c == 's' {
                    *position += 1;
                    return Ok(day);
                } else {
                    return Err(InvalidExpressionError::Syntax);
                }
            }
            None => return Err(InvalidExpressionError::Syntax),
        }
    } else {
        return Ok(day);
    }
}

fn eat_whitespace(position: &mut usize, chars: &[char]) -> Result<(), InvalidExpressionError> {
    match chars.get(*position) {
        Some(ch) => {
            if ch.is_whitespace() {
                *position += 1;
                Ok(())
            } else {
                Err(InvalidExpressionError::Syntax)
            }
        }
        None => Err(InvalidExpressionError::Syntax),
    }
}

fn match_times(position: &mut usize, chars: &[char]) -> Result<Vec<Token>, InvalidExpressionError> {
    let mut tokens = Vec::new();

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
                        return Err(InvalidExpressionError::Syntax);
                    }
                }
                None => break,
            }
        }
    }

    Ok(tokens)
}

fn match_time(position: &mut usize, chars: &[char]) -> Result<Token, InvalidExpressionError> {
    // First character must be a number.
    let hour = chars
        .get(*position)
        .ok_or(InvalidExpressionError::Syntax)?
        .clone();

    if hour.is_numeric() {
        *position += 1;

        // Next character may be the next part of a 2-digit number, a colon,
        // or a whitespace.
        let next = chars
            .get(*position)
            .ok_or(InvalidExpressionError::Syntax)?
            .clone();

        *position += 1;

        if next.is_whitespace() {
            let end_pos = *position + 2;

            let mut time: String = chars
                .get(*position..end_pos)
                .ok_or(InvalidExpressionError::Syntax)?
                .iter()
                .collect();

            *position = end_pos;

            time.insert(0, ' ');
            time.insert(0, hour);

            let parsed = time::Time::parse(&time, &TIME_FORMAT_NO_MINUTES)
                .map_err(InvalidExpressionError::TimeParse)?;

            Ok(Token::Time(parsed))
        } else if next == ':' {
            let mut complete = String::new();
            complete.push(hour);
            complete.push(next);

            let end_pos = *position + 5;

            for c in chars
                .get(*position..end_pos)
                .ok_or(InvalidExpressionError::Syntax)?
            {
                complete.push(*c);
            }

            *position = end_pos;

            let parsed = time::Time::parse(&complete, &TIME_FORMAT_WITH_MINUTES)
                .map_err(InvalidExpressionError::TimeParse)?;

            Ok(Token::Time(parsed))
        } else if next.is_numeric() {
            let mut complete = String::new();
            complete.push(hour);
            complete.push(next);

            let end_pos = *position + 6;

            for c in chars
                .get(*position..end_pos)
                .ok_or(InvalidExpressionError::Syntax)?
            {
                complete.push(*c);
            }

            *position = end_pos;

            let parsed = time::Time::parse(&complete, &TIME_FORMAT_WITH_MINUTES)
                .map_err(InvalidExpressionError::TimeParse)?;

            Ok(Token::Time(parsed))
        } else {
            Err(InvalidExpressionError::Syntax)
        }
    } else {
        Err(InvalidExpressionError::Syntax)
    }
}

fn match_weekdays(
    position: &mut usize,
    chars: &[char],
) -> Result<Vec<Token>, InvalidExpressionError> {
    let mut tokens = Vec::new();

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
        None => return Err(InvalidExpressionError::Syntax),
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
                    return Err(InvalidExpressionError::Syntax);
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
                    return Err(InvalidExpressionError::Syntax);
                }
            }
            None => return Err(InvalidExpressionError::Syntax),
        }
    }

    Ok(tokens)
}

fn match_weekday(position: &mut usize, chars: &[char]) -> Result<Token, InvalidExpressionError> {
    let mut next = chars
        .get(*position)
        .ok_or(InvalidExpressionError::Syntax)?
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

    next = chars
        .get(*position)
        .ok_or(InvalidExpressionError::Syntax)?
        .clone();

    if next.is_uppercase() {
        let day = if modifier.is_some() {
            eat_weekday(position, chars, true)?
        } else {
            eat_weekday(position, chars, false)?
        };
        return Ok(Token::Day((day, modifier)));
    }

    Err(InvalidExpressionError::Syntax)
}

fn match_week(position: &mut usize, chars: &[char]) -> Result<Token, InvalidExpressionError> {
    if eat_keyword("in even weeks", position, chars).is_ok() {
        return Ok(Token::Week(WeekVariant::Even));
    } else if eat_keyword("in odd weeks", position, chars).is_ok() {
        return Ok(Token::Week(WeekVariant::Odd));
    } else {
        return Err(InvalidExpressionError::Syntax);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::time;

    #[test]
    fn test_parse_single_block() {
        let tokens = vec![vec![
            Token::Time(time!(07:30:00)),
            Token::Time(time!(17:00:00)),
            Token::Time(time!(04:00:00)),
            Token::Day((Weekday::Monday, None)),
            Token::Day((Weekday::Wednesday, None)),
            Token::Day((Weekday::Friday, Some(WeekdayModifier::Last))),
            Token::Week(WeekVariant::Odd),
        ]];
        assert_eq!(
            parse("at 07:30 AM, 5 PM and 4 AM on Mondays and Wednesdays and the last Friday in odd weeks"),
            Ok(tokens)
        );
    }

    #[test]
    fn test_parse_multiple_blocks() {
        let tokens = vec![
            vec![
                Token::Time(time!(07:00:00)),
                Token::Day((Weekday::Monday, None)),
            ],
            vec![
                Token::Time(time!(07:00:00)),
                Token::Day((Weekday::Thursday, None)),
                Token::Week(WeekVariant::Odd),
            ],
        ];
        assert_eq!(
            parse("at 07:00 AM (Mondays) and at 07:00 AM (Thursdays) in odd weeks"),
            Ok(tokens)
        );
    }
}
