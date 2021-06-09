//! This crate allows to parse a cron-like, human-readable expression.
//! The resulting object can be turned into an iterator to compute the next date
//! (as a `time::OffsetDateTime`) one at a time.
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
//! let expr = "at 6 AM on Mondays and Thursdays and at 6 PM on Sundays in even weeks";
//! let schedule = Schedule::from_str(expr).unwrap();
//! assert!(schedule.iter().next().is_some());
//! ```
//!
//! # Expression syntax
//!
//! An expression consists of {1, n} blocks. Each block contains a specification according
//! to the following rules:
//!
//! | Hour                   | Weekday (optional)           | Week (optional)  |
//! | ---------------------- | ---------------------------- | ---------------- |
//! | at every full hour     | on Mondays and Tuesdays      | in odd weeks     |
//! | at 7 AM and 7 PM       | on Tuesdays, Saturdays       | in even weeks    |
//! | at 6 AM, 6 PM and 8 PM | on Fridays                   |                  |
//! | at 6 AM, 12 AM, 6 PM   |                              |                  |
//! | at 8 AM                |                              | in odd weeks     |
//! | at 8 AM                | on Wednesdays                |                  |
//! | at 8 AM                | on the first Monday          |                  |
//! | at 8 PM                | on the 4th Friday            | in even weeks    |
//! | at 8 PM                | on Wednesdays and Sundays    |                  |
//! | at 5 AM                | (Monday and Thursdays)       |                  |
//! | at 6 AM and 6 PM       | (first Sunday)               |                  |
//! | at 1 PM                | (1st Monday and 2nd Friday)  |                  |
//! | at 1 PM                | on the third Monday          |                  |
//! | at 1 PM                | on the 3rd Monday            |                  |
//! | at 1 PM                | on the 4th Saturday          |                  |
//!
//! The separate blocks (if its more than one) are then concatenated by commata or "and".
//! Here are a few examples of complete expressions:
//!
//! * at 1 AM and at 6 PM on Saturdays and Sundays
//! * at 6 AM on Mondays and at 6 PM on Thursdays
//! * at 6 AM, 6 PM (Mondays) and at 8 AM on the first Sunday
//! * at 2 PM (Mondays, Thursdays) in even weeks, at 6 PM on Wednesdays in odd weeks and at 1 AM
//!
pub mod error;
pub mod schedule;
mod types;

pub use self::schedule::Schedule;
