use std::fmt::Arguments;
use std::io::Error;
use std::io::IoSlice;
use std::io::Write;

#[cfg(feature = "bzip2")]
use bzip2::write::BzEncoder;
#[cfg(feature = "flate2")]
use flate2::write::GzEncoder;
#[cfg(feature = "flate2")]
use flate2::write::ZlibEncoder;
#[cfg(feature = "xz")]
use xz::write::XzEncoder;
#[cfg(feature = "zstd")]
use zstd::stream::write::Encoder as ZstdEncoder;

use crate::Format;

/// An encoder that dynamically selects compression format via [Format] and [Compression].
pub enum AnyEncoder<W: Write> {
    /// Verbatim encoder.
    Verbatim(W),
    /// Gzip encoder.
    #[cfg(feature = "flate2")]
    Gz(GzEncoder<W>),
    /// Bzip2 encoder.
    #[cfg(feature = "bzip2")]
    Bz(BzEncoder<W>),
    /// Zlib encoder.
    #[cfg(feature = "flate2")]
    Zlib(ZlibEncoder<W>),
    /// XZ encoder.
    #[cfg(feature = "xz")]
    Xz(XzEncoder<W>),
    /// Zstd encoder.
    #[cfg(feature = "zstd")]
    Zstd(ZstdEncoder<'static, W>),
}

impl<W: Write> AnyEncoder<W> {
    /// Create new encoder for the supplied `format` and `compression` ratio.
    pub fn new(writer: W, format: Format, compression: Compression) -> Result<Self, Error> {
        match format {
            Format::Verbatim => Ok(Self::Verbatim(writer)),
            #[cfg(feature = "flate2")]
            Format::Gz => Ok(Self::Gz(GzEncoder::new(writer, compression.to_flate2()))),
            #[cfg(feature = "bzip2")]
            Format::Bz => Ok(Self::Bz(BzEncoder::new(writer, compression.to_bzip2()))),
            #[cfg(feature = "flate2")]
            Format::Zlib => Ok(Self::Zlib(ZlibEncoder::new(
                writer,
                compression.to_flate2(),
            ))),
            #[cfg(feature = "xz")]
            Format::Xz => Ok(Self::Xz(XzEncoder::new(writer, compression.to_xz()))),
            #[cfg(feature = "zstd")]
            Format::Zstd => Ok(Self::Zstd(ZstdEncoder::new(writer, compression.to_zstd())?)),
        }
    }

    /// Get encoding format.
    pub fn format(&self) -> Format {
        match self {
            Self::Verbatim(..) => Format::Verbatim,
            #[cfg(feature = "flate2")]
            Self::Gz(..) => Format::Gz,
            #[cfg(feature = "bzip2")]
            Self::Bz(..) => Format::Bz,
            #[cfg(feature = "flate2")]
            Self::Zlib(..) => Format::Zlib,
            #[cfg(feature = "xz")]
            Self::Xz(..) => Format::Xz,
            #[cfg(feature = "zstd")]
            Self::Zstd(..) => Format::Zstd,
        }
    }

    /// Get immutable reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        match self {
            Self::Verbatim(ref w) => w,
            #[cfg(feature = "flate2")]
            Self::Gz(ref w) => w.get_ref(),
            #[cfg(feature = "bzip2")]
            Self::Bz(ref w) => w.get_ref(),
            #[cfg(feature = "flate2")]
            Self::Zlib(ref w) => w.get_ref(),
            #[cfg(feature = "xz")]
            Self::Xz(ref w) => w.get_ref(),
            #[cfg(feature = "zstd")]
            Self::Zstd(ref w) => w.get_ref(),
        }
    }

    /// Get mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        match self {
            Self::Verbatim(ref mut w) => w,
            #[cfg(feature = "flate2")]
            Self::Gz(ref mut w) => w.get_mut(),
            #[cfg(feature = "bzip2")]
            Self::Bz(ref mut w) => w.get_mut(),
            #[cfg(feature = "flate2")]
            Self::Zlib(ref mut w) => w.get_mut(),
            #[cfg(feature = "xz")]
            Self::Xz(ref mut w) => w.get_mut(),
            #[cfg(feature = "zstd")]
            Self::Zstd(ref mut w) => w.get_mut(),
        }
    }

    /// Finish encoding and return the underlying writer.
    ///
    /// This method is **not** automatically called on drop.
    pub fn finish(self) -> Result<W, Error> {
        match self {
            Self::Verbatim(w) => Ok(w),
            #[cfg(feature = "flate2")]
            Self::Gz(w) => w.finish(),
            #[cfg(feature = "bzip2")]
            Self::Bz(w) => w.finish(),
            #[cfg(feature = "flate2")]
            Self::Zlib(w) => w.finish(),
            #[cfg(feature = "xz")]
            Self::Xz(w) => w.finish(),
            #[cfg(feature = "zstd")]
            Self::Zstd(w) => w.finish(),
        }
    }
}

