pub mod dsmr4;
pub mod e_mucs;

use crate::{Error, Result};

/// One of two tariffs used by the meter.
#[derive(Debug)]
pub enum Tariff {
    Tariff1 = 0,
    Tariff2 = 1,
}

/// One of up to three powerlines connected to the meter.
#[derive(Debug)]
pub enum Line {
    Line1 = 0,
    Line2 = 1,
    Line3 = 2,
}

/// On of up to four slave meters connected to the meter.
#[derive(Debug)]
pub enum Slave {
    Slave1 = 0,
    Slave2 = 1,
    Slave3 = 2,
    Slave4 = 3,
}

pub trait Parseable<'a>: Sized {
    fn parse(reference: &'a str, body: &'a str) -> Result<Self>;

    fn parse_line(line: &'a str) -> Result<Self> {
        let reference_end = line.find('(').ok_or(Error::InvalidFormat)?;
        let (reference, body) = line.split_at(reference_end);

        Self::parse(reference, body)
    }
}
