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
pub enum OBIS<'a> {
    Version(OctetString<'a>),
    DateTime(TST),
    EquipmentIdentifier,
    MeterReadingToTariff1,
    MeterReadingToTariff2,
    MeterReadingByTariff1,
    MeterReadingByTariff2,
    TariffIndicator,
    PowerDelivered,
    PowerReceived,
    PowerFailures,
    LongPowerFailures,
    PowerFailureEventLog,
    VoltageSagsL1,
    VoltageSagsL2,
    VoltageSagsL3,
    VoltageSwellsL1,
    VoltageSwellsL2,
    VoltageSwellsL3,
    TextMessage,
    InstantaneousVoltageL1,
    InstantaneousVoltageL2,
    InstantaneousVoltageL3,
    InstantaneousCurrentL1,
    InstantaneousCurrentL2,
    InstantaneousCurrentL3,
    InstantaneousActivePowerPlusL1,
    InstantaneousActivePowerPlusL2,
    InstantaneousActivePowerPlusL3,
    InstantaneousActivePowerNegL1,
    InstantaneousActivePowerNegL2,
    InstantaneousActivePowerNegL3,
    SlaveDeviceType(u8),
    SlaveEquipmentIdentifier(u8),
    SlaveMeterReading(u8),
}

impl<'a> OBIS<'a> {
    fn parse(line: &'a str) -> Result<OBIS<'a>> {
        let reference_end = line.find('(').ok_or(Error::InvalidFormat)?;
        let (reference, body) = line.split_at(reference_end);

        match reference {
            "1-3:0.2.8" => Ok(OBIS::Version::<'a>(OctetString::parse(body, 2)?)),
            "0-0:1.0.0" => Ok(OBIS::DateTime(TST::parse(body)?)),
            _ => Err(Error::UnknownObis),
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

        let objects: Vec<crate::Result<crate::OBIS>> = telegram.objects().collect();
        println!("{:?}", objects);
    }
}
