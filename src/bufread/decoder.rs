#[cfg(feature = "nightly")]
use std::io::BorrowedCursor;
use std::io::BufRead;
use std::io::Empty;
use std::io::Error;
use std::io::ErrorKind;
use std::io::IoSliceMut;
use std::io::Read;

#[cfg(feature = "bzip2")]
use bzip2::bufread::BzDecoder;
#[cfg(feature = "flate2")]
use flate2::bufread::GzDecoder;
#[cfg(feature = "flate2")]
use flate2::bufread::ZlibDecoder;
#[cfg(feature = "xz")]
use xz::bufread::XzDecoder;
#[cfg(feature = "zstd")]
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::Format;

/// A decoder that decompresses the supplied input stream using any of the supported formats.
///
/// The format is detected using the _magic bytes_ at the start of the stream.
/// By default, if the format is not supported, the data is read verbatim.
/// Use [fail_on_unknown_format](AnyDecoder::fail_on_unknown_format) to change this behaviour.
pub struct AnyDecoder<R: BufRead> {
    reader: Option<MagicReader<R>>,
    inner: InnerDecoder<MagicReader<R>>,
    fail_on_unknown_format: bool,
}

impl<R: BufRead> AnyDecoder<R> {
    /// Create new decoder from the supplied `reader`.
    pub fn new(reader: R) -> Self {
        Self {
            reader: Some(MagicReader::new(reader)),
            inner: InnerDecoder::Empty(std::io::empty()),
            fail_on_unknown_format: false,
        }
    }

    /// Get the input stream format.
    ///
    /// The format is detected automatically when the data is read from the decoder.
    /// If nothing was read before calling this method, a small amount of data is read from the
    /// stream to detect the format.
    /// If the format has already been detected, this method merely returns it.
    pub fn kind(&mut self) -> Result<Format, Error> {
        self.detect()?;
        Ok(self.get_kind())
    }

    /// Throw an error when the decoder fails to detect compression format.
    ///
    /// By default no error is thrown, and the data is read verbatim.
    pub fn fail_on_unknown_format(&mut self, value: bool) {
        self.fail_on_unknown_format = value;
    }

