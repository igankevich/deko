#[cfg(feature = "nightly")]
use std::io::BorrowedCursor;
use std::io::BufRead;
use std::io::Empty;
use std::io::Error;
use std::io::IoSliceMut;
use std::io::Read;

use bzip2::bufread::BzDecoder;
use flate2::bufread::GzDecoder;
use flate2::bufread::ZlibDecoder;
use xz::bufread::XzDecoder;
use zstd::stream::read::Decoder as ZstdDecoder;

pub struct AnyDecoder<R: BufRead> {
    reader: Option<R>,
    inner: InnerDecoder<R>,
}

impl<R: BufRead> AnyDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: Some(reader),
            inner: InnerDecoder::Empty(std::io::empty()),
        }
    }

    pub fn get_ref(&self) -> &R {
        if let Some(r) = self.reader.as_ref() {
            return r;
        }
        match self.inner {
            InnerDecoder::Reader(ref r) => r,
            InnerDecoder::Gz(ref r) => r.get_ref(),
            InnerDecoder::Bz(ref r) => r.get_ref(),
            InnerDecoder::Zlib(ref r) => r.get_ref(),
            InnerDecoder::Xz(ref r) => r.get_ref(),
            InnerDecoder::Zstd(ref r) => r.get_ref(),
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }

    pub fn get_mut(&mut self) -> &R {
        if let Some(r) = self.reader.as_mut() {
            return r;
        }
        match self.inner {
            InnerDecoder::Reader(ref mut r) => r,
            InnerDecoder::Gz(ref mut r) => r.get_mut(),
            InnerDecoder::Bz(ref mut r) => r.get_mut(),
            InnerDecoder::Zlib(ref mut r) => r.get_mut(),
            InnerDecoder::Xz(ref mut r) => r.get_mut(),
            InnerDecoder::Zstd(ref mut r) => r.get_mut(),
            InnerDecoder::Empty(..) => unreachable!(),
        }
    }

    pub fn into_inner(mut self) -> R {
        if let Some(r) = self.reader.take() {
            return r;
        }
        match self.inner {
            InnerDecoder::Reader(r) => r,
            InnerDecoder::Gz(r) => r.into_inner(),
            InnerDecoder::Bz(r) => r.into_inner(),
            InnerDecoder::Zlib(r) => r.into_inner(),
            InnerDecoder::Xz(r) => r.into_inner(),
            InnerDecoder::Zstd(r) => r.finish(),
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

enum InnerDecoder<R: BufRead> {
    Empty(Empty),
    Reader(R),
    Gz(GzDecoder<R>),
    Bz(BzDecoder<R>),
    Zlib(ZlibDecoder<R>),
    Xz(XzDecoder<R>),
    Zstd(ZstdDecoder<'static, R>),
}

impl<R: BufRead> InnerDecoder<R> {
    fn new(mut reader: R) -> Result<InnerDecoder<R>, Error> {
        let data = reader.fill_buf()?;
        match data {
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
            [0x78, 0xda, ..] => Ok(InnerDecoder::Zlib(ZlibDecoder::new(reader))),
            // TODO pbzx
            _ => Ok(InnerDecoder::Reader(reader)),
        }
    }
}

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
