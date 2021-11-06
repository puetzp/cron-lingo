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
    let mut chars: Vec<char> = expression.chars().collect();

    if chars.is_empty() {
        return Err(InvalidExpressionError::EmptyExpression);
    }

    let tokens = match_blocks(&mut chars)?;

    Ok(tokens)
}

fn match_blocks(chars: &mut Vec<char>) -> Result<Vec<Vec<Token>>, InvalidExpressionError> {
    let mut tokens: Vec<Vec<Token>> = Vec::new();

    while !chars.is_empty() {
        tokens.push(match_block(chars)?);
    }

    Ok(tokens)
}

fn match_block(chars: &mut Vec<char>) -> Result<Vec<Token>, InvalidExpressionError> {
    let mut tokens = Vec::new();

    eat_keyword("at", chars)?;
    eat_whitespace(chars)?;
    tokens.extend(match_times(chars)?);

    if !chars.is_empty() {
        if is_block_end(&chars) {
            eat_delimitation(chars)?;
            return Ok(tokens);
        } else {
            tokens.extend(match_weekdays(chars)?);
        }
    }

    if !chars.is_empty() {
        if is_block_end(&chars) {
            eat_delimitation(chars)?;
            return Ok(tokens);
        } else {
            eat_whitespace(chars)?;
            tokens.push(match_week(chars)?);
        }
    }

    Ok(tokens)
}

fn is_block_end(chars: &[char]) -> bool {
    expect_sequence(", at", &chars) || expect_sequence(" and at", &chars)
}

fn expect_sequence(sequence: &str, chars: &[char]) -> bool {
    match chars.get(0..sequence.len()) {
        Some(c) => c.iter().collect::<String>().as_str() == sequence,
        None => false,
    }
}

fn eat_delimitation(chars: &mut Vec<char>) -> Result<(), InvalidExpressionError> {
    match chars.get(0) {
        Some(ch) => {
            if *ch == ',' {
                chars.remove(0);
                eat_whitespace(chars)?;
            } else {
                eat_whitespace(chars)?;
                eat_keyword("and", chars)?;
                eat_whitespace(chars)?;
            }
        }
        None => return Err(InvalidExpressionError::Syntax),
    }

    Ok(())
}

fn eat_keyword(keyword: &str, chars: &mut Vec<char>) -> Result<(), InvalidExpressionError> {
    let length = keyword.len();

    let word: String = chars
        .get(0..length)
        .ok_or(InvalidExpressionError::Syntax)?
        .iter()
        .collect();

    if word.as_str() == keyword {
        *chars = chars[length..].to_vec();
    } else {
        return Err(InvalidExpressionError::Syntax);
    }

    Ok(())
}

fn eat_modifier(chars: &mut Vec<char>) -> Result<WeekdayModifier, InvalidExpressionError> {
    if eat_keyword("1st", chars).is_ok() {
        return Ok(WeekdayModifier::First);
    }

    if eat_keyword("first", chars).is_ok() {
        return Ok(WeekdayModifier::First);
    }

    if eat_keyword("2nd", chars).is_ok() {
        return Ok(WeekdayModifier::Second);
    }

    if eat_keyword("second", chars).is_ok() {
        return Ok(WeekdayModifier::Second);
    }

    if eat_keyword("3rd", chars).is_ok() {
        return Ok(WeekdayModifier::Third);
    }

    if eat_keyword("third", chars).is_ok() {
        return Ok(WeekdayModifier::Third);
    }

    if eat_keyword("4th", chars).is_ok() {
        return Ok(WeekdayModifier::Fourth);
    }

    if eat_keyword("fourth", chars).is_ok() {
        return Ok(WeekdayModifier::Fourth);
    }

    if eat_keyword("last", chars).is_ok() {
        return Ok(WeekdayModifier::Last);
    }

    Err(InvalidExpressionError::Syntax)
}

