#![cfg_attr(feature = "nightly", feature(can_vector))]
#![cfg_attr(feature = "nightly", feature(read_buf))]

pub mod bufread;
pub mod write;

pub use self::bufread::AnyDecoder;
pub use self::write::AnyEncoder;
