use crate::{
    obis::{dsmr4, Line, Parseable, Slave},
    types::*,
    Error, Result,
};

#[derive(Debug)]
pub enum OBIS<'a> {
    DSMR4(dsmr4::OBIS<'a>),
    InstantaneousCurrent(Line, UFixedDouble),
    /// A reading that is not temperature corrected
    SlaveMeterReadingNonCorrected(Slave, TST, UFixedDouble),
    SlaveValveState(Slave, UFixedInteger),
    BreakerState(UFixedInteger),
    LimiterThreshold(UFixedDouble),
    FuseSupervisionThreshold(UFixedInteger),
    CurrentAverageDemand(UFixedDouble),
    MaximumDemandMonth(TST, UFixedDouble),
    MaximumDemandYear,
}

impl<'a> OBIS<'a> {
    fn parse_specific(reference: &'a str, body: &'a str) -> Result<OBIS<'a>> {
        use Line::*;

        match reference {
            "0-0:96.1.1" => Ok(Self::DSMR4(dsmr4::OBIS::<'a>::EquipmentIdentifier(
                OctetString::parse_max(body, 96)?,
            ))),
            "0-0:96.1.4" => Ok(Self::DSMR4(dsmr4::OBIS::<'a>::Version(OctetString::parse(
                body, 5,
            )?))),
            "1-0:31.7.0" => Ok(Self::InstantaneousCurrent(
                Line1,
                UFixedDouble::parse(body, 5, 2)?,
            )),
            "1-0:51.7.0" => Ok(Self::InstantaneousCurrent(
                Line2,
                UFixedDouble::parse(body, 5, 2)?,
            )),
            "1-0:71.7.0" => Ok(Self::InstantaneousCurrent(
                Line3,
                UFixedDouble::parse(body, 5, 2)?,
            )),
            "0-0:96.3.10" => Ok(Self::BreakerState(UFixedInteger::parse(body, 1)?)),
            "0-0:17.0.0" => Ok(Self::LimiterThreshold(UFixedDouble::parse(body, 4, 1)?)),
            "1-0:31.4.0" => Ok(Self::FuseSupervisionThreshold(UFixedInteger::parse(
                body, 3,
            )?)),
            "1-0:1.4.0" => Ok(Self::CurrentAverageDemand(UFixedDouble::parse(body, 5, 3)?)),
            "1-0:1.6.0" => {
                let end = body[1..].find('(').ok_or(Error::InvalidFormat)?;
                let (time, measurement) = body.split_at(end + 1);

                Ok(Self::MaximumDemandMonth(
                    TST::parse(time)?,
                    UFixedDouble::parse(measurement, 5, 3)?,
                ))
            }
            "0-0:98.1.0" => Ok(Self::MaximumDemandYear),
            _ => {
                if reference.len() != 10 || reference.get(..2).ok_or(Error::InvalidFormat)? != "0-"
                {
                    return Err(Error::UnknownObis);
                }

                let channel = reference[2..=2]
                    .parse::<u8>()
                    .map_err(|_| Error::InvalidFormat)?;

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
                    "96.1.1" => Ok(Self::DSMR4(dsmr4::OBIS::<'a>::SlaveEquipmentIdentifier(
                        channel,
                        OctetString::parse_max(body, 96)?,
                    ))),
                    "24.4.0" => Ok(Self::SlaveValveState(
                        channel,
                        UFixedInteger::parse(body, 1)?,
                    )),
                    "24.2.3" => {
                        let end = body[1..].find('(').ok_or(Error::InvalidFormat)?;
                        let (time, measurement) = body.split_at(end + 1);

                        let period = measurement.find('.').ok_or(Error::InvalidFormat)?;

                        Ok(Self::SlaveMeterReadingNonCorrected(
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

impl<'a> Parseable<'a> for OBIS<'a> {
    fn parse(reference: &'a str, body: &'a str) -> Result<Self> {
        match Self::parse_specific(reference, body) {
            Ok(obis) => return Ok(obis),
            Err(Error::UnknownObis) => dsmr4::OBIS::<'a>::parse(reference, body).map(Self::DSMR4),
            Err(e) => return Err(e),
        }
    }
}
