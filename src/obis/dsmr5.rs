use crate::{
    obis::{dsmr4, Line, Message, Parseable, Slave},
    types::*,
    Error, Result,
};

#[derive(Debug)]
pub enum OBIS<'a> {
    DSMR4(dsmr4::OBIS<'a>),
    InstantaneousVoltage(Line, UFixedDouble),
}

impl<'a> From<dsmr4::OBIS<'a>> for OBIS<'a> {
    fn from(o: dsmr4::OBIS<'a>) -> Self {
        Self::DSMR4(o)
    }
}

impl<'a> OBIS<'a> {
    fn parse_specific(reference: &'a str, body: &'a str) -> Result<OBIS<'a>> {
        use Line::*;

        match reference {
            "1-0:32.7.0" => Ok(Self::InstantaneousVoltage(
                Line1,
                UFixedDouble::parse(body, 4, 1)?,
            )),
            "1-0:52.7.0" => Ok(Self::InstantaneousVoltage(
                Line2,
                UFixedDouble::parse(body, 4, 1)?,
            )),
            "1-0:72.7.0" => Ok(Self::InstantaneousVoltage(
                Line3,
                UFixedDouble::parse(body, 4, 1)?,
            )),
            _ => Err(Error::UnknownObis),
        }
    }
}

impl<'a> Parseable<'a> for OBIS<'a> {
    fn parse(reference: &'a str, body: &'a str) -> Result<Self> {
        match Self::parse_specific(reference, body) {
            Ok(obis) => Ok(obis),
            Err(Error::UnknownObis) => dsmr4::OBIS::<'a>::parse(reference, body).map(Self::DSMR4),
            Err(e) => Err(e),
        }
    }
}

impl<'a> Message for OBIS<'a> {
    fn line(&self) -> Option<Line> {
        Some(*match self {
            OBIS::DSMR4(o) => return o.line(),
            OBIS::InstantaneousVoltage(l, _) => l,
        })
    }

    fn slave(&self) -> Option<Slave> {
        match self {
            OBIS::DSMR4(o) => o.slave(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::obis::Tariff;

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

        use crate::obis::dsmr5::OBIS;
        telegram.objects::<OBIS>().for_each(|o| {
            println!("{:?}", o); // to see use `$ cargo test -- --nocapture`
            let o = o.unwrap();

            use OBIS::*;
            match o {
                DSMR4(o) => {
                    use crate::obis::dsmr4::OBIS::*;
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
                        MeterReadingTo(Tariff::Tariff1, mr) => {
                            assert_eq!(f64::from(&mr), 576.239);
                        }
                        MeterReadingTo(Tariff::Tariff2, mr) => {
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
                }
                _ => (), // Do not test the rest.
            }
        });
    }
}