fn eat_weekday(chars: &mut Vec<char>, specific: bool) -> Result<Weekday, InvalidExpressionError> {
    let mut day;

    if eat_keyword("Monday", chars).is_ok() {
        day = Weekday::Monday;
    } else if eat_keyword("Tuesday", chars).is_ok() {
        day = Weekday::Tuesday;
    } else if eat_keyword("Wednesday", chars).is_ok() {
        day = Weekday::Wednesday;
    } else if eat_keyword("Thursday", chars).is_ok() {
        day = Weekday::Thursday;
    } else if eat_keyword("Friday", chars).is_ok() {
        day = Weekday::Friday;
    } else if eat_keyword("Saturday", chars).is_ok() {
        day = Weekday::Saturday;
    } else if eat_keyword("Sunday", chars).is_ok() {
        day = Weekday::Sunday;
    } else {
        return Err(InvalidExpressionError::Syntax);
    }

    if !specific {
        match chars.get(0) {
            Some(c) => {
                if *c == 's' {
                    chars.remove(0);
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

fn eat_whitespace(chars: &mut Vec<char>) -> Result<(), InvalidExpressionError> {
    match chars.get(0) {
        Some(ch) => {
            if ch.is_whitespace() {
                chars.remove(0);
                Ok(())
            } else {
                Err(InvalidExpressionError::Syntax)
            }
        }
        None => Err(InvalidExpressionError::Syntax),
    }
}

fn match_times(chars: &mut Vec<char>) -> Result<Vec<Token>, InvalidExpressionError> {
    let mut tokens = Vec::new();

    tokens.push(match_time(chars)?);

    // Check for more occurrences of time tokens.
    loop {
        if is_block_end(&chars) {
            break;
        } else {
            match chars.get(0) {
                Some(ch) => {
                    if *ch == ',' {
                        chars.remove(0);
                        eat_whitespace(chars)?;
                        tokens.push(match_time(chars)?);
                        continue;
                    } else if ch.is_whitespace() {
                        if expect_sequence(" and", &chars) {
                            eat_whitespace(chars)?;
                            eat_keyword("and", chars)?;
                            eat_whitespace(chars)?;
                            tokens.push(match_time(chars)?);
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

fn match_time(chars: &mut Vec<char>) -> Result<Token, InvalidExpressionError> {
    // First character must be a number.
    let hour = chars.get(0).ok_or(InvalidExpressionError::Syntax)?.clone();

    if hour.is_numeric() {
        chars.remove(0);

        // Next character may be the next part of a 2-digit number, a colon,
        // or a whitespace.
        let next = chars.get(0).ok_or(InvalidExpressionError::Syntax)?.clone();
        chars.remove(0);

        if next.is_whitespace() {
            let mut time: String = chars
                .get(0..2)
                .ok_or(InvalidExpressionError::Syntax)?
                .iter()
                .collect();

            chars.remove(0);
            chars.remove(0);

            time.insert(0, ' ');
            time.insert(0, hour);

            let parsed = time::Time::parse(&time, &TIME_FORMAT_NO_MINUTES)
                .map_err(InvalidExpressionError::TimeParse)?;

            Ok(Token::Time(parsed))
        } else if next == ':' {
            let mut complete = String::new();
            complete.push(hour);
            complete.push(next);

            for c in chars.get(0..5).ok_or(InvalidExpressionError::Syntax)? {
                complete.push(*c);
            }

            *chars = chars[5..].to_vec();

            let parsed = time::Time::parse(&complete, &TIME_FORMAT_WITH_MINUTES)
                .map_err(InvalidExpressionError::TimeParse)?;

            Ok(Token::Time(parsed))
        } else if next.is_numeric() {
            let mut complete = String::new();
            complete.push(hour);
            complete.push(next);

            for c in chars.get(0..6).ok_or(InvalidExpressionError::Syntax)? {
                complete.push(*c);
            }

            *chars = chars[6..].to_vec();

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

fn match_weekdays(chars: &mut Vec<char>) -> Result<Vec<Token>, InvalidExpressionError> {
    let mut tokens = Vec::new();

    eat_whitespace(chars)?;

    let has_braces = match chars.get(0) {
        Some(c) => {
            if *c == '(' {
                chars.remove(0);
                true
            } else {
                eat_keyword("on", chars)?;
                eat_whitespace(chars)?;
                false
            }
        }
        None => return Err(InvalidExpressionError::Syntax),
    };

    tokens.push(match_weekday(chars)?);

    loop {
        match chars.get(0) {
            Some(ch) => {
                if *ch == ',' {
                    chars.remove(0);
                    eat_whitespace(chars)?;
                    tokens.push(match_weekday(chars)?);
                    continue;
                } else if ch.is_whitespace() {
                    if expect_sequence(" and", &chars) {
                        eat_whitespace(chars)?;
                        eat_keyword("and", chars)?;
                        eat_whitespace(chars)?;
                        tokens.push(match_weekday(chars)?);
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
        match chars.get(0) {
            Some(c) => {
                if *c == ')' {
                    chars.remove(0);
                } else {
                    return Err(InvalidExpressionError::Syntax);
                }
            }
            None => return Err(InvalidExpressionError::Syntax),
        }
    }

    Ok(tokens)
}

fn match_weekday(chars: &mut Vec<char>) -> Result<Token, InvalidExpressionError> {
    let mut next = chars.get(0).ok_or(InvalidExpressionError::Syntax)?.clone();
    let mut modifier = None;

    if next.is_numeric() {
        modifier = Some(eat_modifier(chars)?);
        eat_whitespace(chars)?;
    } else if next.is_alphabetic() && next.is_lowercase() {
        if eat_keyword("the", chars).is_ok() {
            eat_whitespace(chars)?;
        }
        modifier = Some(eat_modifier(chars)?);
        eat_whitespace(chars)?;
    }

    next = chars.get(0).ok_or(InvalidExpressionError::Syntax)?.clone();
    if next.is_uppercase() {
        let day = if modifier.is_some() {
            eat_weekday(chars, true)?
        } else {
            eat_weekday(chars, false)?
        };
        return Ok(Token::Day((day, modifier)));
    }

    Err(InvalidExpressionError::Syntax)
}

fn match_week(chars: &mut Vec<char>) -> Result<Token, InvalidExpressionError> {
    if eat_keyword("in even weeks", chars).is_ok() {
        return Ok(Token::Week(WeekVariant::Even));
    } else if eat_keyword("in odd weeks", chars).is_ok() {
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
