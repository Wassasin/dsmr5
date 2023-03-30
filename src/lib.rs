//! A no-std Rust implementation of the [DSMR5 standard](https://www.netbeheernederland.nl/_upload/Files/Slimme_meter_15_a727fce1f1.pdf) (Dutch Smart Meter Requirements).

#![no_std]

pub mod obis;
pub mod state;
pub mod types;

mod reader;
pub use crate::reader::*;

#[derive(Debug)]
pub enum Error {
    InvalidFormat,
    InvalidChecksum,
    UnknownObis,
    ObisForgotten,
}

pub type Result<T> = core::result::Result<T, Error>;

/// A data readout message from the metering system as per section 6.2.
pub struct Readout {
    pub buffer: [u8; 2048], // Maximum size of a Readout
}

impl Readout {
    /// Parse the readout to an actual telegram message.
    ///
    /// Checks the integrity of the telegram by the CRC16 checksum included.
    /// Parses the prefix and identification, and will allow the parsing of the COSEM objects.
    pub fn to_telegram(&'_ self) -> Result<Telegram<'_>> {
        let buffer = core::str::from_utf8(&self.buffer).map_err(|_| Error::InvalidFormat)?;

        if buffer.len() < 16 {
            return Err(Error::InvalidFormat);
        }

        let data_end = buffer.find('!').ok_or(Error::InvalidFormat)?;
        let (buffer, postfix) = buffer.split_at(data_end + 1);

        let given_checksum = u16::from_str_radix(postfix.get(..4).ok_or(Error::InvalidFormat)?, 16)
            .map_err(|_| Error::InvalidFormat)?;
        let real_checksum = crc16::State::<crc16::ARC>::calculate(buffer.as_bytes());

        if given_checksum != real_checksum {
            return Err(Error::InvalidChecksum);
        }

        let data_start = buffer.find("\r\n\r\n").ok_or(Error::InvalidFormat)?;
        let (header, data) = buffer.split_at(data_start);

        let prefix = header.get(1..4).ok_or(Error::InvalidFormat)?;
        let identification = header.get(5..).ok_or(Error::InvalidFormat)?;

        Ok(Telegram {
            checksum: given_checksum,
            prefix,
            identification,
            object_buffer: data.get(4..data.len() - 3).ok_or(Error::InvalidFormat)?,
        })
    }
}

/// A P1 telegram from the metering system as per section 6.12.
pub struct Telegram<'a> {
    /// The verified CRC16 checksum of the telegram data.
    pub checksum: u16,

    /// The first 3 characters of the datagram.
    pub prefix: &'a str,

    /// Metering system identification.
    pub identification: &'a str,

    /// String buffer representing the COSEM objects.
    object_buffer: &'a str,
}

impl<'a> Telegram<'a> {
    /// Parse the COSEM objects, yielding them as part of an iterator.
    pub fn objects<T: obis::Parseable<'a> + 'a>(
        &'a self,
    ) -> impl core::iter::Iterator<Item = Result<T>> + 'a {
        self.object_buffer.lines().map(|l| T::parse_line(l))
    }
}

#[cfg(test)]
#[macro_use]
extern crate std;
