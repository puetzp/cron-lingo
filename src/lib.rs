pub mod error;
mod timetable;

use crate::timetable::Timetable;
use std::error::Error;
use std::str::FromStr;
use time::OffsetDateTime;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn get_next(expression: &str, base: OffsetDateTime) -> Result<OffsetDateTime> {
    let timetable = Timetable::from_str(expression)?;
    timetable.compute_next_date(base)
}

pub fn get_next_date(expression: &str) -> Result<OffsetDateTime> {
    let now = OffsetDateTime::try_now_local()?;
    get_next(expression, now)
}

pub fn get_next_n_dates(_expression: &str, _n: u8) -> Result<Vec<OffsetDateTime>> {
    Ok(vec![])
}