impl<W: Write> Write for AnyEncoder<W> {
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

/// Compression level.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum Compression {
    /// Usually the lowest compression level.
    Fast,
    /// Usually some medium compression level.
    #[default]
    Default,
    /// Usually the highest compression level.
    Best,
    /// Concrete compression level.
    ///
    /// Its meaning depends on the encoder being used.
    Level(u32),
}

impl Compression {
    /// Convert to specific compression level used by the underlying encoder.
    pub fn to_level(self, encoder: Format) -> CompressionLevel {
        match encoder {
            Format::Verbatim => CompressionLevel::None,
            #[cfg(feature = "flate2")]
            Format::Gz => CompressionLevel::Gz(self.to_flate2()),
            #[cfg(feature = "bzip2")]
            Format::Bz => CompressionLevel::Bz(self.to_bzip2()),
            #[cfg(feature = "flate2")]
            Format::Zlib => CompressionLevel::Zlib(self.to_flate2()),
            #[cfg(feature = "xz")]
            Format::Xz => CompressionLevel::Xz(self.to_xz()),
            #[cfg(feature = "zstd")]
            Format::Zstd => CompressionLevel::Zstd(self.to_zstd()),
        }
    }

    #[cfg(feature = "flate2")]
    fn to_flate2(self) -> flate2::Compression {
        match self {
            Self::Fast => flate2::Compression::fast(),
            Self::Default => flate2::Compression::default(),
            Self::Best => flate2::Compression::best(),
            Self::Level(i) => flate2::Compression::new(i),
        }
    }

    #[cfg(feature = "bzip2")]
    fn to_bzip2(self) -> bzip2::Compression {
        match self {
            Self::Fast => bzip2::Compression::fast(),
            Self::Default => bzip2::Compression::default(),
            Self::Best => bzip2::Compression::best(),
            Self::Level(i) => bzip2::Compression::new(i),
        }
    }

    #[cfg(feature = "xz")]
    fn to_xz(self) -> u32 {
        match self {
            Self::Fast => 1,
            Self::Default => 5,
            Self::Best => 9,
            Self::Level(i) => i,
        }
    }

    #[cfg(feature = "zstd")]
    fn to_zstd(self) -> i32 {
        match self {
            Self::Fast => 1,
            Self::Default => 0,
            Self::Best => 22,
            Self::Level(i) => i as i32,
        }
    }
}

/// Specific compression level for each output format.
#[derive(Clone, Copy, Debug)]
pub enum CompressionLevel {
    /// No compression
    None,
    /// Gzip compression level.
    #[cfg(feature = "flate2")]
    Gz(flate2::Compression),
    /// Bzip2 compression level.
    #[cfg(feature = "bzip2")]
    Bz(bzip2::Compression),
    /// Zlib compression level.
    #[cfg(feature = "flate2")]
    Zlib(flate2::Compression),
    /// XZ compression level (1–9).
    #[cfg(feature = "xz")]
    Xz(u32),
    /// Zstd compression level (1–22, 0 means default compression).
    #[cfg(feature = "zstd")]
    Zstd(i32),
}

