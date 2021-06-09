# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2021-06-03
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
