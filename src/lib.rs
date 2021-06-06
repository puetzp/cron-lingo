//! This crate allows to parse a cron-like, human-readable expression.
//! The resulting object can be turned into an iterator to compute the next
//! `time::OffsetDateTime` one at a time.
//!
//! The basic idea was to strip down the set of features that we know from our favorite
//! implementation of cron in order to keep the results predictable, which may or may
//! not be the case with cron in some situations (e.g. using the step notation "*/2").
//!
//! # Example
//! ```rust
//! use cron_lingo::Schedule;
//! use std::str::FromStr;
//!
//! let expr = "at 9 o'clock on Monday and Friday";
//! let schedule = Schedule::from_str(expr).unwrap();
//! assert!(schedule.iter().next().is_some());
//! ```
//!
//! # Expression syntax
//!
//! The syntax is quite limited, but intentionally so.
//!
//! Here are a few examples:
//!
//! | Hour                   | Weekday (optional)      | Week (optional)                 |
//! | ---------------------- | ----------------------- | ------------------------------- |
//! | at every hour          | on Monday and Tuesday   | in odd weeks                    |
//! | at 7 and 8 o'clock     | on Tuesday, Saturday    | in even weeks                   |
//! | at 7, 8 and 16 o'clock | on Friday               |                                 |
//! | at 6, 12, 18 o'clock   |                         |                                 |
//! | at 8 o'clock           |                         | in odd weeks                    |
//! | at 8 o'clock           | on Wednesday            | in the first week of the month  |
//! | at 8 o'clock           | on Wednesday            | in the second week of the month |
//! | at 8 o'clock           | on Wednesday            | in the third week of the month  |
//! | at 8 o'clock           | on Wednesday and Sunday | in the fourth week of the month |
//!
//! The examples are quite self-explanatory, but the last four may need some clarification:
//! In the final example, `next()` could return the date of the fourth Wednesday of this month
//! (or the next if the current one is in the past). But only if the fourth Sunday of this
//! month does not predate the fourth Wednesday, which may be the case when the month in question
//! begins on e.g. Friday.
//!
//! As you can also see in the table above the column "Week" does not depend on the second block
//! "Weekday". However omitting the Weekday spec. rarely makes sense. The example in row #5 would
//! (assuming now is a Sunday in an even week) return a `time::OffsetDateTime` for the next seven
//! days ... and then put in a break for the following week.
pub mod error;
pub mod schedule;
mod types;

pub use self::schedule::Schedule;
