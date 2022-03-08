use crate::error::*;
use crate::parse::parse;
use crate::types::*;
use std::iter::Iterator;
use std::str::FromStr;
use time::{Duration, OffsetDateTime, PrimitiveDateTime, UtcOffset};

/// A schedule that is built from an expression and can be iterated
/// in order to compute the next date(s) that match the specification.
#[derive(Debug, PartialEq, Clone)]
pub struct Schedule(ParsedSchedule);

impl Schedule {
    #[allow(dead_code)]
    pub fn iter(&self) -> Result<ScheduleIter, Error> {
        let Schedule(schedule) = self;
        let iter = ScheduleIter {
            schedule: schedule.clone(),
            current: OffsetDateTime::now_local().map_err(Error::IndeterminateOffset)?,
            skip_outdated: true,
            offset: None,
        };
        Ok(iter)
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
    /// let expr = "at 6 AM on Mondays and Thursdays in even weeks";
    /// assert!(Schedule::from_str(expr).is_ok());
    /// ```
    fn from_str(expression: &str) -> Result<Self, Self::Err> {
        Ok(Schedule(parse(expression)?))
    }
}

impl std::ops::Add<Schedule> for Schedule {
    type Output = MultiSchedule;

    fn add(self, other: Self) -> Self::Output {
        let Schedule(first) = self;
        let Schedule(second) = other;
        MultiSchedule(vec![first, second])
    }
}

/// A wrapper around `Schedule` that keeps track of state during iteration.
#[derive(Clone)]
pub struct ScheduleIter {
    schedule: ParsedSchedule,
    current: OffsetDateTime,
    skip_outdated: bool,
    offset: Option<UtcOffset>,
}

impl ScheduleIter {
    /// By default the `next` method will not return a date that is
    /// in the past but compute the next future data. This method
    /// allows to change the iterators default behaviour.
    pub fn skip_outdated(mut self, skip: bool) -> ScheduleIter {
        self.skip_outdated = skip;
        self
    }

    /// By default the iterator returns dates in the current local
    /// offset taken from the system. This method allows to change
    /// the iteration behaviour to compute dates in another offset.
    pub fn assume_offset(mut self, offset: UtcOffset) -> ScheduleIter {
        self.offset = Some(offset);
        self
    }

    /// Compute dates in the current local offset. This is also the
    /// default behaviour and can be used to revert changes to this
    /// behaviour that were made using `assume_offset`.
    pub fn use_local_offset(mut self) -> ScheduleIter {
        self.offset = None;
        self
    }
}

impl Iterator for ScheduleIter {
    type Item = Result<OffsetDateTime, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(offset) = self.offset {
            self.current = self.current.to_offset(offset);
        }

        if self.skip_outdated {
            let mut now = match OffsetDateTime::now_local().map_err(Error::IndeterminateOffset) {
                Ok(n) => n,
                Err(e) => return Some(Err(e)),
            };

            if let Some(offset) = self.offset {
                now = now.to_offset(offset);
            }

            if now > self.current {
                self.current = now;
            }
        }

        // Create every possible combination of dates for each
        // ParsedSchedule and add them to a vector.
        let candidates: Vec<OffsetDateTime> = compute_dates(self.current, &self.schedule);

        // Iterate the vector of dates and find the next date
        // by subtracting the current date from each element
        // in the vector. Return the date that results in the
        // lowest delta.
        let next_date = candidates
            .iter()
            .min_by_key(|d| **d - self.current)
            .unwrap();

        self.current = *next_date;

        Some(Ok(*next_date))
    }
}

/// A combination of multiple schedules that can be iterated in order
/// to compute the next date(s) that match the set of specifications. By
/// default the computation is based on the current system time, meaning
/// the iterator will never return a date in the past.
#[derive(Debug, PartialEq, Clone)]
pub struct MultiSchedule(Vec<ParsedSchedule>);

