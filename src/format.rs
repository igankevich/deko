/// Compression format.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum Format {
    /// No encoding.
    Verbatim,
    /// Gzip encoding.
    #[cfg(feature = "flate2")]
    Gz,
    /// Bzip2 encoding.
    #[cfg(feature = "bzip2")]
    Bz,
    /// Zlib encoding.
    #[cfg(feature = "flate2")]
    Zlib,
    /// XZ encoding.
    #[cfg(feature = "xz")]
    Xz,
    /// Zstd encoding.
    #[cfg(feature = "zstd")]
    Zstd,
}
