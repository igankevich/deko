#![cfg_attr(feature = "nightly", feature(can_vector))]
#![cfg_attr(feature = "nightly", feature(core_io_borrowed_buf))]
#![cfg_attr(feature = "nightly", feature(read_buf))]
#![cfg_attr(feature = "nightly", feature(write_all_vectored))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/igankevich/rust-docs-assets/master/deko/deko.png",
    html_favicon_url = "https://raw.githubusercontent.com/igankevich/rust-docs-assets/master/deko/deko.png"
)]

pub mod bufread;
mod constants;
mod decoder;
mod format;
mod inner_decoder;
mod magic_reader;
pub mod read;
#[cfg(test)]
pub mod test;
mod tests;
pub mod write;

pub use self::bufread::AnyDecoder;
pub(crate) use self::constants::*;
pub(crate) use self::decoder::*;
pub use self::format::*;
pub(crate) use self::inner_decoder::*;
pub(crate) use self::magic_reader::*;
pub(crate) use self::tests::*;
pub use self::write::AnyEncoder;

// TODO impl write::AnyDecoder
// TODO impl read::AnyEncoder
// TODO impl bufread::AnyEncoder
// TODO add deko-cli crate
// TODO impl AsyncRead, AsyncBufRead
// TODO add AnyDecoder constructor that takes Format as an argument. Use case: xar
