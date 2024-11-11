use std::io::BufRead;
use std::io::Error;
use std::io::Read;

use arbtest::arbtest;

// Reads at most `len` bytes.
pub struct NBytesReader<R: Read> {
    reader: R,
    buf: Vec<u8>,
    len: usize,
}

impl<R: Read> NBytesReader<R> {
    pub fn new(reader: R, len: usize) -> Self {
        Self {
            reader,
            buf: Default::default(),
            len,
        }
    }
}

impl<R: Read> Read for NBytesReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self.buf.is_empty() {
            self.fill_buf()?;
        }
        let n = buf.len().min(self.buf.len());
        buf[..n].copy_from_slice(&self.buf[..n]);
        self.consume(n);
        Ok(n)
    }
}

impl<R: Read> BufRead for NBytesReader<R> {
    fn fill_buf(&mut self) -> Result<&[u8], Error> {
        if !self.buf.is_empty() {
            return Ok(&self.buf[..]);
        }
        self.buf.resize(self.len, 0_u8);
        let n = self.reader.read(&mut self.buf[..])?;
        self.buf.truncate(n);
        Ok(&self.buf[..n])
    }

    fn consume(&mut self, n: usize) {
        self.buf.drain(..n);
    }
}

#[test]
fn test_n_byte_reader() {
    arbtest(|u| {
        let expected: Vec<u8> = u.arbitrary()?;
        let capacity = u.int_in_range(1..=4096)?;
        let mut reader = NBytesReader::new(&expected[..], capacity);
        let mut actual: Vec<u8> = Vec::new();
        loop {
            let buf = reader.fill_buf().unwrap();
            let n = buf.len();
            if buf.is_empty() {
                break;
            }
            actual.extend(buf);
            reader.consume(n);
        }
        assert_eq!(expected, actual);
        Ok(())
    });
}
