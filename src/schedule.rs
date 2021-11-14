use crate::error::*;
use crate::parse::parse;
use crate::types::*;
use std::iter::Iterator;
use std::str::FromStr;
use time::{Duration, OffsetDateTime, PrimitiveDateTime};

/// A schedule that is built from an expression and can be iterated
/// in order to compute the next date(s) that match the specification. By
/// default the computation is based on the current system time, meaning
/// the iterator will never return a date in the past.
///
/// The expression must adhere to a specific syntax. See the module-level
/// documentation for the full range of possibilities.
#[derive(Debug, PartialEq, Clone)]
pub struct Schedule {
    base: OffsetDateTime,
    specs: Vec<ParsedBlock>,
}

impl Schedule {
    #[allow(dead_code)]
    pub fn iter(&self) -> ScheduleIter {
        ScheduleIter {
            schedule: self.clone(),
            current: self.base,
            skip_outdated: true,
        }
    }
}

impl FromStr for Schedule {
    type Err = Error;

    /// Attempt to create a new `Schedule` object from an expression.
    ///
    /// ```rust
    /// use cron_lingo::Schedule;
    /// use std::str::FromStr;
    ///
    /// let expr = "at 6 AM on Mondays and Thursdays and at 6 PM on Sundays in even weeks";
    /// assert!(Schedule::from_str(expr).is_ok());
    /// ```
    fn from_str(expression: &str) -> Result<Self, Self::Err> {
        let tt = Schedule {
            base: OffsetDateTime::now_local().map_err(Error::IndeterminateOffset)?,
            specs: parse(expression)?,
        };
        Ok(tt)
    }
}

/// A wrapper around `Schedule` that keeps track of state during iteration.
#[derive(Clone)]
pub struct ScheduleIter {
    schedule: Schedule,
    current: OffsetDateTime,
    skip_outdated: bool,
}

impl ScheduleIter {
    /// By default the `next` method will not return a date that is
    /// in the past but compute the next future date based on the
    /// current local time instead. This method allows to change the
    /// iterators default behaviour.
    pub fn skip_outdated(mut self, skip: bool) -> ScheduleIter {
        self.skip_outdated = skip;
        self
    }
}

impl Iterator for ScheduleIter {
    type Item = OffsetDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        if self.skip_outdated {
            let now = OffsetDateTime::now_local().unwrap();

            if now > self.current {
                self.current = now;
            }
        }

        // Create every possible combination of dates for each
        // ParsedBlock and add them to a vector.
        let mut candidates: Vec<OffsetDateTime> = vec![];

        for spec in self.schedule.specs.clone() {
            candidates.append(&mut compute_dates(self.current, spec));
        }

        // Iterate the vector of dates and find the next date
        // by subtracting the current date from each element
        // in the vector. Return the date that results in the
        // lowest delta.
        let next_date = candidates
            .iter()
            .min_by_key(|d| **d - self.current)
            .unwrap();

        self.current = *next_date;

        Some(*next_date)
    }
}

// Returns a selection of possible next dates according to the rules in a ParsedBlock.
fn compute_dates(base: OffsetDateTime, spec: ParsedBlock) -> Vec<OffsetDateTime> {
    let mut candidates = vec![];
    let today = base.date();
    let offset = base.offset();

    // For each specified time ...
    for time in spec.times {
        // ... create an OffsetDateTime object for each upcoming weekday ...
        for i in 0..=6 {
            let mut date =
                PrimitiveDateTime::new(today + Duration::days(i), time).assume_offset(offset);

            if date <= base {
                date += Duration::weeks(1);
            }

            candidates.push(date);
        }
    }

    // ... remove all objects that match none of the desired weekdays (if any)
    // and increment the remaining dates according to the optional WeekdayModifier
    // and WeekVariant.
    if let Some(ref days) = spec.days {
        let weeks = spec.weeks;

        candidates = candidates
            .into_iter()
            .filter(|c| days.iter().any(|x| x.0 == c.weekday()))
            .collect();

        for candidate in &mut candidates {
            let day_modifier = days.iter().find(|x| x.0 == candidate.weekday()).unwrap().1;

            while !check_date_validity(candidate, day_modifier, weeks) {
                *candidate += Duration::weeks(1);
            }
        }
    }

    // ... and return the filtered date candidates of this ParsedBlock.
    candidates
}

