//! Types that wrap [Read](std::io::Read) streams.

mod decoder;
mod magic_reader;

pub use self::decoder::*;
pub(crate) use self::magic_reader::*;
