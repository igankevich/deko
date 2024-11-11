//! Types that wrap [BufRead](std::io::BufRead) streams.

mod decoder;

pub use self::decoder::*;
