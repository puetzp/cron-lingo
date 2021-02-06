# cron-lingo

A small Rust library to parse a cron-like, human-readable expression like "at 6 and 18 o'clock on Tuesday and Thursday in odd weeks" and use it to iterate upcoming dates.

The main goal is to provide a more predictable way for e.g. schedulling critical tasks by getting rid of some core functionality of standard cron. Also the expression syntax is self-explanatory to a large extent, which may present a useful side-effect if you are planning to expose the configuration of some scheduler to non-technical staff.

Please check out the module-level documentation on [docs.rs](https://docs.rs/cron-lingo) for specifics on the applied syntax.