    /// Get immutable reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        if let Some(r) = self.reader.as_ref() {
            return r.get_ref();
        }
        match self.inner {
            InnerDecoder::Reader(ref r) => r.get_ref(),
            #[cfg(feature = "flate2")]
            InnerDecoder::Gz(ref r) => r.get_ref().get_ref(),
            #[cfg(feature = "bzip2")]
            InnerDecoder::Bz(ref r) => r.get_ref().get_ref(),
            #[cfg(feature = "flate2")]
            InnerDecoder::Zlib(ref r) => r.get_ref().get_ref(),
            #[cfg(feature = "xz")]
            InnerDecoder::Xz(ref r) => r.get_ref().get_ref(),
            #[cfg(feature = "zstd")]
            InnerDecoder::Zstd(ref r) => r.get_ref().get_ref(),
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }

    /// Get mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        if let Some(r) = self.reader.as_mut() {
            return r.get_mut();
        }
        match self.inner {
            InnerDecoder::Reader(ref mut r) => r.get_mut(),
            #[cfg(feature = "flate2")]
            InnerDecoder::Gz(ref mut r) => r.get_mut().get_mut(),
            #[cfg(feature = "bzip2")]
            InnerDecoder::Bz(ref mut r) => r.get_mut().get_mut(),
            #[cfg(feature = "flate2")]
            InnerDecoder::Zlib(ref mut r) => r.get_mut().get_mut(),
            #[cfg(feature = "xz")]
            InnerDecoder::Xz(ref mut r) => r.get_mut().get_mut(),
            #[cfg(feature = "zstd")]
            InnerDecoder::Zstd(ref mut r) => r.get_mut().get_mut(),
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }

    /// Return the underlying reader.
    pub fn into_inner(mut self) -> R {
        if let Some(r) = self.reader.take() {
            return r.reader;
        }
        match self.inner {
            InnerDecoder::Reader(r) => r.into_inner(),
            #[cfg(feature = "flate2")]
            InnerDecoder::Gz(r) => r.into_inner().into_inner(),
            #[cfg(feature = "bzip2")]
            InnerDecoder::Bz(r) => r.into_inner().into_inner(),
            #[cfg(feature = "flate2")]
            InnerDecoder::Zlib(r) => r.into_inner().into_inner(),
            #[cfg(feature = "xz")]
            InnerDecoder::Xz(r) => r.into_inner().into_inner(),
            #[cfg(feature = "zstd")]
            InnerDecoder::Zstd(r) => r.finish().into_inner(),
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }

    #[inline]
    fn detect(&mut self) -> Result<(), Error> {
        if let Some(r) = self.reader.take() {
            self.inner = InnerDecoder::new(r, self.fail_on_unknown_format)?;
        }
        Ok(())
    }

    #[inline]
    fn get_kind(&self) -> Format {
        match self.inner {
            InnerDecoder::Reader(..) => Format::Verbatim,
            #[cfg(feature = "flate2")]
            InnerDecoder::Gz(..) => Format::Gz,
            #[cfg(feature = "bzip2")]
            InnerDecoder::Bz(..) => Format::Bz,
            #[cfg(feature = "flate2")]
            InnerDecoder::Zlib(..) => Format::Zlib,
            #[cfg(feature = "xz")]
            InnerDecoder::Xz(..) => Format::Xz,
            #[cfg(feature = "zstd")]
            InnerDecoder::Zstd(..) => Format::Zstd,
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }
}

impl<R: BufRead> Read for AnyDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.detect()?;
        dispatch_mut!(self.inner, Read::read, buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize, Error> {
        self.detect()?;
        dispatch_mut!(self.inner, Read::read_vectored, bufs)
    }

    #[cfg(feature = "nightly")]
    fn is_read_vectored(&self) -> bool {
        dispatch!(self.inner, Read::is_read_vectored)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        self.detect()?;
        dispatch_mut!(self.inner, Read::read_to_end, buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Error> {
        self.detect()?;
        dispatch_mut!(self.inner, Read::read_to_string, buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.detect()?;
        dispatch_mut!(self.inner, Read::read_exact, buf)
    }

    #[cfg(feature = "nightly")]
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> Result<(), Error> {
        self.detect()?;
        dispatch_mut!(self.inner, Read::read_buf, buf)
    }

    #[cfg(feature = "nightly")]
    fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> Result<(), Error> {
        self.detect()?;
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
    fn do_read_buf(&mut self, buf: &mut BorrowedCursor<'_>) -> Result<usize, Error> {
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
    #[cfg(feature = "flate2")]
    Gz(GzDecoder<R>),
    #[cfg(feature = "bzip2")]
    Bz(BzDecoder<R>),
    #[cfg(feature = "flate2")]
    Zlib(ZlibDecoder<R>),
    #[cfg(feature = "xz")]
    Xz(XzDecoder<R>),
    #[cfg(feature = "zstd")]
    Zstd(ZstdDecoder<'static, R>),
}

impl<R: BufRead> InnerDecoder<MagicReader<R>> {
    fn new(mut reader: MagicReader<R>, fail_on_unknown_format: bool) -> Result<Self, Error> {
        let magic = reader.read_magic()?;
        let magic = if magic.len() >= MAX_MAGIC_BYTES {
            magic
        } else {
            reader.read_magic_slow()?
        };
        match magic {
            // https://tukaani.org/xz/xz-file-format-1.0.4.txt
            #[cfg(feature = "xz")]
            [0xfd, b'7', b'z', b'X', b'Z', 0, ..] => Ok(InnerDecoder::Xz(XzDecoder::new(reader))),
            // RFC8878
            #[cfg(feature = "zstd")]
            [0x28, 0xb5, 0x2f, 0xfd, ..] => {
                Ok(InnerDecoder::Zstd(ZstdDecoder::with_buffer(reader)?))
            }
            // RFC1952
            #[cfg(feature = "flate2")]
            [0x1f, 0x8b, 0x08, ..] => Ok(InnerDecoder::Gz(GzDecoder::new(reader))),
            // https://en.wikipedia.org/wiki/Bzip2
            #[cfg(feature = "bzip2")]
            [b'B', b'Z', b'h', ..] => Ok(InnerDecoder::Bz(BzDecoder::new(reader))),
            // https://www.rfc-editor.org/rfc/rfc1950
            #[cfg(feature = "flate2")]
            [cmf, flg, ..]
                if zlib_cm(*cmf) == 8
                    && zlib_cinfo(*cmf) <= 7
                    && ((*cmf as u16) * 256 + (*flg as u16)) % 31 == 0 =>
            {
                Ok(InnerDecoder::Zlib(ZlibDecoder::new(reader)))
            }
            // TODO pbzx
            _ if fail_on_unknown_format => Err(Error::new(
                ErrorKind::InvalidData,
                "unknown compression format",
            )),
            _ => Ok(InnerDecoder::Reader(reader)),
        }
    }
}

#[cfg(feature = "flate2")]
const fn zlib_cm(x: u8) -> u8 {
    x & 0b1111
}

#[cfg(feature = "flate2")]
const fn zlib_cinfo(x: u8) -> u8 {
    (x >> 4) & 0b1111
}

const MAX_MAGIC_BYTES: usize = 6;

macro_rules! dispatch_mut {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            InnerDecoder::Reader(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "flate2")]
            InnerDecoder::Gz(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "bzip2")]
            InnerDecoder::Bz(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "flate2")]
            InnerDecoder::Zlib(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "xz")]
            InnerDecoder::Xz(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "zstd")]
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
            #[cfg(feature = "flate2")]
            InnerDecoder::Gz(ref r) => $method(r, $($args),*),
            #[cfg(feature = "bzip2")]
            InnerDecoder::Bz(ref r) => $method(r, $($args),*),
            #[cfg(feature = "flate2")]
            InnerDecoder::Zlib(ref r) => $method(r, $($args),*),
            #[cfg(feature = "xz")]
            InnerDecoder::Xz(ref r) => $method(r, $($args),*),
            #[cfg(feature = "zstd")]
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
    use crate::test::NBytesReader;

    #[cfg(feature = "flate2")]
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

    #[cfg(feature = "bzip2")]
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

    #[cfg(feature = "flate2")]
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

    #[cfg(feature = "xz")]
    #[test]
    fn write_xz_read_any() {
        use xz::write::XzEncoder;
        arbtest(|u| {
            let compression = u.int_in_range(0..=9)?;
            let writer = XzEncoder::new(Vec::new(), compression);
            write_some_read_any(writer, u)
        });
    }

    #[cfg(feature = "zstd")]
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
    fn test_any_decoder() {
        #[cfg(feature = "flate2")]
        test_read_trait(new_gz_reader);
        #[cfg(feature = "flate2")]
        test_read_trait(new_zlib_reader);
        #[cfg(feature = "bzip2")]
        test_read_trait(new_bz_reader);
        #[cfg(feature = "xz")]
        test_read_trait(new_xz_reader);
        #[cfg(feature = "zstd")]
        test_read_trait(new_zstd_reader);
    }

    #[cfg(feature = "flate2")]
    fn new_gz_reader(
        vec: VecDeque<u8>,
        u: &mut Unstructured,
    ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let compression = Compression::new(u.int_in_range(0..=9).unwrap());
        let mut writer = GzEncoder::new(Vec::new(), compression);
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    #[cfg(feature = "flate2")]
    fn new_zlib_reader(
        vec: VecDeque<u8>,
        u: &mut Unstructured,
    ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        let compression = Compression::new(u.int_in_range(0..=9).unwrap());
        let mut writer = ZlibEncoder::new(Vec::new(), compression);
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    #[cfg(feature = "bzip2")]
    fn new_bz_reader(
        vec: VecDeque<u8>,
        u: &mut Unstructured,
    ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
        use bzip2::write::BzEncoder;
        use bzip2::Compression;
        let compression = Compression::new(u.int_in_range(1..=9).unwrap());
        let mut writer = BzEncoder::new(Vec::new(), compression);
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    #[cfg(feature = "xz")]
    fn new_xz_reader(
        vec: VecDeque<u8>,
        u: &mut Unstructured,
    ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
        use xz::write::XzEncoder;
        let compression = u.int_in_range(0..=9).unwrap();
        let mut writer = XzEncoder::new(Vec::new(), compression);
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
        AnyDecoder::new(reader)
    }

    #[cfg(feature = "zstd")]
    fn new_zstd_reader(
        vec: VecDeque<u8>,
        u: &mut Unstructured,
    ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
        use zstd::stream::write::Encoder;
        let compression = u.int_in_range(0..=22).unwrap();
        let mut writer = Encoder::new(Vec::new(), compression).unwrap();
        let bytes = vec.into_iter().collect::<Vec<_>>();
        writer.write_all(&bytes).unwrap();
        let compressed: VecDeque<u8> = writer.finish().unwrap().into();
        let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
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

    fn new_magic_reader(vec: VecDeque<u8>, _u: &mut Unstructured) -> MagicReader<VecDeque<u8>> {
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

    fn new_magic_reader_v2(vec: VecDeque<u8>, _u: &mut Unstructured) -> MagicReader<VecDeque<u8>> {
        MagicReader::new(vec)
    }

    fn new_magic_reader_v3(
        vec: VecDeque<u8>,
        u: &mut Unstructured,
    ) -> MagicReader<NBytesReader<VecDeque<u8>>> {
        let reader = NBytesReader::new(vec, u.int_in_range(1..=100).unwrap());
        MagicReader::new(reader)
    }

    fn write_some_read_any<W: Write + Finish<Vec<u8>>>(
        mut writer: W,
        u: &mut Unstructured,
    ) -> arbitrary::Result<()> {
        let expected: Vec<u8> = u.arbitrary()?;
        writer.write_all(&expected).unwrap();
        let compressed = writer.finish().unwrap();
        let capacity = u.int_in_range(1..=4096)?;
        let reader = NBytesReader::new(&compressed[..], capacity);
        //eprintln!("compressed {:#x?}", compressed);
        let mut reader = AnyDecoder::new(reader);
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        Ok(())
    }
}
