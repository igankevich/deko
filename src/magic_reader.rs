macro_rules! define_magic_reader {
    ($trait: ident) => {
        use crate::MAX_MAGIC_BYTES;
        #[cfg(feature = "nightly")]
        use std::io::BorrowedCursor;
        use std::io::Error;
        use std::io::ErrorKind;
        use std::io::IoSliceMut;
        use std::io::Read;

        pub struct MagicReader<R> {
            reader: R,
            buf: [u8; MAX_MAGIC_BYTES],
            first: usize,
            last: usize,
        }

        impl<R> MagicReader<R> {
            pub fn new(reader: R) -> Self {
                Self {
                    reader,
                    buf: [0; MAX_MAGIC_BYTES],
                    first: 0,
                    last: 0,
                }
            }

            pub fn get_ref(&self) -> &R {
                &self.reader
            }

            pub fn get_mut(&mut self) -> &mut R {
                &mut self.reader
            }

            pub fn into_inner(self) -> R {
                self.reader
            }

            #[cold]
            fn do_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
                let n = buf.len().min(self.last - self.first);
                buf[..n].copy_from_slice(&self.buf[self.first..(self.first + n)]);
                self.first += n;
                Ok(n)
            }

            #[cfg(feature = "nightly")]
            #[cold]
            fn do_read_buf(&mut self, buf: &mut BorrowedCursor<'_>) -> Result<usize, Error> {
                let n = buf.capacity().min(self.last - self.first);
                buf.append(&self.buf[self.first..(self.first + n)]);
                self.first += n;
                Ok(n)
            }
        }

        crate::define_read_magic!($trait);

        impl<R: Read> Read for MagicReader<R> {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
                if self.first == self.last {
                    self.reader.read(buf)
                } else {
                    let mut n = self.do_read(buf)?;
                    n += self.reader.read(&mut buf[n..])?;
                    Ok(n)
                }
            }

            #[cfg(feature = "nightly")]
            fn is_read_vectored(&self) -> bool {
                self.reader.is_read_vectored()
            }

            fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize, Error> {
                if self.first == self.last {
                    self.reader.read_vectored(bufs)
                } else {
                    // this is the default `read_vectored` implementation from `std` library
                    let buf = bufs
                        .iter_mut()
                        .find(|b| !b.is_empty())
                        .map_or(&mut [][..], |b| &mut **b);
                    self.do_read(buf)
                }
            }

            fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
                let mut n: usize = 0;
                if self.first != self.last {
                    buf.extend(&self.buf[self.first..self.last]);
                    n += self.last - self.first;
                    self.first = self.last;
                }
                n += self.reader.read_to_end(buf)?;
                Ok(n)
            }

            fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Error> {
                if self.first != self.last {
                    let mut bytes = Vec::new();
                    let n = self.read_to_end(&mut bytes)?;
                    let s = std::str::from_utf8(&bytes[..]).map_err(|_| {
                        Error::new(ErrorKind::InvalidData, "stream did not contain valid UTF-8")
                    })?;
                    buf.push_str(s);
                    Ok(n)
                } else {
                    self.reader.read_to_string(buf)
                }
            }

            fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Error> {
                if self.first != self.last {
                    let n = self.do_read(buf)?;
                    buf = &mut buf[n..];
                }
                self.reader.read_exact(buf)
            }

            #[cfg(feature = "nightly")]
            fn read_buf(&mut self, mut buf: BorrowedCursor<'_>) -> Result<(), Error> {
                if self.first != self.last {
                    self.do_read_buf(&mut buf)?;
                }
                self.reader.read_buf(buf)
            }

            #[cfg(feature = "nightly")]
            fn read_buf_exact(&mut self, mut buf: BorrowedCursor<'_>) -> Result<(), Error> {
                if self.first != self.last {
                    self.do_read_buf(&mut buf)?;
                }
                self.reader.read_buf_exact(buf)
            }
        }

        crate::impl_buf_read_for_magic_reader!($trait);

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::test::test_read_trait;
            use crate::test::NBytesReader;
            use arbitrary::Unstructured;
            use std::collections::VecDeque;

            #[test]
            fn test_read() {
                test_read_trait(new_magic_reader);
                test_read_trait(new_magic_reader_v2);
                test_read_trait(new_magic_reader_v3);
            }

            crate::define_magic_reader_tests!($trait);

            fn new_magic_reader(
                vec: VecDeque<u8>,
                _u: &mut Unstructured,
            ) -> MagicReader<VecDeque<u8>> {
                let len = vec.len();
                let mut reader = MagicReader::new(vec);
                let magic = reader.read_magic().unwrap();
                let magic = if magic.len() >= MAX_MAGIC_BYTES {
                    magic
                } else {
                    reader.read_magic_slow().unwrap()
                };
                assert!(
                    len >= magic.len(),
                    "len = {}, magic len = {}",
                    len,
                    magic.len()
                );
                reader
            }

            fn new_magic_reader_v2(
                vec: VecDeque<u8>,
                _u: &mut Unstructured,
            ) -> MagicReader<VecDeque<u8>> {
                MagicReader::new(vec)
            }

            fn new_magic_reader_v3(
                vec: VecDeque<u8>,
                u: &mut Unstructured,
            ) -> MagicReader<NBytesReader<VecDeque<u8>>> {
                let reader = NBytesReader::new(vec, u.int_in_range(1..=100).unwrap());
                MagicReader::new(reader)
            }
        }
    };
}

