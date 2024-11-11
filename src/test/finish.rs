use std::io::Error;
use std::io::Write;

pub trait Finish<W> {
    fn finish(self) -> Result<W, Error>;
}

impl<W: Write> Finish<W> for flate2::write::GzEncoder<W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}

impl<W: Write> Finish<W> for flate2::write::ZlibEncoder<W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}

impl<W: Write> Finish<W> for bzip2::write::BzEncoder<W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}

impl<W: Write> Finish<W> for xz::write::XzEncoder<W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}

impl<'a, W: Write> Finish<W> for zstd::stream::write::Encoder<'a, W> {
    fn finish(self) -> Result<W, Error> {
        Self::finish(self)
    }
}
