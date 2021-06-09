# cron-lingo

A small Rust library to parse a cron-like, human-readable expression like "at 6 AM on Mondays and at 6 PM (Saturdays and Sundays)" and use it to iterate upcoming dates.

The main goal is to provide a more predictable way for e.g. schedulling critical tasks by getting rid of some core functionality of standard cron. Also the expression syntax is self-explanatory to a large extent, which may present a useful side-effect if you are planning to expose the configuration of some scheduler to non-technical staff.

## Example

```rust
use cron_lingo::Schedule;
use std::str::FromStr;

fn main() {
    let schedule = Schedule::from_str("at 1 PM on Mondays").unwrap();

    for date in schedule.iter().take(3) {
        println!("{}", date);
    }
}

// Output:
// 2021-06-14 13:00 +2
// 2021-06-21 13:00 +2
// 2021-06-28 13:00 +2
```

Please check out the module-level documentation on [docs.rs](https://docs.rs/cron-lingo) for specifics on the applied syntax.
