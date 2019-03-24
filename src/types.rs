//! COSEM data types such as timestamps or fixed point numbers as per section 6.4.
//!
//! Guaranteed to either be parsed stack-compatible types or buffer references.

use crate::{Error, Result};

/// Octet strings as defined by tag 9.
#[derive(Debug)]
pub struct OctetString<'a>(&'a str);

impl<'a> OctetString<'a> {
    /// Parse a fixed length string from an OBIS body.
    pub fn parse(body: &'a str, length: usize) -> Result<OctetString<'a>> {
        Ok(OctetString(
            body.get(1..=length).ok_or(Error::InvalidFormat)?,
        ))
    }

    /// Parse a variable length string with a max length from an OBIS body.
    pub fn parse_max(body: &'a str, max_length: usize) -> Result<OctetString<'a>> {
        let end = body.find(')').ok_or(Error::InvalidFormat)? - 1;
        if end > max_length {
            return Err(Error::InvalidFormat);
        }

        OctetString::parse(body, end)
    }

    /// Yield this octet string as the underlying octets.
    pub fn as_octets(&'a self) -> impl core::iter::Iterator<Item = Result<u8>> + 'a {
        (0..self.0.len() / 2).map(move |i| {
            u8::from_str_radix(&self.0[i * 2..=i * 2 + 1], 16).map_err(|_| Error::InvalidFormat)
        })
    }
}

/// Timestamps.
#[derive(Debug, PartialEq)]
pub struct TST {
    pub year: u8,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,

    /// Daylight savings time.
    pub dst: bool,
}

impl TST {
    pub fn parse(body: &str) -> Result<TST> {
        if body.len() < 15 {
            return Err(Error::InvalidFormat);
        }

        let parsetwo =
            |i| u8::from_str_radix(&body[i..=(i + 1)], 10).map_err(|_| Error::InvalidFormat);

        Ok(TST {
            year: parsetwo(1)?,
            month: parsetwo(3)?,
            day: parsetwo(5)?,
            hour: parsetwo(7)?,
            minute: parsetwo(9)?,
            second: parsetwo(11)?,
            dst: match &body[13..=13] {
                "S" => Ok(true),
                "W" => Ok(false),
                _ => Err(Error::InvalidFormat),
            }?,
        })
    }
}

/// Fixed length unsigned doubles as defined by tag 6.
#[derive(Debug)]
pub struct UFixedDouble {
    buffer: u64,
    point: u8,
}

impl UFixedDouble {
    pub fn parse(body: &str, length: usize, point: u8) -> Result<UFixedDouble> {
        // Do not forget the extra '.'
        let buffer = body.get(1..length + 2).ok_or(Error::InvalidFormat)?;
        let (upper, lower) = buffer.split_at(length - point as usize);

        let upper = u64::from_str_radix(upper, 10).map_err(|_| Error::InvalidFormat)?;
        let lower = u64::from_str_radix(&lower[1..], 10).map_err(|_| Error::InvalidFormat)?;

        Ok(UFixedDouble {
            buffer: upper * 10u64.pow(u32::from(point)) + lower,
            point,
        })
    }
}

impl core::convert::From<&UFixedDouble> for f64 {
    fn from(other: &UFixedDouble) -> Self {
        other.buffer as f64 / (10u64.pow(u32::from(other.point)) as f64)
    }
}

/// Fixed length unsigned integers as defined by tags 15-21.
#[derive(Debug)]
pub struct UFixedInteger(pub u64);

impl UFixedInteger {
    pub fn parse(body: &str, length: usize) -> Result<UFixedInteger> {
        let buffer = body.get(1..=length).ok_or(Error::InvalidFormat)?;
        let number = u64::from_str_radix(buffer, 10).map_err(|_| Error::InvalidFormat)?;

        Ok(UFixedInteger(number))
    }
}
