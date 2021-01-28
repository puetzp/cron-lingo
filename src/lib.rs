//! Small utility to parse a cron-like expression in human-readable format and
//! iterate over upcoming dates according to this schedule.
//! # Example
//! ```rust
//! use cron_lingo::Timetable;
//!
//! fn main() {
//!     let expr = "at 9 o'clock on Monday and Friday";
//!     let timetable = Timetable::new(expr).unwrap();
//!     assert!(timetable.into_iter().next().is_some());
//! }
//! ```
pub mod error;
pub mod timetable;

pub use self::timetable::Timetable;
