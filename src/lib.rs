//! This crate allows to parse a cron-like, human-readable expression.
//! The resulting object can be turned into an iterator to compute the next date
//! (as a `time::OffsetDateTime`) one at a time. By default dates are computed
//! in the current local offset, but the iterator can be configured to use other
//! offsets.
//!
//! # Example
//! ```rust
//! use cron_lingo::Schedule;
//! use std::str::FromStr;
//! use time::macros::offset;
//!
//! fn main() -> Result<(), cron_lingo::error::Error> {
//!     // Create a schedule from an expression and iterate.
//!     let expr = "at 6:30 AM on Mondays and Thursdays";
//!     let schedule1 = Schedule::from_str(expr)?;
//!     assert!(schedule1.iter()?.next().is_some());
//!
//!     // Create another schedule, add it to the first, and then iterate.
//!     let schedule2 = Schedule::from_str("at 8 PM on the first Sunday")?;
//!     let mut combination = schedule1 + schedule2;
//!     assert!(combination.iter()?.next().is_some());
//!
//!     // Finally add another to the existing collection of schedules.
//!     let schedule3 = Schedule::from_str("at 12:00 PM on the last Friday")?;
//!     combination += schedule3;
//!     assert!(combination.iter()?.next().is_some());
//!
//!     // The examples above assume that the current local offset is to
//!     // be used to determine the dates, but dates can also be computed
//!     // in different offsets.
//!     let expr = "at 6:30 AM on Mondays and Thursdays";
//!     let schedule = Schedule::from_str(expr)?;
//!     assert!(schedule.iter()?.assume_offset(offset!(+3)).next().is_some());
//!     Ok(())
//! }
//! ```
//!
//! # Expression syntax
//!
//! A single expression consists of three parts:
//! a time specification, and optionally a weekday and week specification.
//!
//! > \<time spec\> [\<weekday spec\>] [\<week spec\>]
//!
//! Here are a few random examples of complete expressions:
//!
//! * at 1 AM
//! * 6 PM on Saturdays and Sundays
//! * at 2 PM (Mondays, Thursdays) in even weeks
//! * at 6:45 PM on Wednesdays in odd weeks
//! * at 6:30 AM on Mondays
//! * at 6 AM, 6 PM (Mondays)
//! * at 8 AM on the first Sunday
//!
//! This table gives some more examples for each type of specification in a block:
//!
//! | Times                  | Weekday (optional)           | Week (optional)  |
//! | ---------------------- | ---------------------------- | ---------------- |
//! | at every full hour     | on Mondays and Tuesdays      | in odd weeks     |
//! | at 7:30 AM and 7:30 PM | on Tuesdays, Saturdays       | in even weeks    |
//! | at 6 AM, 6 PM and 8 PM | on Fridays                   |                  |
//! | at 6 AM, 12 AM, 6 PM   |                              |                  |
//! | at 8:30 AM             |                              | in odd weeks     |
//! | at 8 AM                | on Wednesdays                |                  |
//! | at 08 AM               | on the first Monday          |                  |
//! | at 08 PM               | on the 4th Friday            | in even weeks    |
//! | at 08 PM               | on Wednesdays and Sundays    |                  |
//! | at 5:45 AM             | (Mondays and Thursdays)      |                  |
//! | at 6 AM and 6 PM       | (first Sunday)               |                  |
//! | at 1:15 PM             | (1st Monday and 2nd Friday)  |                  |
//! | at 1 PM                | on the third Monday          |                  |
//! | at 1:50 PM             | on the 3rd Monday            |                  |
//! | at 1 PM                | on the 4th Saturday          |                  |
//! | at 6 PM                | on the last Monday           |                  |
//!
//! ## Ruleset
//!
//! The examples above cover the basic rules of the expression syntax to a certain (and for most use cases
//! probably sufficient) extent.
//! Nevertheless, here is a list of "rules" that an expression must comply with and that you might find useful to avoid mistakes:
//!
//! ### Times specification
//!
//! * must start with _at_
//! * then follows either _every full hour_ OR a list of distinct _times_
//! * a _time_ adheres to the 12-hour-clock, so a number from 1 to 12 followed by _AM_ or _PM_ (uppercase!), e.g. 1 AM or 1 PM
//! * a time may also contain _minutes_ from 00 to 59 (separated from the hour by a _colon_). Omitting the minutes means
//! _on the hour_, e.g. 8 PM == 8:00 PM
//! * distinct times are concatenated by _commata_ or _and_
//!
//! ### Weekday specification
//!
//! * is _optional_
//! * succeeds the _time spec_
//! * consists of a list of _weekdays_ with optional _modifiers_ to select only specific weekdays in a month.
//! * the list either starts with _on_ OR is enclosed by simple braces _()_ for compactness
//! * a weekday must be one of [ Monday | Tuesday | Wednesday | Thursday | Friday | Saturday | Sunday ] appended with an ***s*** if e.g. _every_ Monday is to be included OR a weekday preceded by a modifier [ first | 1st | second | 2nd | third | 3rd | fourth | 4th | last ] in order to include only specific weekdays in a month.
//!
//! ### Week specification
//!
//! * is _optional_
//! * must be one of _in even weeks_ / _in odd weeks_
pub mod error;
mod parse;
pub mod schedule;
mod types;

pub use self::schedule::Schedule;
