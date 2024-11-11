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
    pub fn new(writer: W, encoder: EncoderKind, compression: Compression) -> Result<Self, Error> {
        match encoder {
            EncoderKind::Write => Ok(Self::Write(writer)),
            EncoderKind::Gz => Ok(Self::Gz(GzEncoder::new(writer, compression.to_flate2()))),
            EncoderKind::Bz => Ok(Self::Bz(BzEncoder::new(writer, compression.to_bzip2()))),
            EncoderKind::Zlib => Ok(Self::Zlib(ZlibEncoder::new(
                writer,
                compression.to_flate2(),
            ))),
            EncoderKind::Xz => Ok(Self::Xz(XzEncoder::new(writer, compression.to_xz()))),
            EncoderKind::Zstd => Ok(Self::Zstd(ZstdEncoder::new(writer, compression.to_zstd())?)),
        }
    }

    pub fn kind(&self) -> EncoderKind {
        match self {
            Self::Write(..) => EncoderKind::Write,
            Self::Gz(..) => EncoderKind::Gz,
            Self::Bz(..) => EncoderKind::Bz,
            Self::Zlib(..) => EncoderKind::Zlib,
            Self::Xz(..) => EncoderKind::Xz,
            Self::Zstd(..) => EncoderKind::Zstd,
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
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum EncoderKind {
    Write,
    Gz,
    Bz,
    Zlib,
    Xz,
    Zstd,
}

/// Compression level.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
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
    pub fn to_level(self, encoder: EncoderKind) -> CompressionLevel {
        match encoder {
            EncoderKind::Write => CompressionLevel::Write,
            EncoderKind::Gz => CompressionLevel::Gz(self.to_flate2()),
            EncoderKind::Bz => CompressionLevel::Bz(self.to_bzip2()),
            EncoderKind::Zlib => CompressionLevel::Zlib(self.to_flate2()),
            EncoderKind::Xz => CompressionLevel::Xz(self.to_xz()),
            EncoderKind::Zstd => CompressionLevel::Zstd(self.to_zstd()),
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

    fn new_any_encoder(
        writer: VecDeque<u8>,
        u: &mut Unstructured<'_>,
    ) -> arbitrary::Result<AnyEncoder<'static, VecDeque<u8>>> {
        let kind: EncoderKind = u.arbitrary()?;
        let compression: Compression = arbitrary_compression(kind, u)?;
        let encoder = AnyEncoder::new(writer, kind, compression).unwrap();
        assert_eq!(kind, encoder.kind());
        Ok(encoder)
    }

    fn new_any_decoder(
        writer: AnyEncoder<VecDeque<u8>>,
        u: &mut Unstructured<'_>,
    ) -> arbitrary::Result<Box<dyn Read>> {
        let kind = writer.kind();
        let inner = writer.finish().unwrap();
        let any: bool = u.arbitrary()?;
        let decoder: Box<dyn Read> = if any {
            Box::new(AnyDecoder::new(inner))
        } else {
            match kind {
                EncoderKind::Write => Box::new(inner),
                EncoderKind::Gz => Box::new(flate2::read::GzDecoder::new(inner)),
                EncoderKind::Zlib => Box::new(flate2::read::ZlibDecoder::new(inner)),
                EncoderKind::Bz => Box::new(bzip2::read::BzDecoder::new(inner)),
                EncoderKind::Xz => Box::new(xz::read::XzDecoder::new(inner)),
                EncoderKind::Zstd => Box::new(zstd::stream::read::Decoder::new(inner).unwrap()),
            }
        };
        Ok(decoder)
    }

    fn arbitrary_compression(
        kind: EncoderKind,
        u: &mut Unstructured<'_>,
    ) -> arbitrary::Result<Compression> {
        let compression = u.arbitrary()?;
        Ok(match kind {
            EncoderKind::Write => compression,
            EncoderKind::Gz => compression.clamp(0, 9),
            EncoderKind::Zlib => compression.clamp(0, 9),
            EncoderKind::Bz => compression.clamp(1, 9),
            EncoderKind::Xz => compression.clamp(0, 9),
            EncoderKind::Zstd => compression.clamp(0, 22),
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
