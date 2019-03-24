use crate::types::*;
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

/// OBIS data objects like the current power usage.
///
/// As per section 6.12 of the requirements specification.
#[derive(Debug)]
pub enum OBIS<'a> {
    Version(OctetString<'a>),
    DateTime(TST),
    EquipmentIdentifier(OctetString<'a>),
    MeterReadingTo(Tariff, UFixedDouble),
    MeterReadingBy(Tariff, UFixedDouble),

    /// Current Tariff applicable as reported by the meter.
    /// Note that the format of this string is not defined in the requirements.
    /// Check what your meter emits in practice.
    TariffIndicator(OctetString<'a>),
    PowerDelivered(UFixedDouble),
    PowerReceived(UFixedDouble),
    PowerFailures(UFixedInteger),
    LongPowerFailures(UFixedInteger),
    PowerFailureEventLog, // TODO
    TextMessage,          // TODO
    VoltageSags(Line, UFixedInteger),
    VoltageSwells(Line, UFixedInteger),
    InstantaneousVoltage(Line, UFixedDouble),
    InstantaneousCurrent(Line, UFixedInteger),
    InstantaneousActivePowerPlus(Line, UFixedDouble),
    InstantaneousActivePowerNeg(Line, UFixedDouble),
    SlaveDeviceType(Slave, UFixedInteger),
    SlaveEquipmentIdentifier(Slave, OctetString<'a>),
    SlaveMeterReading(Slave, TST, UFixedDouble),
}

impl<'a> OBIS<'a> {
    pub fn parse(line: &'a str) -> Result<OBIS<'a>> {
        let reference_end = line.find('(').ok_or(Error::InvalidFormat)?;
        let (reference, body) = line.split_at(reference_end);

        use Line::*;
        use Tariff::*;

        match reference {
            "1-3:0.2.8" => Ok(OBIS::Version::<'a>(OctetString::parse(body, 2)?)),
            "0-0:1.0.0" => Ok(OBIS::DateTime(TST::parse(body)?)),
            "0-0:96.1.1" => Ok(OBIS::EquipmentIdentifier::<'a>(OctetString::parse_max(
                body, 96,
            )?)),
            "1-0:1.8.1" => Ok(OBIS::MeterReadingTo(
                Tariff1,
                UFixedDouble::parse(body, 9, 3)?,
            )),
            "1-0:1.8.2" => Ok(OBIS::MeterReadingTo(
                Tariff2,
                UFixedDouble::parse(body, 9, 3)?,
            )),
            "1-0:2.8.1" => Ok(OBIS::MeterReadingBy(
                Tariff1,
                UFixedDouble::parse(body, 9, 3)?,
            )),
            "1-0:2.8.2" => Ok(OBIS::MeterReadingBy(
                Tariff2,
                UFixedDouble::parse(body, 9, 3)?,
            )),
            "0-0:96.14.0" => Ok(OBIS::TariffIndicator::<'a>(OctetString::parse(body, 4)?)),
            "1-0:1.7.0" => Ok(OBIS::PowerDelivered(UFixedDouble::parse(body, 5, 3)?)),
            "1-0:2.7.0" => Ok(OBIS::PowerReceived(UFixedDouble::parse(body, 5, 3)?)),
            "0-0:96.7.21" => Ok(OBIS::PowerFailures(UFixedInteger::parse(body, 5)?)),
            "0-0:96.7.9" => Ok(OBIS::LongPowerFailures(UFixedInteger::parse(body, 5)?)),
            "1-0:99.97.0" => Ok(OBIS::PowerFailureEventLog), // TODO
            "1-0:32.32.0" => Ok(OBIS::VoltageSags(Line1, UFixedInteger::parse(body, 5)?)),
            "1-0:52.32.0" => Ok(OBIS::VoltageSags(Line2, UFixedInteger::parse(body, 5)?)),
            "1-0:72.32.0" => Ok(OBIS::VoltageSags(Line3, UFixedInteger::parse(body, 5)?)),
            "1-0:32.36.0" => Ok(OBIS::VoltageSwells(Line1, UFixedInteger::parse(body, 5)?)),
            "1-0:52.36.0" => Ok(OBIS::VoltageSwells(Line2, UFixedInteger::parse(body, 5)?)),
            "1-0:72.36.0" => Ok(OBIS::VoltageSwells(Line3, UFixedInteger::parse(body, 5)?)),
            "0-0:96.13.0" => Ok(OBIS::TextMessage), // TODO
            "1-0:32.7.0" => Ok(OBIS::InstantaneousVoltage(
                Line1,
                UFixedDouble::parse(body, 4, 1)?,
            )),
            "1-0:52.7.0" => Ok(OBIS::InstantaneousVoltage(
                Line2,
                UFixedDouble::parse(body, 4, 1)?,
            )),
            "1-0:72.7.0" => Ok(OBIS::InstantaneousVoltage(
                Line3,
                UFixedDouble::parse(body, 4, 1)?,
            )),
            "1-0:31.7.0" => Ok(OBIS::InstantaneousCurrent(
                Line1,
                UFixedInteger::parse(body, 3)?,
            )),
            "1-0:51.7.0" => Ok(OBIS::InstantaneousCurrent(
                Line2,
                UFixedInteger::parse(body, 3)?,
            )),
            "1-0:71.7.0" => Ok(OBIS::InstantaneousCurrent(
                Line3,
                UFixedInteger::parse(body, 3)?,
            )),
            "1-0:21.7.0" => Ok(OBIS::InstantaneousActivePowerPlus(
                Line1,
                UFixedDouble::parse(body, 5, 3)?,
            )),
            "1-0:41.7.0" => Ok(OBIS::InstantaneousActivePowerPlus(
                Line1,
                UFixedDouble::parse(body, 5, 3)?,
            )),
            "1-0:61.7.0" => Ok(OBIS::InstantaneousActivePowerPlus(
                Line3,
                UFixedDouble::parse(body, 5, 3)?,
            )),
            "1-0:22.7.0" => Ok(OBIS::InstantaneousActivePowerNeg(
                Line2,
                UFixedDouble::parse(body, 5, 3)?,
            )),
            "1-0:42.7.0" => Ok(OBIS::InstantaneousActivePowerNeg(
                Line2,
                UFixedDouble::parse(body, 5, 3)?,
            )),
            "1-0:62.7.0" => Ok(OBIS::InstantaneousActivePowerNeg(
                Line3,
                UFixedDouble::parse(body, 5, 3)?,
            )),
            _ => {
                if reference.len() != 10 || reference.get(..2).ok_or(Error::InvalidFormat)? != "0-"
                {
                    return Err(Error::UnknownObis);
                }

                let channel =
                    u8::from_str_radix(&reference[2..=2], 10).map_err(|_| Error::InvalidFormat)?;

                use Slave::*;
                let channel = match channel {
                    1 => Ok(Slave1),
                    2 => Ok(Slave2),
                    3 => Ok(Slave3),
                    4 => Ok(Slave4),
                    _ => Err(Error::InvalidFormat),
                }?;
                let subreference = &reference[4..];

                match subreference {
                    "24.1.0" => Ok(OBIS::SlaveDeviceType(
                        channel,
                        UFixedInteger::parse(body, 3)?,
                    )),
                    "96.1.0" => Ok(OBIS::SlaveEquipmentIdentifier::<'a>(
                        channel,
                        OctetString::parse_max(body, 96)?,
                    )),
                    "24.2.1" => {
                        let end = body[1..].find('(').ok_or(Error::InvalidFormat)?;
                        let (time, measurement) = body.split_at(end + 1);

                        let period = measurement.find('.').ok_or(Error::InvalidFormat)?;

                        Ok(OBIS::SlaveMeterReading(
                            channel,
                            TST::parse(time)?,
                            UFixedDouble::parse(measurement, 8, 9 - period as u8)?,
                        ))
                    }
                    _ => Err(Error::UnknownObis),
                }
            }
        }
    }
}