// Takes a date and checks its bounds according to optional WeekdayModifiers
// and/or WeekVariants. Returns false if the date does not match the specified rules.
fn check_date_validity(
    date: &OffsetDateTime,
    weekday_mod: Option<WeekdayModifier>,
    week_mod: Option<WeekVariant>,
) -> bool {
    let is_correct_day = match weekday_mod {
        Some(modifier) => {
            let day = date.day();

            match modifier {
                WeekdayModifier::First => day <= 7,
                WeekdayModifier::Second => day > 7 && day <= 14,
                WeekdayModifier::Third => day > 14 && day <= 21,
                WeekdayModifier::Fourth => day > 21 && day <= 28,
                WeekdayModifier::Last => date.month() != (*date + Duration::weeks(1)).month(),
            }
        }
        None => true,
    };

    let is_correct_week = match week_mod {
        Some(modifier) => {
            let week = date.date().iso_week();

            match modifier {
                WeekVariant::Even => week % 2 == 0,
                WeekVariant::Odd => week % 2 != 0,
            }
        }
        None => true,
    };

    is_correct_day && is_correct_week
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::{datetime, time};
    use time::Weekday;

    /*
    #[test]
    fn test_empty_expression() {
        let result = Schedule::from_str("").unwrap_err();
        assert_eq!(result, Error::EmptyExpression);
    }

    #[test]
    fn test_split_expression() {
        let expression = "at 4 PM on Mondays, at 6 PM on Thursdays and at 3 AM";
        let result = vec!["at 4 PM on Mondays", "6 PM on Thursdays", "3 AM"];
        assert_eq!(split_expression(expression), result);
    }

    #[test]
    fn test_split_block() {
        let block = "at 4 PM and 6 PM on Mondays and Tuesdays";
        let result = ("4 PM and 6 PM", Some("Mondays and Tuesdays"), None);
        assert_eq!(split_block(block).unwrap(), result);
    }

    #[test]
    fn test_split_block_with_week_mod() {
        let block = "at 4 PM and 6 PM on Mondays and Tuesdays in odd weeks";
        let result = (
            "4 PM and 6 PM",
            Some("Mondays and Tuesdays"),
            Some("in odd weeks"),
        );
        assert_eq!(split_block(block).unwrap(), result);
    }

    #[test]
    fn test_split_block_for_error() {
        let block = "at 4 PM and 6 PM on Mondays and on Tuesdays";
        assert_eq!(
            split_block(block).unwrap_err(),
            Error::Syntax
        );
    }

    #[test]
    fn test_parse_times() {
        let expression = "1 AM, 5 AM, 4 PM, 5 PM and 6 PM";
        let result = vec![
            time!(01:00:00),
            time!(05:00:00),
            time!(16:00:00),
            time!(17:00:00),
            time!(18:00:00),
        ];
        assert_eq!(parse_times(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_times_every_hour() {
        let expression = "every full hour";
        let result = vec![
            time!(00:00:00),
            time!(01:00:00),
            time!(02:00:00),
            time!(03:00:00),
            time!(04:00:00),
            time!(05:00:00),
            time!(06:00:00),
            time!(07:00:00),
            time!(08:00:00),
            time!(09:00:00),
            time!(10:00:00),
            time!(11:00:00),
            time!(12:00:00),
            time!(13:00:00),
            time!(14:00:00),
            time!(15:00:00),
            time!(16:00:00),
            time!(17:00:00),
            time!(18:00:00),
            time!(19:00:00),
            time!(20:00:00),
            time!(21:00:00),
            time!(22:00:00),
            time!(23:00:00),
        ];
        assert_eq!(parse_times(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_times_for_parse_error() {
        let expression = "1 AM and 5:30";
        assert!(parse_times(expression).is_err());
    }

    #[test]
    fn test_parse_times_for_duplicate_error() {
        let expression = "1 AM and 1 AM and 5 PM";
        assert_eq!(
            parse_times(expression).unwrap_err(),
            Error::DuplicateInput
        );
    }

    #[test]
    fn test_parse_days() {
        let expression = "Mondays, Tuesdays and Thursdays";
        let result = vec![
            (Weekday::Monday, None),
            (Weekday::Tuesday, None),
            (Weekday::Thursday, None),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_days_for_duplicate_error() {
        let expression = "Mondays, Mondays and Thursdays";
        assert_eq!(
            parse_days(expression).unwrap_err(),
            Error::DuplicateInput
        );
    }

    #[test]
    fn test_parse_days_for_duplicate_error_with_modifiers() {
        let expression = "the 1st Monday, Mondays and Thursdays";
        assert_eq!(
            parse_days(expression).unwrap_err(),
            Error::IllogicalWeekdayCombination
        );
    }

    #[test]
    fn test_parse_days_with_modifiers_1() {
        let expression = "the first Monday, Tuesdays and the 4th Thursday";
        let result = vec![
            (Weekday::Monday, Some(WeekdayModifier::First)),
            (Weekday::Tuesday, None),
            (Weekday::Thursday, Some(WeekdayModifier::Fourth)),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_days_with_modifiers_2() {
        let expression = "the first Monday and the 3rd Monday";
        let result = vec![
            (Weekday::Monday, Some(WeekdayModifier::First)),
            (Weekday::Monday, Some(WeekdayModifier::Third)),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_days_with_modifiers_3() {
        let expression = "the last Monday and the 2nd Monday";
        let result = vec![
            (Weekday::Monday, Some(WeekdayModifier::Last)),
            (Weekday::Monday, Some(WeekdayModifier::Second)),
        ];
        assert_eq!(parse_days(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_1() {
        let expression = "at 5 PM (Mondays and Thursdays) in odd weeks";
        let result = ParsedBlock {
            times: vec![time!(17:00:00)],
            days: Some(vec![(Weekday::Monday, None), (Weekday::Thursday, None)]),
            weeks: Some(WeekVariant::Odd),
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_2() {
        let expression = "at 5 AM and 6:30 PM (first Monday and Thursdays)";
        let result = ParsedBlock {
            times: vec![time!(05:00:00), time!(18:30:00)],
            days: Some(vec![
                (Weekday::Monday, Some(WeekdayModifier::First)),
                (Weekday::Thursday, None),
            ]),
            weeks: None,
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_3() {
        let expression = "at 6:30 AM and 6:30 PM on Mondays and Fridays in even weeks";
        let result = ParsedBlock {
            times: vec![time!(06:30:00), time!(18:30:00)],
            days: Some(vec![(Weekday::Monday, None), (Weekday::Friday, None)]),
            weeks: Some(WeekVariant::Even),
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }

    #[test]
    fn test_parse_block_4() {
        let expression = "at every full hour on Mondays";
        assert!(parse_block(expression).is_ok());
    }

    #[test]
    fn test_parse_block_5() {
        let expression = "at 6 PM in even weeks";
        let result = ParsedBlock {
            times: vec![time!(18:00:00)],
            days: None,
            weeks: Some(WeekVariant::Even),
        };
        assert_eq!(parse_block(expression).unwrap(), result);
    }
     */

    #[test]
    fn test_compute_dates_1() {
        let base = datetime!(2021-06-04 13:38:00 UTC);
        let spec = ParsedBlock {
            times: vec![time!(12:00:00), time!(18:00:00)],
            days: None,
            weeks: None,
        };
        let result = vec![
            datetime!(2021-06-11 12:00:00 UTC),
            datetime!(2021-06-05 12:00:00 UTC),
            datetime!(2021-06-06 12:00:00 UTC),
            datetime!(2021-06-07 12:00:00 UTC),
            datetime!(2021-06-08 12:00:00 UTC),
            datetime!(2021-06-09 12:00:00 UTC),
            datetime!(2021-06-10 12:00:00 UTC),
            datetime!(2021-06-04 18:00:00 UTC),
            datetime!(2021-06-05 18:00:00 UTC),
            datetime!(2021-06-06 18:00:00 UTC),
            datetime!(2021-06-07 18:00:00 UTC),
            datetime!(2021-06-08 18:00:00 UTC),
            datetime!(2021-06-09 18:00:00 UTC),
            datetime!(2021-06-10 18:00:00 UTC),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_compute_dates_2() {
        let base = datetime!(2021-06-04 13:38:00 UTC);
        let spec = ParsedBlock {
            times: vec![time!(18:00:00)],
            days: Some(vec![(Weekday::Monday, None), (Weekday::Thursday, None)]),
            weeks: None,
        };
        let result = vec![
            datetime!(2021-06-07 18:00:00 UTC),
            datetime!(2021-06-10 18:00:00 UTC),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_compute_dates_3() {
        let base = datetime!(2021-06-04 13:38:00 UTC);
        let spec = ParsedBlock {
            times: vec![time!(18:00:00)],
            days: Some(vec![
                (Weekday::Monday, Some(WeekdayModifier::Second)),
                (Weekday::Thursday, None),
            ]),
            weeks: None,
        };
        let result = vec![
            datetime!(2021-06-14 18:00:00 UTC),
            datetime!(2021-06-10 18:00:00 UTC),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_compute_dates_4() {
        let base = datetime!(2021-06-04 13:38:00 UTC);
        let spec = ParsedBlock {
            times: vec![time!(12:00:00), time!(18:00:00)],
            days: Some(vec![
                (Weekday::Friday, Some(WeekdayModifier::First)),
                (Weekday::Thursday, None),
            ]),
            weeks: None,
        };
        let result = vec![
            datetime!(2021-07-02 12:00:00 UTC),
            datetime!(2021-06-10 12:00:00 UTC),
            datetime!(2021-06-04 18:00:00 UTC),
            datetime!(2021-06-10 18:00:00 UTC),
        ];
        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_compute_dates_5() {
        let base = datetime!(2021-06-12 13:38:00 UTC);
        let spec = ParsedBlock {
            times: vec![time!(06:00:00), time!(12:00:00), time!(18:00:00)],
            days: Some(vec![
                (Weekday::Friday, Some(WeekdayModifier::First)),
                (Weekday::Thursday, None),
                (Weekday::Monday, Some(WeekdayModifier::Third)),
            ]),
            weeks: None,
        };
        let result = vec![
            datetime!(2021-06-21 06:00:00 UTC),
            datetime!(2021-06-17 06:00:00 UTC),
            datetime!(2021-07-02 06:00:00 UTC),
            datetime!(2021-06-21 12:00:00 UTC),
            datetime!(2021-06-17 12:00:00 UTC),
            datetime!(2021-07-02 12:00:00 UTC),
            datetime!(2021-06-21 18:00:00 UTC),
            datetime!(2021-06-17 18:00:00 UTC),
            datetime!(2021-07-02 18:00:00 UTC),
        ];

        assert_eq!(compute_dates(base, spec), result);
    }

    #[test]
    fn test_schedule_iteration_1() {
        let schedule = Schedule {
            base: datetime!(2021-06-09 13:00:00 UTC),
            specs: vec![ParsedBlock {
                times: vec![time!(01:00:00)],
                days: None,
                weeks: None,
            }],
        };

        let result = vec![
            datetime!(2021-06-10 01:00:00 UTC),
            datetime!(2021-06-11 01:00:00 UTC),
            datetime!(2021-06-12 01:00:00 UTC),
        ];

        assert_eq!(
            schedule
                .iter()
                .skip_outdated(false)
                .take(3)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_2() {
        let schedule = Schedule {
            base: datetime!(2021-06-09 13:00:00 UTC),
            specs: vec![ParsedBlock {
                times: vec![time!(13:00:00)],
                days: Some(vec![(Weekday::Monday, None)]),
                weeks: None,
            }],
        };

        let result = vec![
            datetime!(2021-06-14 13:00:00 UTC),
            datetime!(2021-06-21 13:00:00 UTC),
            datetime!(2021-06-28 13:00:00 UTC),
        ];

        assert_eq!(
            schedule
                .iter()
                .skip_outdated(false)
                .take(3)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_3() {
        let schedule = Schedule {
            base: datetime!(2021-06-09 13:00:00 UTC),
            specs: vec![ParsedBlock {
                times: vec![time!(06:00:00), time!(13:00:00)],
                days: Some(vec![
                    (Weekday::Monday, Some(WeekdayModifier::Third)),
                    (Weekday::Thursday, None),
                ]),
                weeks: None,
            }],
        };

        let result = vec![
            datetime!(2021-06-10 06:00:00 UTC),
            datetime!(2021-06-10 13:00:00 UTC),
            datetime!(2021-06-17 06:00:00 UTC),
            datetime!(2021-06-17 13:00:00 UTC),
            datetime!(2021-06-21 06:00:00 UTC),
            datetime!(2021-06-21 13:00:00 UTC),
            datetime!(2021-06-24 06:00:00 UTC),
            datetime!(2021-06-24 13:00:00 UTC),
        ];

        assert_eq!(
            schedule
                .iter()
                .skip_outdated(false)
                .take(8)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_4() {
        let schedule = Schedule {
            base: datetime!(2021-06-09 13:00:00 UTC),
            specs: vec![
                ParsedBlock {
                    times: vec![time!(06:00:00), time!(13:00:00)],
                    days: Some(vec![
                        (Weekday::Monday, Some(WeekdayModifier::Third)),
                        (Weekday::Thursday, None),
                    ]),
                    weeks: None,
                },
                ParsedBlock {
                    times: vec![time!(18:00:00)],
                    days: Some(vec![(Weekday::Saturday, Some(WeekdayModifier::Fourth))]),
                    weeks: Some(WeekVariant::Odd),
                },
            ],
        };

        let result = vec![
            datetime!(2021-06-10 06:00:00 UTC),
            datetime!(2021-06-10 13:00:00 UTC),
            datetime!(2021-06-17 06:00:00 UTC),
            datetime!(2021-06-17 13:00:00 UTC),
            datetime!(2021-06-21 06:00:00 UTC),
            datetime!(2021-06-21 13:00:00 UTC),
            datetime!(2021-06-24 06:00:00 UTC),
            datetime!(2021-06-24 13:00:00 UTC),
            datetime!(2021-06-26 18:00:00 UTC),
            datetime!(2021-07-01 06:00:00 UTC),
            datetime!(2021-07-01 13:00:00 UTC),
        ];

        assert_eq!(
            schedule
                .iter()
                .skip_outdated(false)
                .take(11)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_5() {
        let schedule = Schedule {
            base: datetime!(2021-06-18 13:00:00 UTC),
            specs: vec![
                ParsedBlock {
                    times: vec![time!(06:00:00), time!(18:00:00)],
                    days: Some(vec![
                        (Weekday::Monday, Some(WeekdayModifier::Last)),
                        (Weekday::Thursday, None),
                    ]),
                    weeks: None,
                },
                ParsedBlock {
                    times: vec![time!(18:00:00)],
                    days: Some(vec![(Weekday::Saturday, Some(WeekdayModifier::Fourth))]),
                    weeks: None,
                },
            ],
        };

        let result = vec![
            datetime!(2021-06-24 06:00:00 UTC),
            datetime!(2021-06-24 18:00:00 UTC),
            datetime!(2021-06-26 18:00:00 UTC),
            datetime!(2021-06-28 06:00:00 UTC),
            datetime!(2021-06-28 18:00:00 UTC),
            datetime!(2021-07-01 06:00:00 UTC),
            datetime!(2021-07-01 18:00:00 UTC),
            datetime!(2021-07-08 06:00:00 UTC),
            datetime!(2021-07-08 18:00:00 UTC),
            datetime!(2021-07-15 06:00:00 UTC),
            datetime!(2021-07-15 18:00:00 UTC),
            datetime!(2021-07-22 06:00:00 UTC),
            datetime!(2021-07-22 18:00:00 UTC),
            datetime!(2021-07-24 18:00:00 UTC),
            datetime!(2021-07-26 06:00:00 UTC),
            datetime!(2021-07-26 18:00:00 UTC),
            datetime!(2021-07-29 06:00:00 UTC),
            datetime!(2021-07-29 18:00:00 UTC),
        ];

        assert_eq!(
            schedule
                .iter()
                .skip_outdated(false)
                .take(18)
                .collect::<Vec<OffsetDateTime>>(),
            result
        );
    }
}