impl MultiSchedule {
    #[allow(dead_code)]
    pub fn iter(&self) -> Result<MultiScheduleIter, Error> {
        let MultiSchedule(schedules) = self;
        let iter = MultiScheduleIter {
            schedules: &schedules,
            current: OffsetDateTime::now_local().map_err(Error::IndeterminateOffset)?,
            skip_outdated: true,
            offset: None,
        };
        Ok(iter)
    }
}

impl std::ops::Add<Schedule> for MultiSchedule {
    type Output = Self;

    fn add(self, other: Schedule) -> Self {
        let MultiSchedule(mut schedules) = self;
        let Schedule(schedule) = other;
        schedules.push(schedule);
        MultiSchedule(schedules)
    }
}

impl std::ops::Add<MultiSchedule> for MultiSchedule {
    type Output = Self;

    fn add(self, other: MultiSchedule) -> Self {
        let MultiSchedule(mut schedules) = self;
        let MultiSchedule(mut other_schedules) = other;
        schedules.append(&mut other_schedules);
        MultiSchedule(schedules)
    }
}

impl std::ops::AddAssign<Schedule> for MultiSchedule {
    fn add_assign(&mut self, other: Schedule) {
        let MultiSchedule(schedules) = self;
        let Schedule(schedule) = other;
        schedules.push(schedule);
    }
}

impl std::ops::AddAssign<MultiSchedule> for MultiSchedule {
    fn add_assign(&mut self, other: MultiSchedule) {
        let MultiSchedule(schedules) = self;
        let MultiSchedule(mut other_schedules) = other;
        schedules.append(&mut other_schedules);
    }
}

/// A wrapper around `MultiSchedule` that keeps track of state during iteration.
#[derive(Clone)]
pub struct MultiScheduleIter<'a> {
    schedules: &'a [ParsedSchedule],
    current: OffsetDateTime,
    skip_outdated: bool,
    offset: Option<UtcOffset>,
}

impl<'a> MultiScheduleIter<'a> {
    /// By default the `next` method will not return a date that is
    /// in the past but compute the next future data. This method
    /// allows to change the iterators default behaviour.
    pub fn skip_outdated(mut self, skip: bool) -> MultiScheduleIter<'a> {
        self.skip_outdated = skip;
        self
    }
    /// By default the iterator returns dates in the current local
    /// offset taken from the system. This method allows to change
    /// the iteration behaviour to compute dates in another offset.
    pub fn assume_offset(mut self, offset: UtcOffset) -> MultiScheduleIter<'a> {
        self.offset = Some(offset);
        self
    }

    /// Compute dates in the current local offset. This is also the
    /// default behaviour and can be used to revert changes to this
    /// behaviour that were made using `assume_offset`.
    pub fn use_local_offset(mut self) -> MultiScheduleIter<'a> {
        self.offset = None;
        self
    }
}

impl<'a> Iterator for MultiScheduleIter<'a> {
    type Item = Result<OffsetDateTime, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(offset) = self.offset {
            self.current = self.current.to_offset(offset);
        }

        if self.skip_outdated {
            let mut now = match OffsetDateTime::now_local().map_err(Error::IndeterminateOffset) {
                Ok(n) => n,
                Err(e) => return Some(Err(e)),
            };

            if let Some(offset) = self.offset {
                now = now.to_offset(offset);
            }

            if now > self.current {
                self.current = now;
            }
        }

        // Create every possible combination of dates for each
        // ParsedSchedule and add them to a vector.
        let mut candidates: Vec<OffsetDateTime> = vec![];

        for schedule in self.schedules {
            candidates.append(&mut compute_dates(self.current, schedule));
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

        Some(Ok(*next_date))
    }
}

