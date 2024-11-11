#[cfg(feature = "nightly")]
use std::io::BorrowedCursor;
use std::io::BufRead;
use std::io::Empty;
use std::io::Error;
use std::io::ErrorKind;
use std::io::IoSliceMut;
use std::io::Read;

use bzip2::bufread::BzDecoder;
use flate2::bufread::GzDecoder;
use flate2::bufread::ZlibDecoder;
use xz::bufread::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

pub struct AnyDecoder<R: BufRead> {
    reader: Option<MagicReader<R>>,
    inner: InnerDecoder<MagicReader<R>>,
}

impl<R: BufRead> AnyDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: Some(MagicReader::new(reader)),
            inner: InnerDecoder::Empty(std::io::empty()),
        }
    }

    pub fn get_ref(&self) -> &R {
        if let Some(r) = self.reader.as_ref() {
            return r.get_ref();
        }
        match self.inner {
            InnerDecoder::Reader(ref r) => r.get_ref(),
            InnerDecoder::Gz(ref r) => r.get_ref().get_ref(),
            InnerDecoder::Bz(ref r) => r.get_ref().get_ref(),
            InnerDecoder::Zlib(ref r) => r.get_ref().get_ref(),
            InnerDecoder::Xz(ref r) => r.get_ref().get_ref(),
            InnerDecoder::Zstd(ref r) => r.get_ref().get_ref(),
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }

    pub fn get_mut(&mut self) -> &R {
        if let Some(r) = self.reader.as_mut() {
            return r.get_mut();
        }
        match self.inner {
            InnerDecoder::Reader(ref mut r) => r.get_mut(),
            InnerDecoder::Gz(ref mut r) => r.get_mut().get_mut(),
            InnerDecoder::Bz(ref mut r) => r.get_mut().get_mut(),
            InnerDecoder::Zlib(ref mut r) => r.get_mut().get_mut(),
            InnerDecoder::Xz(ref mut r) => r.get_mut().get_mut(),
            InnerDecoder::Zstd(ref mut r) => r.get_mut().get_mut(),
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }

    pub fn into_inner(mut self) -> R {
        if let Some(r) = self.reader.take() {
            return r.reader;
        }
        match self.inner {
            InnerDecoder::Reader(r) => r.into_inner(),
            InnerDecoder::Gz(r) => r.into_inner().into_inner(),
            InnerDecoder::Bz(r) => r.into_inner().into_inner(),
            InnerDecoder::Zlib(r) => r.into_inner().into_inner(),
            InnerDecoder::Xz(r) => r.into_inner().into_inner(),
            InnerDecoder::Zstd(r) => r.finish().into_inner(),
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }

    #[inline]
    fn detect_decoder(&mut self) -> Result<(), Error> {
        if let Some(r) = self.reader.take() {
            self.inner = InnerDecoder::new(r)?;
        }
        Ok(())
    }
}

impl<R: BufRead> Read for AnyDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.detect_decoder()?;
        dispatch_mut!(self.inner, Read::read, buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize, Error> {
        self.detect_decoder()?;
        dispatch_mut!(self.inner, Read::read_vectored, bufs)
    }

    #[cfg(feature = "nightly")]
    fn is_read_vectored(&self) -> bool {
        dispatch!(self.inner, Read::is_read_vectored)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        self.detect_decoder()?;
        dispatch_mut!(self.inner, Read::read_to_end, buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Error> {
        self.detect_decoder()?;
        dispatch_mut!(self.inner, Read::read_to_string, buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.detect_decoder()?;
        dispatch_mut!(self.inner, Read::read_exact, buf)
    }

    #[cfg(feature = "nightly")]
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> Result<(), Error> {
        self.detect_decoder()?;
        dispatch_mut!(self.inner, Read::read_buf, buf)
    }

    #[cfg(feature = "nightly")]
    fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> Result<(), Error> {
        self.detect_decoder()?;
        dispatch_mut!(self.inner, Read::read_buf_exact, buf)
    }
}

struct MagicReader<R: BufRead> {
    reader: R,
    buf: [u8; MAX_MAGIC_BYTES],
    first: usize,
    last: usize,
}

impl<R: BufRead> MagicReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            buf: [0; MAX_MAGIC_BYTES],
            first: 0,
            last: 0,
        }
    }

    fn read_magic(&mut self) -> Result<&[u8], Error> {
        self.reader.fill_buf()
    }

    #[cold]
    fn read_magic_slow(&mut self) -> Result<&[u8], Error> {
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

    fn get_ref(&self) -> &R {
        &self.reader
    }

    fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    fn into_inner(self) -> R {
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
    fn do_read_buf(&mut self, buf: BorrowedCursor<'_>) -> Result<usize, Error> {
        let n = buf.capacity().min(self.last - self.first);
        buf.append(&self.buf[self.first..(self.first + n)]);
        self.first += n;
        Ok(n)
    }
}

impl<R: BufRead> Read for MagicReader<R> {
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
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> Result<(), Error> {
        if self.first != self.last {
            self.do_read_buf(buf)?;
        }
        self.reader.read_buf(buf)
    }

    #[cfg(feature = "nightly")]
    fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> Result<(), Error> {
        if self.first != self.last {
            self.do_read_buf(buf)?;
        }
        self.reader.read_buf_exact(buf)
    }
}

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

enum InnerDecoder<R: BufRead> {
    Empty(Empty),
    Reader(R),
    Gz(GzDecoder<R>),
    Bz(BzDecoder<R>),
    Zlib(ZlibDecoder<R>),
    Xz(XzDecoder<R>),
    Zstd(ZstdDecoder<'static, R>),
}

impl<R: BufRead> InnerDecoder<MagicReader<R>> {
    fn new(mut reader: MagicReader<R>) -> Result<Self, Error> {
        let magic = reader.read_magic()?;
        let magic = if magic.len() >= MAX_MAGIC_BYTES {
            magic
        } else {
            reader.read_magic_slow()?
        };
        match magic {
            // https://tukaani.org/xz/xz-file-format-1.0.4.txt
            [0xfd, b'7', b'z', b'X', b'Z', 0, ..] => Ok(InnerDecoder::Xz(XzDecoder::new(reader))),
            // RFC8878
            [0x28, 0xb5, 0x2f, 0xfd, ..] => {
                Ok(InnerDecoder::Zstd(ZstdDecoder::with_buffer(reader)?))
            }
            // RFC1952
            [0x1f, 0x8b, 0x08, ..] => Ok(InnerDecoder::Gz(GzDecoder::new(reader))),
            // https://en.wikipedia.org/wiki/Bzip2
            [b'B', b'Z', b'h', ..] => Ok(InnerDecoder::Bz(BzDecoder::new(reader))),
            // https://www.rfc-editor.org/rfc/rfc1950
            [cmf, flg, ..]
                if zlib_cm(*cmf) == 8
                    && zlib_cinfo(*cmf) <= 7
                    && ((*cmf as u16) * 256 + (*flg as u16)) % 31 == 0 =>
            {
                Ok(InnerDecoder::Zlib(ZlibDecoder::new(reader)))
            }
            // TODO pbzx
            _ => Ok(InnerDecoder::Reader(reader)),
        }
    }
}

const fn zlib_cm(x: u8) -> u8 {
    x & 0b1111
}

const fn zlib_cinfo(x: u8) -> u8 {
    (x >> 4) & 0b1111
}

const MAX_MAGIC_BYTES: usize = 6;

macro_rules! dispatch_mut {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            InnerDecoder::Reader(ref mut r) => $method(r, $($args),*),
            InnerDecoder::Gz(ref mut r) => $method(r, $($args),*),
            InnerDecoder::Bz(ref mut r) => $method(r, $($args),*),
            InnerDecoder::Zlib(ref mut r) => $method(r, $($args),*),
            InnerDecoder::Xz(ref mut r) => $method(r, $($args),*),
            InnerDecoder::Zstd(ref mut r) => $method(r, $($args),*),
            InnerDecoder::Empty(ref mut r) => $method(r, $($args),*),
        }
    }
}

use dispatch_mut;

#[cfg(feature = "nightly")]
macro_rules! dispatch {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            InnerDecoder::Reader(ref r) => $method(r, $($args),*),
            InnerDecoder::Gz(ref r) => $method(r, $($args),*),
            InnerDecoder::Bz(ref r) => $method(r, $($args),*),
            InnerDecoder::Zlib(ref r) => $method(r, $($args),*),
            InnerDecoder::Xz(ref r) => $method(r, $($args),*),
            InnerDecoder::Zstd(ref r) => $method(r, $($args),*),
            InnerDecoder::Empty(ref r) => $method(r, $($args),*),
        }
    }
}

#[cfg(feature = "nightly")]
use dispatch;

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io::Write;

    use arbitrary::Unstructured;
    use arbtest::arbtest;

    use super::*;
    use crate::test::test_bufread_all;
    use crate::test::test_read_trait;
    use crate::test::Finish;

    #[test]
    fn write_gz_read_any() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        arbtest(|u| {
            let compression = Compression::new(u.int_in_range(0..=9)?);
            let writer = GzEncoder::new(Vec::new(), compression);
            write_some_read_any(writer, u)
        });
    }

    #[test]
    fn write_bz_read_any() {
        use bzip2::write::BzEncoder;
        use bzip2::Compression;
        arbtest(|u| {
            let compression = Compression::new(u.int_in_range(1..=9)?);
            let writer = BzEncoder::new(Vec::new(), compression);
            write_some_read_any(writer, u)
        });
    }

    #[test]
    fn write_zlib_read_any() {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        arbtest(|u| {
            let compression = Compression::new(u.int_in_range(1..=9)?);
            let writer = ZlibEncoder::new(Vec::new(), compression);
            write_some_read_any(writer, u)
        });
    }

    #[test]
    fn write_xz_read_any() {
        use xz::write::XzEncoder;
        arbtest(|u| {
            let compression = u.int_in_range(0..=9)?;
            let writer = XzEncoder::new(Vec::new(), compression);
            write_some_read_any(writer, u)
        });
    }

    #[test]
    fn write_zstd_read_any() {
        use zstd::stream::write::Encoder;
        arbtest(|u| {
            let compression = u.int_in_range(0..=22)?;
            let writer = Encoder::new(Vec::new(), compression).unwrap();
            write_some_read_any(writer, u)
        });
    }

    #[test]
    fn single_byte_reader() {
        arbtest(|u| {
            let expected: Vec<u8> = u.arbitrary()?;
            let capacity = u.int_in_range(1..=4096)?;
            let mut reader = SingleByteReader::new(&expected[..], capacity);
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

    #[test]
    fn test_any_decoder() {
        test_read_trait(new_gz_reader);
        test_read_trait(new_zlib_reader);
        test_read_trait(new_bz_reader);
        test_read_trait(new_xz_reader);
        test_read_trait(new_zstd_reader);
    }

    fn new_gz_reader<'a>(
        vec: VecDeque<u8>,
        u: &mut Unstructured<'a>,
    ) -> AnyDecoder<SingleByteReader<VecDeque<u8>>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let compression = Compression::new(u.int_in_range(0..=9).unwrap());
        let mut writer = GzEncoder::new(Vec::new(), compression);
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = SingleByteReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    fn new_zlib_reader<'a>(
        vec: VecDeque<u8>,
        u: &mut Unstructured<'a>,
    ) -> AnyDecoder<SingleByteReader<VecDeque<u8>>> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        let compression = Compression::new(u.int_in_range(0..=9).unwrap());
        let mut writer = ZlibEncoder::new(Vec::new(), compression);
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = SingleByteReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    fn new_bz_reader<'a>(
        vec: VecDeque<u8>,
        u: &mut Unstructured<'a>,
    ) -> AnyDecoder<SingleByteReader<VecDeque<u8>>> {
        use bzip2::write::BzEncoder;
        use bzip2::Compression;
        let compression = Compression::new(u.int_in_range(1..=9).unwrap());
        let mut writer = BzEncoder::new(Vec::new(), compression);
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = SingleByteReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    fn new_xz_reader<'a>(
        vec: VecDeque<u8>,
        u: &mut Unstructured<'a>,
    ) -> AnyDecoder<SingleByteReader<VecDeque<u8>>> {
        use xz::write::XzEncoder;
        let compression = u.int_in_range(0..=9).unwrap();
        let mut writer = XzEncoder::new(Vec::new(), compression);
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = SingleByteReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    fn new_zstd_reader<'a>(
        vec: VecDeque<u8>,
        u: &mut Unstructured<'a>,
    ) -> AnyDecoder<SingleByteReader<VecDeque<u8>>> {
        use zstd::stream::write::Encoder;
        let compression = u.int_in_range(0..=22).unwrap();
        let mut writer = Encoder::new(Vec::new(), compression).unwrap();
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = SingleByteReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    #[test]
    fn test_magic_reader() {
        test_read_trait(new_magic_reader);
        test_bufread_all(new_magic_reader);
        test_read_trait(new_magic_reader_v2);
        test_bufread_all(new_magic_reader_v2);
        test_read_trait(new_magic_reader_v3);
        test_bufread_all(new_magic_reader_v3);
    }

    fn new_magic_reader<'a>(
        vec: VecDeque<u8>,
        _u: &mut Unstructured<'a>,
    ) -> MagicReader<VecDeque<u8>> {
        let len = vec.len();
        let mut reader = MagicReader::new(vec);
        let magic = reader.read_magic().unwrap();
        let magic = if magic.len() >= MAX_MAGIC_BYTES {
            magic
        } else {
            reader.read_magic_slow().unwrap()
        };
        assert!(len <= magic.len());
        reader
    }

    fn new_magic_reader_v2<'a>(
        vec: VecDeque<u8>,
        _u: &mut Unstructured<'a>,
    ) -> MagicReader<VecDeque<u8>> {
        MagicReader::new(vec)
    }

    fn new_magic_reader_v3<'a>(
        vec: VecDeque<u8>,
        u: &mut Unstructured<'a>,
    ) -> MagicReader<SingleByteReader<VecDeque<u8>>> {
        let reader = SingleByteReader::new(vec, u.int_in_range(1..=100).unwrap());
        MagicReader::new(reader)
    }

    fn write_some_read_any<'a, W: Write + Finish<Vec<u8>>>(
        mut writer: W,
        u: &mut Unstructured<'a>,
    ) -> arbitrary::Result<()> {
        let expected: Vec<u8> = u.arbitrary()?;
        writer.write_all(&expected).unwrap();
        let compressed = writer.finish().unwrap();
        let capacity = u.int_in_range(1..=4096)?;
        let reader = SingleByteReader::new(&compressed[..], capacity);
        //eprintln!("compressed {:#x?}", compressed);
        let mut reader = AnyDecoder::new(reader);
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        Ok(())
    }

    // Reads at most `len` bytes.
    struct SingleByteReader<R: Read> {
        reader: R,
        buf: Vec<u8>,
        len: usize,
    }

    impl<R: Read> SingleByteReader<R> {
        fn new(reader: R, len: usize) -> Self {
            Self {
                reader,
                buf: Default::default(),
                len,
            }
        }
    }

    impl<R: Read> Read for SingleByteReader<R> {
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

    impl<R: Read> BufRead for SingleByteReader<R> {
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
}
