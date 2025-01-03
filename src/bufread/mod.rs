//! Types that wrap [BufRead](std::io::BufRead) streams.

mod decoder;
mod magic_reader;

pub use self::decoder::*;
pub(crate) use self::magic_reader::*;
