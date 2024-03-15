//! A no-std Rust implementation of the [DSMR5 standard](https://www.netbeheernederland.nl/_upload/Files/Slimme_meter_15_a727fce1f1.pdf) (Dutch Smart Meter Requirements).

#![no_std]

pub mod state;
pub mod types;

mod obis;
mod reader;

pub use crate::obis::*;
pub use crate::reader::*;

#[derive(Debug)]
pub enum Error {
    InvalidFormat,
    InvalidChecksum,
    UnknownObis,
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
    pub fn objects(&self) -> impl core::iter::Iterator<Item = Result<OBIS<'a>>> {
        self.object_buffer.lines().map(OBIS::parse)
    }
}

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    #[test]
    fn example_isk() {
        let mut buffer = [0u8; 2048];
        let file = std::fs::read("test/isk.txt").unwrap();

        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout { buffer };
        let telegram = readout.to_telegram().unwrap();

        assert_eq!(telegram.prefix, "ISK");
        assert_eq!(telegram.identification, "\\2M550E-1012");

        telegram.objects().for_each(|o| {
            println!("{:?}", o); // to see use `$ cargo test -- --nocapture`
            let o = o.unwrap();

            use crate::OBIS::*;
            use core::convert::From;
            match o {
                Version(v) => {
                    let b: std::vec::Vec<u8> = v.as_octets().map(|b| b.unwrap()).collect();
                    assert_eq!(b, [80]);
                }
                DateTime(tst) => {
                    assert_eq!(
                        tst,
                        crate::types::TST {
                            year: 19,
                            month: 3,
                            day: 20,
                            hour: 18,
                            minute: 14,
                            second: 3,
                            dst: false
                        }
                    );
                }
                EquipmentIdentifier(ei) => {
                    let b: std::vec::Vec<u8> = ei.as_octets().map(|b| b.unwrap()).collect();
                    assert_eq!(std::str::from_utf8(&b).unwrap(), "E0043007052870318");
                }
                MeterReadingTo(crate::Tariff::Tariff1, mr) => {
                    assert_eq!(f64::from(&mr), 576.239);
                }
                MeterReadingTo(crate::Tariff::Tariff2, mr) => {
                    assert_eq!(f64::from(&mr), 465.162);
                }
                TariffIndicator(ti) => {
                    let b: std::vec::Vec<u8> = ti.as_octets().map(|b| b.unwrap()).collect();
                    assert_eq!(b, [0, 2]);
                }
                PowerFailures(crate::types::UFixedInteger(pf)) => {
                    assert_eq!(pf, 9);
                }
                _ => (), // Do not test the rest.
            }
        });
    }

    #[test]
    fn example_kaifa() {
        let mut buffer = [0u8; 2048];
        let file = std::fs::read("test/kaifa.txt").unwrap();

        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout { buffer };
        let telegram = readout.to_telegram().unwrap();

        assert_eq!(telegram.prefix, "KFM");
        assert_eq!(telegram.identification, "KAIFA-METER");

        telegram.objects().for_each(|o| {
            println!("{:?}", o); // to see use `$ cargo test -- --nocapture`
            let o = o.unwrap();

            use crate::OBIS::*;
            use core::convert::From;
            match o {
                Version(v) => {
                    let b: std::vec::Vec<u8> = v.as_octets().map(|b| b.unwrap()).collect();
                    assert_eq!(b, [66]);
                }
                DateTime(tst) => {
                    assert_eq!(
                        tst,
                        crate::types::TST {
                            year: 22,
                            month: 9,
                            day: 1,
                            hour: 15,
                            minute: 22,
                            second: 1,
                            dst: true
                        }
                    );
                }
                EquipmentIdentifier(ei) => {
                    let b: std::vec::Vec<u8> = ei.as_octets().map(|b| b.unwrap()).collect();
                    assert_eq!(std::str::from_utf8(&b).unwrap(), "E0026000024153615");
                }
                MeterReadingTo(crate::Tariff::Tariff1, mr) => {
                    assert_eq!(f64::from(&mr), 6285.065);
                }
                MeterReadingTo(crate::Tariff::Tariff2, mr) => {
                    assert_eq!(f64::from(&mr), 6758.327);
                }
                TariffIndicator(ti) => {
                    let b: std::vec::Vec<u8> = ti.as_octets().map(|b| b.unwrap()).collect();
                    assert_eq!(b, [0, 2]);
                }
                PowerFailures(crate::types::UFixedInteger(pf)) => {
                    assert_eq!(pf, 3);
                }
                _ => (), // Do not test the rest.
            }
        });
    }

    #[test]
    fn example_mcs() {
        let mut buffer = [0u8; 2048];
        let file = std::fs::read("test/mcs.txt").unwrap();

        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout { buffer };
        let telegram = readout.to_telegram().unwrap();

        assert_eq!(telegram.prefix, "MCS");
        assert_eq!(telegram.identification, "0000000000000");

        telegram.objects().for_each(|o| {
            //     println!("{:?}", o); // to see use `$ cargo test -- --nocapture`
            let o = o.unwrap();

            use crate::OBIS::*;
            match o {
                Version(v) => {
                    let b: std::vec::Vec<u8> = v.as_octets().map(|b| b.unwrap()).collect();
                    assert_eq!(b, [80]);
                }
                SlaveDeviceType(_slave, devide_type) => {
                    assert_eq!(devide_type.is_none(), true);
                }
                SlaveMeterReading(_slave, _tst, value) => {
                    assert_eq!(value.is_none(), true)
                }
                _ => (), // Do not test the rest.
            }
        });
    }
}