pub(crate) use define_magic_reader;

macro_rules! define_read_magic {
    (Read) => {
        impl<R: Read> MagicReader<R> {
            pub fn read_magic(&mut self) -> Result<&[u8], Error> {
                let n = self.reader.read(&mut self.buf[self.last..])?;
                self.last += n;
                Ok(&self.buf[..self.last])
            }

            #[cold]
            pub fn read_magic_slow(&mut self) -> Result<&[u8], Error> {
                loop {
                    let n = match self.reader.read(&mut self.buf[self.last..]) {
                        Ok(n) => n,
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e),
                    };
                    if n == 0 {
                        return Ok(&self.buf[..self.last]);
                    }
                    self.last += n;
                    if self.last == MAX_MAGIC_BYTES {
                        return Ok(&self.buf[..]);
                    }
                }
            }
        }
    };
    (BufRead) => {
        impl<R: std::io::BufRead> MagicReader<R> {
            pub fn read_magic(&mut self) -> Result<&[u8], Error> {
                self.reader.fill_buf()
            }

            #[cold]
            pub fn read_magic_slow(&mut self) -> Result<&[u8], Error> {
                loop {
                    let buf = match self.reader.fill_buf() {
                        Ok(buf) => buf,
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e),
                    };
                    let n = buf.len().min(MAX_MAGIC_BYTES - self.last);
                    if n == 0 {
                        return Ok(&self.buf[..self.last]);
                    }
                    self.buf[self.last..(self.last + n)].copy_from_slice(&buf[..n]);
                    self.reader.consume(n);
                    self.last += n;
                    if self.last == MAX_MAGIC_BYTES {
                        return Ok(&self.buf[..]);
                    }
                }
            }
        }
    };
}

pub(crate) use define_read_magic;

macro_rules! impl_buf_read_for_magic_reader {
    (BufRead) => {
        use std::io::BufRead;

        impl<R: BufRead> BufRead for MagicReader<R> {
            fn fill_buf(&mut self) -> Result<&[u8], Error> {
                if self.first == self.last {
                    self.reader.fill_buf()
                } else {
                    Ok(&self.buf[self.first..self.last])
                }
            }

            fn consume(&mut self, n: usize) {
                if self.first == self.last {
                    self.reader.consume(n);
                } else {
                    debug_assert!(self.first + n <= self.last);
                    self.first += n;
                }
            }
        }
    };
    (Read) => {};
}

pub(crate) use impl_buf_read_for_magic_reader;

#[cfg(test)]
macro_rules! define_magic_reader_tests {
    (BufRead) => {
        use crate::test::test_bufread_all;

        #[test]
        fn test_buf_read() {
            test_bufread_all(new_magic_reader);
            test_bufread_all(new_magic_reader_v2);
            test_bufread_all(new_magic_reader_v3);
        }
    };
    (Read) => {};
}

#[cfg(test)]
pub(crate) use define_magic_reader_tests;
