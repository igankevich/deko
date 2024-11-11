use std::io::Error;
use std::io::Write;

pub trait Finish<W> {
    fn finish(self) -> Result<W, Error>;
}

#[cfg(feature = "flate2")]
impl<W: Write> Finish<W> for flate2::write::GzEncoder<W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}

#[cfg(feature = "flate2")]
impl<W: Write> Finish<W> for flate2::write::ZlibEncoder<W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}

#[cfg(feature = "bzip2")]
impl<W: Write> Finish<W> for bzip2::write::BzEncoder<W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}

#[cfg(feature = "xz")]
impl<W: Write> Finish<W> for xz::write::XzEncoder<W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}

#[cfg(feature = "zstd")]
impl<'a, W: Write> Finish<W> for zstd::stream::write::Encoder<'a, W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}
