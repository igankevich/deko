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

pub enum AnyEncoder<#[cfg(feature = "zstd")] 'a, W: Write> {
    Write(W),
    #[cfg(feature = "flate2")]
    Gz(GzEncoder<W>),
    #[cfg(feature = "bzip2")]
    Bz(BzEncoder<W>),
    #[cfg(feature = "flate2")]
    Zlib(ZlibEncoder<W>),
    #[cfg(feature = "xz")]
    Xz(XzEncoder<W>),
    #[cfg(feature = "zstd")]
    Zstd(ZstdEncoder<'a, W>),
}

impl_any_encoder! {
    pub fn new(writer: W, encoder: EncoderKind, compression: Compression) -> Result<Self, Error> {
        match encoder {
            EncoderKind::Write => Ok(Self::Write(writer)),
            #[cfg(feature = "flate2")]
            EncoderKind::Gz => Ok(Self::Gz(GzEncoder::new(writer, compression.to_flate2()))),
            #[cfg(feature = "bzip2")]
            EncoderKind::Bz => Ok(Self::Bz(BzEncoder::new(writer, compression.to_bzip2()))),
            #[cfg(feature = "flate2")]
            EncoderKind::Zlib => Ok(Self::Zlib(ZlibEncoder::new(
                writer,
                compression.to_flate2(),
            ))),
            #[cfg(feature = "xz")]
            EncoderKind::Xz => Ok(Self::Xz(XzEncoder::new(writer, compression.to_xz()))),
            #[cfg(feature = "zstd")]
            EncoderKind::Zstd => Ok(Self::Zstd(ZstdEncoder::new(writer, compression.to_zstd())?)),
        }
    }

    pub fn kind(&self) -> EncoderKind {
        match self {
            Self::Write(..) => EncoderKind::Write,
            #[cfg(feature = "flate2")]
            Self::Gz(..) => EncoderKind::Gz,
            #[cfg(feature = "bzip2")]
            Self::Bz(..) => EncoderKind::Bz,
            #[cfg(feature = "flate2")]
            Self::Zlib(..) => EncoderKind::Zlib,
            #[cfg(feature = "xz")]
            Self::Xz(..) => EncoderKind::Xz,
            #[cfg(feature = "zstd")]
            Self::Zstd(..) => EncoderKind::Zstd,
        }
    }

    pub fn get_ref(&self) -> &W {
        match self {
            Self::Write(ref w) => w,
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

    pub fn get_mut(&mut self) -> &mut W {
        match self {
            Self::Write(ref mut w) => w,
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

    pub fn finish(self) -> Result<W, Error> {
        match self {
            Self::Write(w) => Ok(w),
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

impl_write_for_any_encoder! {
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
    #[cfg(feature = "flate2")]
    Gz,
    #[cfg(feature = "bzip2")]
    Bz,
    #[cfg(feature = "flate2")]
    Zlib,
    #[cfg(feature = "xz")]
    Xz,
    #[cfg(feature = "zstd")]
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
            #[cfg(feature = "flate2")]
            EncoderKind::Gz => CompressionLevel::Gz(self.to_flate2()),
            #[cfg(feature = "bzip2")]
            EncoderKind::Bz => CompressionLevel::Bz(self.to_bzip2()),
            #[cfg(feature = "flate2")]
            EncoderKind::Zlib => CompressionLevel::Zlib(self.to_flate2()),
            #[cfg(feature = "xz")]
            EncoderKind::Xz => CompressionLevel::Xz(self.to_xz()),
            #[cfg(feature = "zstd")]
            EncoderKind::Zstd => CompressionLevel::Zstd(self.to_zstd()),
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

#[derive(Clone, Copy, Debug)]
pub enum CompressionLevel {
    Write,
    #[cfg(feature = "flate2")]
    Gz(flate2::Compression),
    #[cfg(feature = "bzip2")]
    Bz(bzip2::Compression),
    #[cfg(feature = "flate2")]
    Zlib(flate2::Compression),
    #[cfg(feature = "xz")]
    Xz(u32),
    #[cfg(feature = "zstd")]
    Zstd(i32),
}

macro_rules! dispatch_mut {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            Self::Write(ref mut w) => $method(w, $($args),*),
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
            Self::Write(ref w) => $method(w, $($args),*),
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

macro_rules! impl_any_encoder {
    ($($body:item)*) => {
        #[cfg(feature = "zstd")]
        impl<'a, W: Write> AnyEncoder<'a, W> {
            $($body)*
        }
        #[cfg(not(feature = "zstd"))]
        impl<W: Write> AnyEncoder<W> {
            $($body)*
        }
    }
}

use impl_any_encoder;

macro_rules! impl_write_for_any_encoder {
    ($($body:item)*) => {
        #[cfg(feature = "zstd")]
        impl<'a, W: Write> Write for AnyEncoder<'a, W> {
            $($body)*
        }
        #[cfg(not(feature = "zstd"))]
        impl<W: Write> Write for AnyEncoder<W> {
            $($body)*
        }
    }
}

use impl_write_for_any_encoder;

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

    #[cfg(feature = "zstd")]
    type AnyEncoderVecDeque = AnyEncoder<'static, VecDeque<u8>>;
    #[cfg(not(feature = "zstd"))]
    type AnyEncoderVecDeque = AnyEncoder<VecDeque<u8>>;

    fn new_any_encoder(
        writer: VecDeque<u8>,
        u: &mut Unstructured<'_>,
    ) -> arbitrary::Result<AnyEncoderVecDeque> {
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
                #[cfg(feature = "flate2")]
                EncoderKind::Gz => Box::new(flate2::read::GzDecoder::new(inner)),
                #[cfg(feature = "flate2")]
                EncoderKind::Zlib => Box::new(flate2::read::ZlibDecoder::new(inner)),
                #[cfg(feature = "bzip2")]
                EncoderKind::Bz => Box::new(bzip2::read::BzDecoder::new(inner)),
                #[cfg(feature = "xz")]
                EncoderKind::Xz => Box::new(xz::read::XzDecoder::new(inner)),
                #[cfg(feature = "zstd")]
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
            #[cfg(feature = "flate2")]
            EncoderKind::Gz => compression.clamp(0, 9),
            #[cfg(feature = "flate2")]
            EncoderKind::Zlib => compression.clamp(0, 9),
            #[cfg(feature = "bzip2")]
            EncoderKind::Bz => compression.clamp(1, 9),
            #[cfg(feature = "xz")]
            EncoderKind::Xz => compression.clamp(0, 9),
            #[cfg(feature = "zstd")]
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
