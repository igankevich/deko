#![cfg_attr(feature = "nightly", feature(can_vector))]
#![cfg_attr(feature = "nightly", feature(read_buf))]
#![cfg_attr(feature = "nightly", feature(write_all_vectored))]
#![cfg_attr(feature = "nightly", feature(core_io_borrowed_buf))]
#![doc = include_str!("../README.md")]

pub mod bufread;
mod format;
#[cfg(test)]
pub mod test;
pub mod write;

pub use self::bufread::AnyDecoder;
pub use self::format::*;
pub use self::write::AnyEncoder;
