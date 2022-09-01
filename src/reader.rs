use crate::Readout;

/// A blocking Iterator that parses a bytestreaming Iterator to Readouts.
pub struct Reader<T: core::iter::Iterator<Item = u8>> {
    stream: T,
}

impl<T: core::iter::Iterator<Item = u8>> Reader<T> {
    pub fn new(stream: T) -> Self {
        Reader { stream }
    }
}

impl<T: core::iter::Iterator<Item = u8>> Iterator for Reader<T> {
    type Item = Readout;

    /// Generates Readout by blocking on the underlying byte iterator until
    /// a full Readout was passed.
    ///
    /// Will ignore all bytes until the first Readout is spotted.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let b = self.stream.next()?;
            if b == b'/' {
                break;
            }
        }

        let mut buffer = [0u8; 2048];
        buffer[0] = b'/';

        let mut i = 1;
        let mut write = |b| {
            if i >= buffer.len() {
                return None; // Buffer overflow.
            }

            buffer[i] = b;
            i += 1;

            Some(())
        };

        loop {
            let b = self.stream.next()?;
            write(b)?;

            if b == b'!' {
                // Add CRC bytes
                for _ in 0..4 {
                    write(self.stream.next()?)?;
                }

                return Some(Readout { buffer });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reader() {
        use std::io::Read;

        let f1 = std::fs::File::open("test/isk.txt").unwrap().bytes();
        let f2 = std::fs::File::open("test/isk.txt").unwrap().bytes();
        let f3 = std::fs::File::open("test/isk.txt").unwrap().bytes();

        let mut bytes = f1.chain(f2).chain(f3).map(|b| b.unwrap());

        // Spool some bytes such that the stream starts midway.
        for _ in 0..100 {
            bytes.next().unwrap();
        }

        let mut reader = crate::Reader::new(bytes);

        // Instantiating telegrams forces CRC to be checked.
        let _t1 = reader.next().unwrap().to_telegram().unwrap();
        let _t2 = reader.next().unwrap().to_telegram().unwrap();

        // We only have two messages until stream is terminated.
        assert!(reader.next().is_none());
    }
}