macro_rules! dispatch_mut {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            Self::Verbatim(ref mut w) => $method(w, $($args),*),
            #[cfg(feature = "flate2")]
            Self::Gz(ref mut w) => $method(w, $($args),*),
            #[cfg(feature = "bzip2")]
            Self::Bz(ref mut w) => $method(w, $($args),*),
            #[cfg(feature = "flate2")]
            Self::Zlib(ref mut w) => $method(w, $($args),*),
            #[cfg(feature = "xz")]
            Self::Xz(ref mut w) => $method(w, $($args),*),
            #[cfg(feature = "zstd")]
            Self::Zstd(ref mut w) => $method(w, $($args),*),
        }
    }
}

use dispatch_mut;

#[cfg(feature = "nightly")]
macro_rules! dispatch {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            Self::Verbatim(ref w) => $method(w, $($args),*),
            #[cfg(feature = "flate2")]
            Self::Gz(ref w) => $method(w, $($args),*),
            #[cfg(feature = "bzip2")]
            Self::Bz(ref w) => $method(w, $($args),*),
            #[cfg(feature = "flate2")]
            Self::Zlib(ref w) => $method(w, $($args),*),
            #[cfg(feature = "xz")]
            Self::Xz(ref w) => $method(w, $($args),*),
            #[cfg(feature = "zstd")]
            Self::Zstd(ref w) => $method(w, $($args),*),
        }
    }
}

#[cfg(feature = "nightly")]
use dispatch;

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io::Read;

    use arbitrary::Unstructured;

    use super::*;
    use crate::bufread::AnyDecoder;
    use crate::test::test_write_trait;

    #[test]
    fn test_any_encoder() {
        test_write_trait(new_any_encoder, new_any_decoder);
    }

    type AnyEncoderVecDeque = AnyEncoder<VecDeque<u8>>;

    fn new_any_encoder(
        writer: VecDeque<u8>,
        u: &mut Unstructured<'_>,
    ) -> arbitrary::Result<AnyEncoderVecDeque> {
        let format: Format = u.arbitrary()?;
        let compression: Compression = arbitrary_compression(format, u)?;
        let encoder = AnyEncoder::new(writer, format, compression).unwrap();
        assert_eq!(format, encoder.format());
        Ok(encoder)
    }

    fn new_any_decoder(
        writer: AnyEncoder<VecDeque<u8>>,
        u: &mut Unstructured<'_>,
    ) -> arbitrary::Result<Box<dyn Read>> {
        let format = writer.format();
        let inner = writer.finish().unwrap();
        let any: bool = u.arbitrary()?;
        let decoder: Box<dyn Read> = if any {
            Box::new(AnyDecoder::new(inner))
        } else {
            match format {
                Format::Verbatim => Box::new(inner),
                #[cfg(feature = "flate2")]
                Format::Gz => Box::new(flate2::read::GzDecoder::new(inner)),
                #[cfg(feature = "flate2")]
                Format::Zlib => Box::new(flate2::read::ZlibDecoder::new(inner)),
                #[cfg(feature = "bzip2")]
                Format::Bz => Box::new(bzip2::read::BzDecoder::new(inner)),
                #[cfg(feature = "xz")]
                Format::Xz => Box::new(xz::read::XzDecoder::new(inner)),
                #[cfg(feature = "zstd")]
                Format::Zstd => Box::new(zstd::stream::read::Decoder::new(inner).unwrap()),
            }
        };
        Ok(decoder)
    }

    fn arbitrary_compression(
        format: Format,
        u: &mut Unstructured<'_>,
    ) -> arbitrary::Result<Compression> {
        let compression = u.arbitrary()?;
        Ok(match format {
            Format::Verbatim => compression,
            #[cfg(feature = "flate2")]
            Format::Gz => compression.clamp(0, 9),
            #[cfg(feature = "flate2")]
            Format::Zlib => compression.clamp(0, 9),
            #[cfg(feature = "bzip2")]
            Format::Bz => compression.clamp(1, 9),
            #[cfg(feature = "xz")]
            Format::Xz => compression.clamp(0, 9),
            #[cfg(feature = "zstd")]
            Format::Zstd => compression.clamp(0, 22),
        })
    }

    impl Compression {
        fn clamp(self, min: u32, max: u32) -> Self {
            match self {
                Self::Level(i) => Self::Level(i.min(max).max(min)),
                other => other,
            }
        }
    }
}