// Returns a selection of possible next dates according to the rules in a ParsedSchedule.
fn compute_dates(base: OffsetDateTime, spec: &ParsedSchedule) -> Vec<OffsetDateTime> {
    let mut candidates = vec![];
    let today = base.date();
    let offset = base.offset();

    // For each specified time ...
    for time in &spec.times {
        // ... create an OffsetDateTime object for each upcoming weekday ...
        for i in 0..=6 {
            let mut date =
                PrimitiveDateTime::new(today + Duration::days(i), *time).assume_offset(offset);

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

    // ... and return the filtered date candidates of this ParsedSchedule.
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
    use time::macros::{datetime, offset, time};
    use time::Weekday;

    #[test]
    fn test_compute_dates_1() {
        let base = datetime!(2021-06-04 13:38:00 UTC);
        let spec = ParsedSchedule {
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
        assert_eq!(compute_dates(base, &spec), result);
    }

    #[test]
    fn test_compute_dates_2() {
        let base = datetime!(2021-06-04 13:38:00 UTC);
        let spec = ParsedSchedule {
            times: vec![time!(18:00:00)],
            days: Some(vec![(Weekday::Monday, None), (Weekday::Thursday, None)]),
            weeks: None,
        };
        let result = vec![
            datetime!(2021-06-07 18:00:00 UTC),
            datetime!(2021-06-10 18:00:00 UTC),
        ];
        assert_eq!(compute_dates(base, &spec), result);
    }

    #[test]
    fn test_compute_dates_3() {
        let base = datetime!(2021-06-04 13:38:00 UTC);
        let spec = ParsedSchedule {
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
        assert_eq!(compute_dates(base, &spec), result);
    }

    #[test]
    fn test_compute_dates_4() {
        let base = datetime!(2021-06-04 13:38:00 UTC);
        let spec = ParsedSchedule {
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
        assert_eq!(compute_dates(base, &spec), result);
    }

    #[test]
    fn test_compute_dates_5() {
        let base = datetime!(2021-06-12 13:38:00 UTC);
        let spec = ParsedSchedule {
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

        assert_eq!(compute_dates(base, &spec), result);
    }

    #[test]
    fn test_schedule_iteration_1() {
        let iterator = ScheduleIter {
            current: datetime!(2021-06-09 13:00:00 UTC),
            schedule: ParsedSchedule {
                times: vec![time!(01:00:00)],
                days: None,
                weeks: None,
            },
            skip_outdated: false,
            offset: None,
        };

        let result = vec![
            Ok(datetime!(2021-06-10 01:00:00 UTC)),
            Ok(datetime!(2021-06-11 01:00:00 UTC)),
            Ok(datetime!(2021-06-12 01:00:00 UTC)),
        ];

        assert_eq!(
            iterator
                .take(3)
                .collect::<Vec<Result<OffsetDateTime, Error>>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_2() {
        let iterator = ScheduleIter {
            current: datetime!(2021-06-09 13:00:00 UTC),
            schedule: ParsedSchedule {
                times: vec![time!(13:00:00)],
                days: Some(vec![(Weekday::Monday, None)]),
                weeks: None,
            },
            skip_outdated: false,
            offset: None,
        };

        let result = vec![
            Ok(datetime!(2021-06-14 13:00:00 UTC)),
            Ok(datetime!(2021-06-21 13:00:00 UTC)),
            Ok(datetime!(2021-06-28 13:00:00 UTC)),
        ];

        assert_eq!(
            iterator
                .take(3)
                .collect::<Vec<Result<OffsetDateTime, Error>>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_3() {
        let iterator = ScheduleIter {
            current: datetime!(2021-06-09 13:00:00 UTC),
            schedule: ParsedSchedule {
                times: vec![time!(06:00:00), time!(13:00:00)],
                days: Some(vec![
                    (Weekday::Monday, Some(WeekdayModifier::Third)),
                    (Weekday::Thursday, None),
                ]),
                weeks: None,
            },
            skip_outdated: false,
            offset: None,
        };

        let result = vec![
            Ok(datetime!(2021-06-10 06:00:00 UTC)),
            Ok(datetime!(2021-06-10 13:00:00 UTC)),
            Ok(datetime!(2021-06-17 06:00:00 UTC)),
            Ok(datetime!(2021-06-17 13:00:00 UTC)),
            Ok(datetime!(2021-06-21 06:00:00 UTC)),
            Ok(datetime!(2021-06-21 13:00:00 UTC)),
            Ok(datetime!(2021-06-24 06:00:00 UTC)),
            Ok(datetime!(2021-06-24 13:00:00 UTC)),
        ];

        assert_eq!(
            iterator
                .take(8)
                .collect::<Vec<Result<OffsetDateTime, Error>>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_4() {
        let iterator = ScheduleIter {
            current: datetime!(2021-06-09 13:00:00 UTC),
            schedule: ParsedSchedule {
                times: vec![time!(06:00:00), time!(13:00:00)],
                days: Some(vec![
                    (Weekday::Monday, Some(WeekdayModifier::Third)),
                    (Weekday::Thursday, None),
                ]),
                weeks: None,
            },
            skip_outdated: false,
            offset: Some(offset!(+3)),
        };

        let result = vec![
            Ok(datetime!(2021-06-10 06:00:00 +3)),
            Ok(datetime!(2021-06-10 13:00:00 +3)),
            Ok(datetime!(2021-06-17 06:00:00 +3)),
            Ok(datetime!(2021-06-17 13:00:00 +3)),
            Ok(datetime!(2021-06-21 06:00:00 +3)),
            Ok(datetime!(2021-06-21 13:00:00 +3)),
            Ok(datetime!(2021-06-24 06:00:00 +3)),
            Ok(datetime!(2021-06-24 13:00:00 +3)),
        ];

        assert_eq!(
            iterator
                .take(8)
                .collect::<Vec<Result<OffsetDateTime, Error>>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_5() {
        let iterator = MultiScheduleIter {
            current: datetime!(2021-06-09 13:00:00 UTC),
            schedules: &vec![
                ParsedSchedule {
                    times: vec![time!(06:00:00), time!(13:00:00)],
                    days: Some(vec![
                        (Weekday::Monday, Some(WeekdayModifier::Third)),
                        (Weekday::Thursday, None),
                    ]),
                    weeks: None,
                },
                ParsedSchedule {
                    times: vec![time!(18:00:00)],
                    days: Some(vec![(Weekday::Saturday, Some(WeekdayModifier::Fourth))]),
                    weeks: Some(WeekVariant::Odd),
                },
            ],
            skip_outdated: false,
            offset: None,
        };

        let result = vec![
            Ok(datetime!(2021-06-10 06:00:00 UTC)),
            Ok(datetime!(2021-06-10 13:00:00 UTC)),
            Ok(datetime!(2021-06-17 06:00:00 UTC)),
            Ok(datetime!(2021-06-17 13:00:00 UTC)),
            Ok(datetime!(2021-06-21 06:00:00 UTC)),
            Ok(datetime!(2021-06-21 13:00:00 UTC)),
            Ok(datetime!(2021-06-24 06:00:00 UTC)),
            Ok(datetime!(2021-06-24 13:00:00 UTC)),
            Ok(datetime!(2021-06-26 18:00:00 UTC)),
            Ok(datetime!(2021-07-01 06:00:00 UTC)),
            Ok(datetime!(2021-07-01 13:00:00 UTC)),
        ];

        assert_eq!(
            iterator
                .take(11)
                .collect::<Vec<Result<OffsetDateTime, Error>>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_6() {
        let iterator = MultiScheduleIter {
            current: datetime!(2021-06-18 13:00:00 UTC),
            schedules: &vec![
                ParsedSchedule {
                    times: vec![time!(06:00:00), time!(18:00:00)],
                    days: Some(vec![
                        (Weekday::Monday, Some(WeekdayModifier::Last)),
                        (Weekday::Thursday, None),
                    ]),
                    weeks: None,
                },
                ParsedSchedule {
                    times: vec![time!(18:00:00)],
                    days: Some(vec![(Weekday::Saturday, Some(WeekdayModifier::Fourth))]),
                    weeks: None,
                },
            ],
            skip_outdated: false,
            offset: None,
        };

        let result = vec![
            Ok(datetime!(2021-06-24 06:00:00 UTC)),
            Ok(datetime!(2021-06-24 18:00:00 UTC)),
            Ok(datetime!(2021-06-26 18:00:00 UTC)),
            Ok(datetime!(2021-06-28 06:00:00 UTC)),
            Ok(datetime!(2021-06-28 18:00:00 UTC)),
            Ok(datetime!(2021-07-01 06:00:00 UTC)),
            Ok(datetime!(2021-07-01 18:00:00 UTC)),
            Ok(datetime!(2021-07-08 06:00:00 UTC)),
            Ok(datetime!(2021-07-08 18:00:00 UTC)),
            Ok(datetime!(2021-07-15 06:00:00 UTC)),
            Ok(datetime!(2021-07-15 18:00:00 UTC)),
            Ok(datetime!(2021-07-22 06:00:00 UTC)),
            Ok(datetime!(2021-07-22 18:00:00 UTC)),
            Ok(datetime!(2021-07-24 18:00:00 UTC)),
            Ok(datetime!(2021-07-26 06:00:00 UTC)),
            Ok(datetime!(2021-07-26 18:00:00 UTC)),
            Ok(datetime!(2021-07-29 06:00:00 UTC)),
            Ok(datetime!(2021-07-29 18:00:00 UTC)),
        ];

        assert_eq!(
            iterator
                .take(18)
                .collect::<Vec<Result<OffsetDateTime, Error>>>(),
            result
        );
    }

    #[test]
    fn test_schedule_iteration_7() {
        let iterator = MultiScheduleIter {
            current: datetime!(2021-06-18 13:00:00 UTC),
            schedules: &vec![
                ParsedSchedule {
                    times: vec![time!(06:00:00), time!(18:00:00)],
                    days: Some(vec![
                        (Weekday::Monday, Some(WeekdayModifier::Last)),
                        (Weekday::Thursday, None),
                    ]),
                    weeks: None,
                },
                ParsedSchedule {
                    times: vec![time!(18:00:00)],
                    days: Some(vec![(Weekday::Saturday, Some(WeekdayModifier::Fourth))]),
                    weeks: None,
                },
            ],
            skip_outdated: false,
            offset: Some(offset!(+2:30)),
        };

        let result = vec![
            Ok(datetime!(2021-06-24 06:00:00 +2:30)),
            Ok(datetime!(2021-06-24 18:00:00 +2:30)),
            Ok(datetime!(2021-06-26 18:00:00 +2:30)),
            Ok(datetime!(2021-06-28 06:00:00 +2:30)),
            Ok(datetime!(2021-06-28 18:00:00 +2:30)),
            Ok(datetime!(2021-07-01 06:00:00 +2:30)),
            Ok(datetime!(2021-07-01 18:00:00 +2:30)),
        ];

        assert_eq!(
            iterator
                .take(7)
                .collect::<Vec<Result<OffsetDateTime, Error>>>(),
            result
        );
    }

    #[test]
    fn test_add_two_schedules() {
        let sched1 = Schedule(ParsedSchedule {
            times: vec![time!(06:00:00), time!(13:00:00)],
            days: Some(vec![
                (Weekday::Monday, Some(WeekdayModifier::Third)),
                (Weekday::Thursday, None),
            ]),
            weeks: None,
        });

        let sched2 = Schedule(ParsedSchedule {
            times: vec![time!(18:00:00)],
            days: Some(vec![(Weekday::Saturday, Some(WeekdayModifier::Fourth))]),
            weeks: Some(WeekVariant::Odd),
        });

        let multi_sched = MultiSchedule(vec![
            ParsedSchedule {
                times: vec![time!(06:00:00), time!(13:00:00)],
                days: Some(vec![
                    (Weekday::Monday, Some(WeekdayModifier::Third)),
                    (Weekday::Thursday, None),
                ]),
                weeks: None,
            },
            ParsedSchedule {
                times: vec![time!(18:00:00)],
                days: Some(vec![(Weekday::Saturday, Some(WeekdayModifier::Fourth))]),
                weeks: Some(WeekVariant::Odd),
            },
        ]);

        assert_eq!(multi_sched, sched1 + sched2)
    }
}
