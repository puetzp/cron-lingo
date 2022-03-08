# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2022-03-08
### Added
- Added methods `ScheduleIter::assume_offset`, `ScheduleIter::use_local_offset` and pendants for `MultiScheduleIter` that allow to change the default offset (local) that dates are computed in to another.

## [0.4.0] - 2021-12-04
### Added
- Introduced new WeekdayModifier variant "Last" to select the last occurrence of some weekday, e.g. "at 6 PM (last Monday)".
- Added `MultiSchedule` to accomodate a combination of several distinct `Schedule`s.
- Added `cron_lingo::error::Error::UnexpectedEndOfInput`.
- Added `cron_lingo::error::Error::Syntax` and `cron_lingo::error::SyntaxError`.
### Changed
- Renamed `cron_lingo::error::InvalidExpression` to simply `cron_lingo::error::Error`.
- Expressions must now be parsed separately using `from_str` but may then be combined together with `std::ops::Add` or added to an existing `MultiSchedule` using `std::ops::AddAssign`.
- Iterator of `Schedule` or `MultiSchedule` is now fallible (`next()` returns `Option<Result<time::OffsetDateTime, cron_lingo::error::Error>`) as the retrieval of the local offset may fail.
- Creation of said iterators via `Schedule::iter()` or `MultiSchedule::iter()` is now fallible as well for the same reason.
- Moved from time v0.2.22 to v0.3. Compile flag is needed in order to avoid runtime errors when the local offset is retrieved internally: `RUSTFLAGS="--cfg unsound_local_offset" cargo build`. The compile flag may be omitted when your application is single-threaded.
### Removed
- Removed variants `DuplicateInput` and `IllogicalWeekdayCombination` from error enum as the occurrence of duplicates et al. does not prevent correct computation of the next upcoming date.

## [0.3.1] - 2021-06-10
### Added
- Check for illogical combinations of weekdays and modifiers, e.g. "on Tuesdays and the 3rd Tuesday"
### Changed
- Updated module description for more clarity regarding expression syntax

## [0.3.0] - 2021-06-09
### Added
- Add `skip_outdated()` method to ScheduleIter to override default behaviour in `next()`. By default `next()` will never return a date that is in the past, but instead resume the iteration from the current local time.
### Changed
- Rename Timetable to Schedule
- Complete expression syntax overhaul to offer more possibilities

## [0.2.2] - 2021-05-25
### Changed
- Remove explicit lifetime from TimetableIter in order to use it with pyo3

## [0.2.1] - 2021-03-16
### Changed
- Fixed readme to reflect latest changes ...

## [0.2.0] - 2021-03-15
### Changed
- `Timetable` does not implement `Iterator` anymore. Instead `.iter()` returns a wrapper struct `TimetableIter` that keeps track of state during iteration.
- Removed function `new()`.
