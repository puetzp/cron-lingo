//! Small utility to parse a cron-like expression in human-readable format and
//! iterate over upcoming dates according to this schedule.
//! # Example
//! ```rust
//! use cron_lingo::Timetable;
//!
//! let expr = "at 9 o'clock on Monday and Friday";
//! let timetable = Timetable::new(expr).unwrap();
//! assert!(timetable.into_iter().next().is_some());
//! ```
//!
//! # Expression syntax
//!
//! The syntax is (so far) quite limited. Some things that are expressible
//! in standard cron syntax, for example the step-notation ("*/2"), are intentionally
//! omitted. Some things are just missing because they have not been implemented
//! yet.
//!
//! Here are a few examples:
//!
//! | Hour                   | Weekday (optional)    | Week (optional) |
//! | ---------------------- | --------------------- | --------------- |
//! | at every hour          | on Monday and Tuesday | in odd weeks    |
//! | at 7 and 8 o'clock     | on Tuesday, Saturday  | in even weeks   |
//! | at 7, 8 and 16 o'clock | on Friday             |                 |
//! | at 6, 12, 18 o'clock   |                       |                 |
//! | at 8 o'clock           |                       | in odd weeks    |
pub mod error;
pub mod timetable;

pub use self::timetable::Timetable;
