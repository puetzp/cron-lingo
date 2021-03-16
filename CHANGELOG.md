# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2021-03-15
### Changed
- `Timetable` does not implement `Iterator` anymore. Instead `.iter()` returns a wrapper struct `TimetableIter` that keeps track of state during iteration.
- Removed function `new()`.
