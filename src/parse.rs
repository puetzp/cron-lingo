use crate::error::*;
use std::fmt;

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum Token {
    // Literal "weeks".
    Weeks,
    // Literal "and" or ",".
    Delimiter,
    // Whitespace is actually a token to be matched as we are being picky and do not allow
    // arbitrary amounts of whitespace in an expression (also, it helps with control flow).
    Whitespace,
    // A time object in 12-hour format.
    Time(time::Time),
    // Opening brace "(".
    OBrace,
    // Closing brace ")".
    CBrace,
    // A specific weekday like "Monday".
    SWeekday,
    // A general weekday as in "Mondays"
    GWeekday,
    // A modifier hinting to a specific weekday in a month period like "first" or "1st".
    WeekdayMod,
    // A modifier narrowing down the ordinal week number, one of "even" or "odd".
    WeekMod,
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

    Ok(tokens)
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

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::time;

    #[test]
    fn test_parse() {
        let tokens = vec![vec![Token::Time(time!(07:30:00))]];
        assert_eq!(parse("at 07:30 AM"), Ok(tokens));
    }
}
