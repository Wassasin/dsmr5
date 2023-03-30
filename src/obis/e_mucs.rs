use crate::{
    obis::{dsmr4, dsmr5, Line, Message, Parseable, Slave},
    types::*,
    Error, Result,
};

#[derive(Debug)]
pub enum OBIS<'a> {
    DSMR5(dsmr5::OBIS<'a>),
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

impl<'a> From<dsmr5::OBIS<'a>> for OBIS<'a> {
    fn from(o: dsmr5::OBIS<'a>) -> Self {
        Self::DSMR5(o)
    }
}

impl<'a> OBIS<'a> {
    fn parse_specific(reference: &'a str, body: &'a str) -> Result<OBIS<'a>> {
        use Line::*;

        match reference {
            "0-0:96.1.4" => Ok(Self::from(dsmr5::OBIS::from(dsmr4::OBIS::Version::<'a>(
                OctetString::parse(body, 5)?,
            )))),
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
                if reference.len() != 10 || reference.get(..2).ok_or(Error::UnknownObis)? != "0-" {
                    return Err(Error::UnknownObis);
                }

                let channel = reference[2..=2]
                    .parse::<u8>()
                    .map_err(|_| Error::UnknownObis)?;

                use Slave::*;
                let channel = match channel {
                    1 => Ok(Slave1),
                    2 => Ok(Slave2),
                    3 => Ok(Slave3),
                    4 => Ok(Slave4),
                    _ => Err(Error::UnknownObis),
                }?;
                let subreference = &reference[4..];

                match subreference {
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
                    "96.1.1" => Ok(Self::from(dsmr5::OBIS::from(
                        dsmr4::OBIS::SlaveEquipmentIdentifier::<'a>(
                            channel,
                            OctetString::parse_max(body, 96)?,
                        ),
                    ))),
                    _ => Err(Error::UnknownObis),
                }
            }
        }
    }
}

impl<'a> Parseable<'a> for OBIS<'a> {
    fn parse(reference: &'a str, body: &'a str) -> Result<Self> {
        match Self::parse_specific(reference, body) {
            Ok(obis) => Ok(obis),
            Err(Error::UnknownObis) => dsmr5::OBIS::<'a>::parse(reference, body).map(Self::DSMR5),
            Err(e) => Err(e),
        }
    }
}

impl<'a> Message for OBIS<'a> {
    fn line(&self) -> Option<Line> {
        Some(*match self {
            OBIS::DSMR5(o) => return o.line(),
            OBIS::InstantaneousCurrent(l, _) => l,
            _ => return None,
        })
    }

    fn slave(&self) -> Option<Slave> {
        Some(*match self {
            OBIS::DSMR5(o) => return o.slave(),
            OBIS::SlaveMeterReadingNonCorrected(s, _, _) => s,
            OBIS::SlaveValveState(s, _) => s,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::obis::{Line, Tariff};

    #[test]
    fn example_flu() {
        let mut buffer = [0u8; 2048];
        let file = std::fs::read("test/flu.txt").unwrap();

        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout { buffer };
        let telegram = readout.to_telegram().unwrap();

        assert_eq!(telegram.prefix, "FLU");
        assert_eq!(telegram.identification, "\\253770234_A");

        telegram
            .objects::<crate::obis::e_mucs::OBIS>()
            .for_each(|o| {
                println!("{:?}", o); // to see use `$ cargo test -- --nocapture`
                let o = o.unwrap();

                use crate::obis::e_mucs::OBIS::*;
                use core::convert::From;
                match o {
                    DSMR5(o) => {
                        use crate::obis::dsmr5::OBIS::*;
                        match o {
                            DSMR4(o) => {
                                use crate::obis::dsmr4::OBIS::*;
                                match o {
                                    Version(v) => {
                                        let b: std::vec::Vec<u8> =
                                            v.as_octets().map(|b| b.unwrap()).collect();
                                        assert_eq!(b, [80, 33]);
                                    }
                                    DateTime(tst) => {
                                        assert_eq!(
                                            tst,
                                            crate::types::TST {
                                                year: 23,
                                                month: 2,
                                                day: 11,
                                                hour: 11,
                                                minute: 15,
                                                second: 41,
                                                dst: false
                                            }
                                        );
                                    }
                                    EquipmentIdentifier(ei) => {
                                        let b: std::vec::Vec<u8> =
                                            ei.as_octets().map(|b| b.unwrap()).collect();
                                        assert_eq!(
                                            std::str::from_utf8(&b).unwrap(),
                                            "1SAG1105067226"
                                        );
                                    }
                                    MeterReadingTo(Tariff::Tariff1, mr) => {
                                        assert_eq!(f64::from(&mr), 1114.057);
                                    }
                                    MeterReadingTo(Tariff::Tariff2, mr) => {
                                        assert_eq!(f64::from(&mr), 997.282);
                                    }
                                    TariffIndicator(ti) => {
                                        let b: std::vec::Vec<u8> =
                                            ti.as_octets().map(|b| b.unwrap()).collect();
                                        assert_eq!(b, [0, 2]);
                                    }
                                    PowerFailures(crate::types::UFixedInteger(pf)) => {
                                        assert_eq!(pf, 3);
                                    }
                                    _ => (), // Do not test the rest.
                                }
                            }
                            InstantaneousVoltage(Line::Line1, v) => {
                                assert_eq!(f64::from(&v), 1.);
                            }
                            _ => (), // Do not test the rest.
                        }
                    }
                    _ => (), // Do not test the rest.
                }
            });
    }
}
