use std::fmt::Arguments;
use std::io::Error;
use std::io::IoSlice;
use std::io::Write;

use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use flate2::write::ZlibEncoder;
use xz::write::XzEncoder;
use zstd::stream::write::Encoder as ZstdEncoder;

pub enum AnyEncoder<'a, W: Write> {
    Write(W),
    Gz(GzEncoder<W>),
    Bz(BzEncoder<W>),
    Zlib(ZlibEncoder<W>),
    Xz(XzEncoder<W>),
    Zstd(ZstdEncoder<'a, W>),
}

impl<'a, W: Write> AnyEncoder<'a, W> {
    pub fn new(writer: W, encoder: Encoder, compression: Compression) -> Result<Self, Error> {
        match encoder {
            Encoder::Write => Ok(Self::Write(writer)),
            Encoder::Gz => Ok(Self::Gz(GzEncoder::new(writer, compression.to_flate2()))),
            Encoder::Bz => Ok(Self::Bz(BzEncoder::new(writer, compression.to_bzip2()))),
            Encoder::Zlib => Ok(Self::Zlib(ZlibEncoder::new(
                writer,
                compression.to_flate2(),
            ))),
            Encoder::Xz => Ok(Self::Xz(XzEncoder::new(writer, compression.to_xz()))),
            Encoder::Zstd => Ok(Self::Zstd(ZstdEncoder::new(writer, compression.to_zstd())?)),
        }
    }

    pub fn encoder(&self) -> Encoder {
        match self {
            Self::Write(..) => Encoder::Write,
            Self::Gz(..) => Encoder::Gz,
            Self::Bz(..) => Encoder::Bz,
            Self::Zlib(..) => Encoder::Zlib,
            Self::Xz(..) => Encoder::Xz,
            Self::Zstd(..) => Encoder::Zstd,
        }
    }

    pub fn get_ref(&self) -> &W {
        match self {
            Self::Write(ref w) => w,
            Self::Gz(ref w) => w.get_ref(),
            Self::Bz(ref w) => w.get_ref(),
            Self::Zlib(ref w) => w.get_ref(),
            Self::Xz(ref w) => w.get_ref(),
            Self::Zstd(ref w) => w.get_ref(),
        }
    }

    pub fn get_mut(&mut self) -> &mut W {
        match self {
            Self::Write(ref mut w) => w,
            Self::Gz(ref mut w) => w.get_mut(),
            Self::Bz(ref mut w) => w.get_mut(),
            Self::Zlib(ref mut w) => w.get_mut(),
            Self::Xz(ref mut w) => w.get_mut(),
            Self::Zstd(ref mut w) => w.get_mut(),
        }
    }

    pub fn finish(self) -> Result<W, Error> {
        match self {
            Self::Write(w) => Ok(w),
            Self::Gz(w) => w.finish(),
            Self::Bz(w) => w.finish(),
            Self::Zlib(w) => w.finish(),
            Self::Xz(w) => w.finish(),
            Self::Zstd(w) => w.finish(),
        }
    }
}

impl<'a, W: Write> Write for AnyEncoder<'a, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        dispatch_mut!(self, Write::write, buf)
    }

    fn flush(&mut self) -> Result<(), Error> {
        dispatch_mut!(self, Write::flush)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize, Error> {
        dispatch_mut!(self, Write::write_vectored, bufs)
    }

    #[cfg(feature = "nightly")]
    fn is_write_vectored(&self) -> bool {
        dispatch!(self, Write::is_write_vectored)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        dispatch_mut!(self, Write::write_all, buf)
    }

    #[cfg(feature = "nightly")]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> Result<(), Error> {
        dispatch_mut!(self, Write::write_all_vectored, bufs)
    }

    fn write_fmt(&mut self, fmt: Arguments<'_>) -> Result<(), Error> {
        dispatch_mut!(self, Write::write_fmt, fmt)
    }

    fn by_ref(&mut self) -> &mut Self {
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Encoder {
    Write,
    Gz,
    Bz,
    Zlib,
    Xz,
    Zstd,
}

/// Compression level.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum Compression {
    /// Usually the lowest compression level.
    Fast,
    /// Usually the medium compression level.
    #[default]
    Default,
    /// Usually the highest compression level.
    Best,
    /// Use the numeric value as the compression level.
    ///
    /// Its meaning depends on the encoder being used.
    Level(u32),
}

impl Compression {
    /// Convert to specific compression level used by the underlying encoder.
    pub fn to_level(self, encoder: Encoder) -> CompressionLevel {
        match encoder {
            Encoder::Write => CompressionLevel::Write,
            Encoder::Gz => CompressionLevel::Gz(self.to_flate2()),
            Encoder::Bz => CompressionLevel::Bz(self.to_bzip2()),
            Encoder::Zlib => CompressionLevel::Zlib(self.to_flate2()),
            Encoder::Xz => CompressionLevel::Xz(self.to_xz()),
            Encoder::Zstd => CompressionLevel::Zstd(self.to_zstd()),
        }
    }

    fn to_flate2(self) -> flate2::Compression {
        match self {
            Self::Fast => flate2::Compression::fast(),
            Self::Default => flate2::Compression::default(),
            Self::Best => flate2::Compression::best(),
            Self::Level(i) => flate2::Compression::new(i),
        }
    }

    fn to_bzip2(self) -> bzip2::Compression {
        match self {
            Self::Fast => bzip2::Compression::fast(),
            Self::Default => bzip2::Compression::default(),
            Self::Best => bzip2::Compression::best(),
            Self::Level(i) => bzip2::Compression::new(i),
        }
    }

    fn to_xz(self) -> u32 {
        match self {
            Self::Fast => 1,
            Self::Default => 5,
            Self::Best => 9,
            Self::Level(i) => i,
        }
    }

    fn to_zstd(self) -> i32 {
        match self {
            Self::Fast => 1,
            Self::Default => 0,
            Self::Best => 22,
            Self::Level(i) => i as i32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CompressionLevel {
    Write,
    Gz(flate2::Compression),
    Bz(bzip2::Compression),
    Zlib(flate2::Compression),
    Xz(u32),
    Zstd(i32),
}

macro_rules! dispatch_mut {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            Self::Write(ref mut w) => $method(w, $($args),*),
            Self::Gz(ref mut w) => $method(w, $($args),*),
            Self::Bz(ref mut w) => $method(w, $($args),*),
            Self::Zlib(ref mut w) => $method(w, $($args),*),
            Self::Xz(ref mut w) => $method(w, $($args),*),
            Self::Zstd(ref mut w) => $method(w, $($args),*),
        }
    }
}

use dispatch_mut;

#[cfg(feature = "nightly")]
macro_rules! dispatch {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            Self::Write(ref w) => $method(w, $($args),*),
            Self::Gz(ref w) => $method(w, $($args),*),
            Self::Bz(ref w) => $method(w, $($args),*),
            Self::Zlib(ref w) => $method(w, $($args),*),
            Self::Xz(ref w) => $method(w, $($args),*),
            Self::Zstd(ref w) => $method(w, $($args),*),
        }
    }
}

#[cfg(feature = "nightly")]
use dispatch;
