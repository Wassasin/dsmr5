#[derive(Debug)]
pub enum Error {
    InvalidFormat,
    InvalidChecksum,
    UnknownObis,
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct OctetString<'a>(&'a str);

impl<'a> OctetString<'a> {
    pub fn parse(body: &'a str, length: usize) -> Result<OctetString<'a>> {
        Ok(OctetString(
            body.get(1..length + 1).ok_or(Error::InvalidFormat)?,
        ))
    }

    pub fn parse_max(body: &'a str, max_length: usize) -> Result<OctetString<'a>> {
        let end = body.find(')').ok_or(Error::InvalidFormat)? - 1;
        if end > max_length {
            return Err(Error::InvalidFormat);
        }

        OctetString::parse(body, end)
    }
}

#[derive(Debug)]
pub struct TST {
    year: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    dst: bool,
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

#[derive(Debug)]
pub struct FixedFloat {
    buffer: u64,
    point: u8,
}

impl FixedFloat {
    pub fn parse(body: &str, length: usize, point: u8) -> Result<FixedFloat> {
        // Do not forget the extra '.'
        let buffer = body.get(1..length + 2).ok_or(Error::InvalidFormat)?;
        let (upper, lower) = buffer.split_at(length - point as usize);

        let upper = u64::from_str_radix(upper, 10).map_err(|_| Error::InvalidFormat)?;
        let lower = u64::from_str_radix(&lower[1..], 10).map_err(|_| Error::InvalidFormat)?;

        Ok(FixedFloat {
            buffer: upper*10u64.pow(point as u32) + lower,
            point,
        })
    }
}

#[derive(Debug)]
pub struct FixedInteger(u64);

impl FixedInteger {
    pub fn parse(body: &str, length: usize) -> Result<FixedInteger> {
        let buffer = body.get(1..length + 1).ok_or(Error::InvalidFormat)?;
        let number = u64::from_str_radix(buffer, 10).map_err(|_| Error::InvalidFormat)?;

        Ok(FixedInteger(number))
    }
}

#[derive(Debug)]
pub enum OBIS<'a> {
    Version(OctetString<'a>),
    DateTime(TST),
    EquipmentIdentifier(OctetString<'a>),
    MeterReadingToTariff1(FixedFloat),
    MeterReadingToTariff2(FixedFloat),
    MeterReadingByTariff1(FixedFloat),
    MeterReadingByTariff2(FixedFloat),
    TariffIndicator(OctetString<'a>),
    PowerDelivered(FixedFloat),
    PowerReceived(FixedFloat),
    PowerFailures(FixedInteger),
    LongPowerFailures(FixedInteger),
    PowerFailureEventLog, // TODO
    VoltageSagsL1(FixedInteger),
    VoltageSagsL2(FixedInteger),
    VoltageSagsL3(FixedInteger),
    VoltageSwellsL1(FixedInteger),
    VoltageSwellsL2(FixedInteger),
    VoltageSwellsL3(FixedInteger),
    TextMessage, // TODO
    InstantaneousVoltageL1(FixedFloat),
    InstantaneousVoltageL2(FixedFloat),
    InstantaneousVoltageL3(FixedFloat),
    InstantaneousCurrentL1(FixedInteger),
    InstantaneousCurrentL2(FixedInteger),
    InstantaneousCurrentL3(FixedInteger),
    InstantaneousActivePowerPlusL1(FixedFloat),
    InstantaneousActivePowerPlusL2(FixedFloat),
    InstantaneousActivePowerPlusL3(FixedFloat),
    InstantaneousActivePowerNegL1(FixedFloat),
    InstantaneousActivePowerNegL2(FixedFloat),
    InstantaneousActivePowerNegL3(FixedFloat),
    SlaveDeviceType(u8, FixedInteger),
    SlaveEquipmentIdentifier(u8, OctetString<'a>),
    SlaveMeterReading(u8, TST, FixedFloat),
}

impl<'a> OBIS<'a> {
    fn parse(line: &'a str) -> Result<OBIS<'a>> {
        let reference_end = line.find('(').ok_or(Error::InvalidFormat)?;
        let (reference, body) = line.split_at(reference_end);

        match reference {
            "1-3:0.2.8" => Ok(OBIS::Version::<'a>(OctetString::parse(body, 2)?)),
            "0-0:1.0.0" => Ok(OBIS::DateTime(TST::parse(body)?)),
            "0-0:96.1.1" => Ok(OBIS::EquipmentIdentifier::<'a>(OctetString::parse_max(body, 96)?)),
            "1-0:1.8.1" => Ok(OBIS::MeterReadingToTariff1(FixedFloat::parse(body, 9, 3)?)),
            "1-0:1.8.2" => Ok(OBIS::MeterReadingToTariff2(FixedFloat::parse(body, 9, 3)?)),
            "1-0:2.8.1" => Ok(OBIS::MeterReadingByTariff1(FixedFloat::parse(body, 9, 3)?)),
            "1-0:2.8.2" => Ok(OBIS::MeterReadingByTariff2(FixedFloat::parse(body, 9, 3)?)),
            "0-0:96.14.0" => Ok(OBIS::TariffIndicator::<'a>(OctetString::parse(body, 4)?)),
            "1-0:1.7.0" => Ok(OBIS::PowerDelivered(FixedFloat::parse(body, 5, 3)?)),
            "1-0:2.7.0" => Ok(OBIS::PowerReceived(FixedFloat::parse(body, 5, 3)?)),
            "0-0:96.7.21" => Ok(OBIS::PowerFailures(FixedInteger::parse(body, 5)?)),
            "0-0:96.7.9" => Ok(OBIS::LongPowerFailures(FixedInteger::parse(body, 5)?)),
            "1-0:99.97.0" => Ok(OBIS::PowerFailureEventLog), // TODO
            "1-0:32.32.0" => Ok(OBIS::VoltageSagsL1(FixedInteger::parse(body, 5)?)),
            "1-0:52.32.0" => Ok(OBIS::VoltageSagsL2(FixedInteger::parse(body, 5)?)),
            "1-0:72.32.0" => Ok(OBIS::VoltageSagsL3(FixedInteger::parse(body, 5)?)),
            "1-0:32.36.0" => Ok(OBIS::VoltageSwellsL1(FixedInteger::parse(body, 5)?)),
            "1-0:52.36.0" => Ok(OBIS::VoltageSwellsL2(FixedInteger::parse(body, 5)?)),
            "1-0:72.36.0" => Ok(OBIS::VoltageSwellsL3(FixedInteger::parse(body, 5)?)),
            "0-0:96.13.0" => Ok(OBIS::TextMessage), // TODO
            "1-0:32.7.0" => Ok(OBIS::InstantaneousVoltageL1(FixedFloat::parse(body, 4, 1)?)),
            "1-0:52.7.0" => Ok(OBIS::InstantaneousVoltageL2(FixedFloat::parse(body, 4, 1)?)),
            "1-0:72.7.0" => Ok(OBIS::InstantaneousVoltageL3(FixedFloat::parse(body, 4, 1)?)),
            "1-0:31.7.0" => Ok(OBIS::InstantaneousCurrentL1(FixedInteger::parse(body, 3)?)),
            "1-0:51.7.0" => Ok(OBIS::InstantaneousCurrentL2(FixedInteger::parse(body, 3)?)),
            "1-0:71.7.0" => Ok(OBIS::InstantaneousCurrentL3(FixedInteger::parse(body, 3)?)),
            "1-0:21.7.0" => Ok(OBIS::InstantaneousActivePowerPlusL1(FixedFloat::parse(body, 5, 3)?)),
            "1-0:41.7.0" => Ok(OBIS::InstantaneousActivePowerPlusL2(FixedFloat::parse(body, 5, 3)?)),
            "1-0:61.7.0" => Ok(OBIS::InstantaneousActivePowerPlusL3(FixedFloat::parse(body, 5, 3)?)),
            "1-0:22.7.0" => Ok(OBIS::InstantaneousActivePowerNegL1(FixedFloat::parse(body, 5, 3)?)),
            "1-0:42.7.0" => Ok(OBIS::InstantaneousActivePowerNegL2(FixedFloat::parse(body, 5, 3)?)),
            "1-0:62.7.0" => Ok(OBIS::InstantaneousActivePowerNegL3(FixedFloat::parse(body, 5, 3)?)),
            _ => {
                if reference.len() != 10 || reference.get(..2).ok_or(Error::InvalidFormat)? != "0-" {
                    return Err(Error::UnknownObis);
                }

                let channel = u8::from_str_radix(&reference[2..=2], 10).map_err(|_| Error::InvalidFormat)?;
                let subreference = &reference[4..];

                match subreference {
                    "24.1.0" => Ok(OBIS::SlaveDeviceType(channel, FixedInteger::parse(body, 3)?)),
                    "96.1.0" => Ok(OBIS::SlaveEquipmentIdentifier::<'a>(channel, OctetString::parse_max(body, 96)?)),
                    "24.2.1" => {
                        let end = body[1..].find("(").ok_or(Error::InvalidFormat)?;
                        let (time, measurement) = body.split_at(end+1);

                        let period = measurement.find(".").ok_or(Error::InvalidFormat)?;

                        Ok(OBIS::SlaveMeterReading(
                            channel,
                            TST::parse(time)?,
                            FixedFloat::parse(measurement, 8, 9 - period as u8)?,
                        ))
                    },
                    _ => Err(Error::UnknownObis),
                }
            },
        }
    }
}

pub struct Readout {
    pub buffer: [u8; 1024], // Maximum size of a Readout
}

impl Readout {
    pub fn to_telegram<'a>(&'a self) -> Result<Telegram<'a>> {
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
            prefix,
            identification,
            object_buffer: data.get(4..data.len() - 3).ok_or(Error::InvalidFormat)?,
        })
    }
}

pub struct Telegram<'a> {
    pub prefix: &'a str,
    pub identification: &'a str,
    object_buffer: &'a str,
}

impl<'a> Telegram<'a> {
    pub fn objects(&self) -> impl core::iter::Iterator<Item = Result<OBIS<'a>>> {
        self.object_buffer.lines().map(OBIS::parse)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tryit() {
        let mut buffer = [0u8; 1024];
        let file = std::fs::read("test/telegram.txt").unwrap();

        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout { buffer };

        let telegram = readout.to_telegram().unwrap();

        assert_eq!(telegram.prefix, "ISK");
        assert_eq!(telegram.identification, "\\2M550E-1012");

        telegram.objects().for_each(|o| {
            println!("{:?}", o);
        });
    }
}
