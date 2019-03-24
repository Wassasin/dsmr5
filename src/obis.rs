use crate::types::*;
use crate::{Error, Result};

/// OBIS data objects like the current power usage.
///
/// As per section 6.12 of the requirements specification.
#[derive(Debug)]
pub enum OBIS<'a> {
    Version(OctetString<'a>),
    DateTime(TST),
    EquipmentIdentifier(OctetString<'a>),
    MeterReadingToTariff1(UFixedDouble),
    MeterReadingToTariff2(UFixedDouble),
    MeterReadingByTariff1(UFixedDouble),
    MeterReadingByTariff2(UFixedDouble),
    TariffIndicator(OctetString<'a>),
    PowerDelivered(UFixedDouble),
    PowerReceived(UFixedDouble),
    PowerFailures(UFixedInteger),
    LongPowerFailures(UFixedInteger),
    PowerFailureEventLog, // TODO
    VoltageSagsL1(UFixedInteger),
    VoltageSagsL2(UFixedInteger),
    VoltageSagsL3(UFixedInteger),
    VoltageSwellsL1(UFixedInteger),
    VoltageSwellsL2(UFixedInteger),
    VoltageSwellsL3(UFixedInteger),
    TextMessage, // TODO
    InstantaneousVoltageL1(UFixedDouble),
    InstantaneousVoltageL2(UFixedDouble),
    InstantaneousVoltageL3(UFixedDouble),
    InstantaneousCurrentL1(UFixedInteger),
    InstantaneousCurrentL2(UFixedInteger),
    InstantaneousCurrentL3(UFixedInteger),
    InstantaneousActivePowerPlusL1(UFixedDouble),
    InstantaneousActivePowerPlusL2(UFixedDouble),
    InstantaneousActivePowerPlusL3(UFixedDouble),
    InstantaneousActivePowerNegL1(UFixedDouble),
    InstantaneousActivePowerNegL2(UFixedDouble),
    InstantaneousActivePowerNegL3(UFixedDouble),
    SlaveDeviceType(u8, UFixedInteger),
    SlaveEquipmentIdentifier(u8, OctetString<'a>),
    SlaveMeterReading(u8, TST, UFixedDouble),
}

impl<'a> OBIS<'a> {
    pub fn parse(line: &'a str) -> Result<OBIS<'a>> {
        let reference_end = line.find('(').ok_or(Error::InvalidFormat)?;
        let (reference, body) = line.split_at(reference_end);

        match reference {
            "1-3:0.2.8" => Ok(OBIS::Version::<'a>(OctetString::parse(body, 2)?)),
            "0-0:1.0.0" => Ok(OBIS::DateTime(TST::parse(body)?)),
            "0-0:96.1.1" => Ok(OBIS::EquipmentIdentifier::<'a>(OctetString::parse_max(
                body, 96,
            )?)),
            "1-0:1.8.1" => Ok(OBIS::MeterReadingToTariff1(UFixedDouble::parse(
                body, 9, 3,
            )?)),
            "1-0:1.8.2" => Ok(OBIS::MeterReadingToTariff2(UFixedDouble::parse(
                body, 9, 3,
            )?)),
            "1-0:2.8.1" => Ok(OBIS::MeterReadingByTariff1(UFixedDouble::parse(
                body, 9, 3,
            )?)),
            "1-0:2.8.2" => Ok(OBIS::MeterReadingByTariff2(UFixedDouble::parse(
                body, 9, 3,
            )?)),
            "0-0:96.14.0" => Ok(OBIS::TariffIndicator::<'a>(OctetString::parse(body, 4)?)),
            "1-0:1.7.0" => Ok(OBIS::PowerDelivered(UFixedDouble::parse(body, 5, 3)?)),
            "1-0:2.7.0" => Ok(OBIS::PowerReceived(UFixedDouble::parse(body, 5, 3)?)),
            "0-0:96.7.21" => Ok(OBIS::PowerFailures(UFixedInteger::parse(body, 5)?)),
            "0-0:96.7.9" => Ok(OBIS::LongPowerFailures(UFixedInteger::parse(body, 5)?)),
            "1-0:99.97.0" => Ok(OBIS::PowerFailureEventLog), // TODO
            "1-0:32.32.0" => Ok(OBIS::VoltageSagsL1(UFixedInteger::parse(body, 5)?)),
            "1-0:52.32.0" => Ok(OBIS::VoltageSagsL2(UFixedInteger::parse(body, 5)?)),
            "1-0:72.32.0" => Ok(OBIS::VoltageSagsL3(UFixedInteger::parse(body, 5)?)),
            "1-0:32.36.0" => Ok(OBIS::VoltageSwellsL1(UFixedInteger::parse(body, 5)?)),
            "1-0:52.36.0" => Ok(OBIS::VoltageSwellsL2(UFixedInteger::parse(body, 5)?)),
            "1-0:72.36.0" => Ok(OBIS::VoltageSwellsL3(UFixedInteger::parse(body, 5)?)),
            "0-0:96.13.0" => Ok(OBIS::TextMessage), // TODO
            "1-0:32.7.0" => Ok(OBIS::InstantaneousVoltageL1(UFixedDouble::parse(
                body, 4, 1,
            )?)),
            "1-0:52.7.0" => Ok(OBIS::InstantaneousVoltageL2(UFixedDouble::parse(
                body, 4, 1,
            )?)),
            "1-0:72.7.0" => Ok(OBIS::InstantaneousVoltageL3(UFixedDouble::parse(
                body, 4, 1,
            )?)),
            "1-0:31.7.0" => Ok(OBIS::InstantaneousCurrentL1(UFixedInteger::parse(body, 3)?)),
            "1-0:51.7.0" => Ok(OBIS::InstantaneousCurrentL2(UFixedInteger::parse(body, 3)?)),
            "1-0:71.7.0" => Ok(OBIS::InstantaneousCurrentL3(UFixedInteger::parse(body, 3)?)),
            "1-0:21.7.0" => Ok(OBIS::InstantaneousActivePowerPlusL1(UFixedDouble::parse(
                body, 5, 3,
            )?)),
            "1-0:41.7.0" => Ok(OBIS::InstantaneousActivePowerPlusL2(UFixedDouble::parse(
                body, 5, 3,
            )?)),
            "1-0:61.7.0" => Ok(OBIS::InstantaneousActivePowerPlusL3(UFixedDouble::parse(
                body, 5, 3,
            )?)),
            "1-0:22.7.0" => Ok(OBIS::InstantaneousActivePowerNegL1(UFixedDouble::parse(
                body, 5, 3,
            )?)),
            "1-0:42.7.0" => Ok(OBIS::InstantaneousActivePowerNegL2(UFixedDouble::parse(
                body, 5, 3,
            )?)),
            "1-0:62.7.0" => Ok(OBIS::InstantaneousActivePowerNegL3(UFixedDouble::parse(
                body, 5, 3,
            )?)),
            _ => {
                if reference.len() != 10 || reference.get(..2).ok_or(Error::InvalidFormat)? != "0-"
                {
                    return Err(Error::UnknownObis);
                }

                let channel =
                    u8::from_str_radix(&reference[2..=2], 10).map_err(|_| Error::InvalidFormat)?;
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
                        let end = body[1..].find("(").ok_or(Error::InvalidFormat)?;
                        let (time, measurement) = body.split_at(end + 1);

                        let period = measurement.find(".").ok_or(Error::InvalidFormat)?;

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
