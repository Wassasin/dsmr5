#[derive(Debug)]
pub enum Error {
    InvalidFormat,
    InvalidChecksum,
}

pub type Result<T> = core::result::Result<T, Error>;

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
        let (buffer, postfix) = buffer.split_at(data_end+1);

        let given_checksum = u16::from_str_radix(postfix.get(..4).ok_or(Error::InvalidFormat)?, 16).map_err(|_| Error::InvalidFormat)?;
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
            object_buffer: data.get(4..data.len()-3).ok_or(Error::InvalidFormat)?,
        })
    }
}

pub struct Telegram<'a> {
    prefix: &'a str,
    identification: &'a str,
    object_buffer: &'a str,
}

#[cfg(test)]
mod tests {
    #[test]
    fn tryit() {
        let mut buffer = [0u8; 1024];
        let file = std::fs::read("test/telegram.txt").unwrap();
        
        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout {
            buffer,
        };

        readout.to_telegram().unwrap();
    }
}
