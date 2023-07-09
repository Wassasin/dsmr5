use crate::Readout;

/// A blocking Iterator that parses a bytestreaming Iterator to Readouts.
pub struct Reader<T: core::iter::Iterator<Item = Result<u8, E>>, E> {
    stream: T,
}

impl<T: core::iter::Iterator<Item = Result<u8, E>>, E> Reader<T, E> {
    pub fn new(stream: T) -> Self {
        Reader { stream }
    }
}

#[derive(Debug)]
pub enum ReaderError<E> {
    IOError(E),
    BufferOverFlow,
}

impl<T: core::iter::Iterator<Item = Result<u8, E>>, E> Iterator for Reader<T, E> {
    type Item = Result<Readout, ReaderError<E>>;

    /// Generates Readout by blocking on the underlying byte iterator until
    /// a full Readout was passed.
    ///
    /// Will ignore all bytes until the first Readout is spotted.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stream.next()? {
                Ok(b'/') => break,
                Ok(_) => {}
                Err(e) => return Some(Err(ReaderError::IOError(e))),
            }
        }

        let mut buffer = [0u8; 2048];
        buffer[0] = b'/';

        let mut i = 1;
        let mut copy_byte = || match self.stream.next()? {
            Ok(b) => {
                if i >= buffer.len() {
                    return Some(Err(ReaderError::BufferOverFlow));
                }

                buffer[i] = b;
                i += 1;
                Some(Ok(b))
            }
            Err(e) => return Some(Err(ReaderError::IOError(e))),
        };

        loop {
            match copy_byte()? {
                Ok(b'!') => {
                    // Add CRC bytes
                    for _ in 0..4 {
                        if let Err(e) = copy_byte()? {
                            return Some(Err(e));
                        }
                    }
                    return Some(Ok(Readout { buffer }));
                }
                Ok(_) => {}
                Err(e) => return Some(Err(e)),
            };
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

        let mut bytes = f1.chain(f2).chain(f3);

        // Spool some bytes such that the stream starts midway.
        for _ in 0..100 {
            bytes.next().unwrap().unwrap();
        }

        let mut reader = crate::Reader::new(bytes);

        // Instantiating telegrams forces CRC to be checked.
        let _t1 = reader.next().unwrap().unwrap().to_telegram().unwrap();
        let _t2 = reader.next().unwrap().unwrap().to_telegram().unwrap();

        // We only have two messages until stream is terminated.
        assert!(reader.next().is_none());
    }

    #[test]
    fn recover_from_overflow() {
        use std::io::Read;

        let f1 = std::fs::File::open("test/isk.txt").unwrap().bytes();
        let f2 = std::fs::File::open("test/overflow.txt").unwrap().bytes();
        let f3 = std::fs::File::open("test/isk.txt").unwrap().bytes();
        let bytes = f1.chain(f2).chain(f3);

        let mut reader = crate::Reader::new(bytes);

        let t1 = reader.next();
        let t2 = reader.next();
        let t3 = reader.next();
        let t4 = reader.next();

        assert!(matches!(t1, Some(Ok(_))));
        assert!(matches!(t2, Some(Err(crate::ReaderError::BufferOverFlow))));
        assert!(matches!(t3, Some(Ok(_))));
        assert!(matches!(t4, None));
    }
}
