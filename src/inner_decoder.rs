macro_rules! import_decoders {
    (Read) => {
        #[cfg(feature = "bzip2")]
        use bzip2::read::BzDecoder;
        #[cfg(feature = "flate2")]
        use flate2::read::GzDecoder;
        #[cfg(feature = "flate2")]
        use flate2::read::ZlibDecoder;
        #[cfg(feature = "xz")]
        use xz::read::XzDecoder;
        // TODO ???
        #[cfg(feature = "zstd")]
        use zstd::stream::read::Decoder as ZstdDecoder;
    };
    (BufRead) => {
        #[cfg(feature = "bzip2")]
        use bzip2::bufread::BzDecoder;
        #[cfg(feature = "flate2")]
        use flate2::bufread::GzDecoder;
        #[cfg(feature = "flate2")]
        use flate2::bufread::ZlibDecoder;
        #[cfg(feature = "xz")]
        use xz::bufread::XzDecoder;
        #[cfg(feature = "zstd")]
        use zstd::stream::read::Decoder as ZstdDecoder;
    };
}

pub(crate) use import_decoders;

macro_rules! define_inner_decoder {
    ($trait: ident) => {
        use crate::MAX_MAGIC_BYTES;

        crate::import_decoders!($trait);

        enum InnerDecoder<R: $trait> {
            Empty(Empty),
            Reader(R),
            #[cfg(feature = "flate2")]
            Gz(GzDecoder<R>),
            #[cfg(feature = "bzip2")]
            Bz(BzDecoder<R>),
            #[cfg(feature = "flate2")]
            Zlib(ZlibDecoder<R>),
            #[cfg(feature = "xz")]
            Xz(XzDecoder<R>),
            #[cfg(feature = "zstd")]
            Zstd(crate::zstd_decoder!($trait, R)),
        }

        impl<R: $trait> InnerDecoder<MagicReader<R>> {
            fn new(
                mut reader: MagicReader<R>,
                fail_on_unknown_format: bool,
            ) -> Result<Self, Error> {
                let magic = reader.read_magic()?;
                let magic = if magic.len() >= MAX_MAGIC_BYTES {
                    magic
                } else {
                    reader.read_magic_slow()?
                };
                match magic {
                    // https://tukaani.org/xz/xz-file-format-1.0.4.txt
                    #[cfg(feature = "xz")]
                    [0xfd, b'7', b'z', b'X', b'Z', 0, ..] => {
                        Ok(InnerDecoder::Xz(XzDecoder::new(reader)))
                    }
                    // RFC8878
                    #[cfg(feature = "zstd")]
                    [0x28, 0xb5, 0x2f, 0xfd, ..] => Ok(InnerDecoder::Zstd(
                        crate::zstd_decoder_new!($trait, reader)?,
                    )),
                    // RFC1952
                    #[cfg(feature = "flate2")]
                    [0x1f, 0x8b, 0x08, ..] => Ok(InnerDecoder::Gz(GzDecoder::new(reader))),
                    // https://en.wikipedia.org/wiki/Bzip2
                    #[cfg(feature = "bzip2")]
                    [b'B', b'Z', b'h', ..] => Ok(InnerDecoder::Bz(BzDecoder::new(reader))),
                    // https://www.rfc-editor.org/rfc/rfc1950
                    #[cfg(feature = "flate2")]
                    [cmf, flg, ..]
                        if zlib_cm(*cmf) == 8
                            && zlib_cinfo(*cmf) <= 7
                            && ((*cmf as u16) * 256 + (*flg as u16)) % 31 == 0 =>
                    {
                        Ok(InnerDecoder::Zlib(ZlibDecoder::new(reader)))
                    }
                    // TODO pbzx
                    _ if fail_on_unknown_format => Err(Error::new(
                        ErrorKind::InvalidData,
                        "unknown compression format",
                    )),
                    _ => Ok(InnerDecoder::Reader(reader)),
                }
            }
        }

        #[cfg(feature = "flate2")]
        const fn zlib_cm(x: u8) -> u8 {
            x & 0b1111
        }

        #[cfg(feature = "flate2")]
        const fn zlib_cinfo(x: u8) -> u8 {
            (x >> 4) & 0b1111
        }
    };
}

pub(crate) use define_inner_decoder;

macro_rules! zstd_decoder_new {
    (BufRead, $reader: ident) => {
        ZstdDecoder::with_buffer($reader)
    };
    (Read, $reader: ident) => {
        ZstdDecoder::new($reader)
    };
}

pub(crate) use zstd_decoder_new;

macro_rules! zstd_decoder {
    (BufRead, $r: ident) => {
        ZstdDecoder<'static, $r>
    };
    (Read, $r: ident) => {
        ZstdDecoder<'static, std::io::BufReader<$r>>
    };
}

pub(crate) use zstd_decoder;

macro_rules! zstd_get_ref {
    (BufRead, $r: ident) => {
        $r.get_ref().get_ref()
    };
    (Read, $r: ident) => {
        $r.get_ref().get_ref().get_ref()
    };
}

pub(crate) use zstd_get_ref;

macro_rules! zstd_get_mut {
    (BufRead, $r: ident) => {
        $r.get_mut().get_mut()
    };
    (Read, $r: ident) => {
        $r.get_mut().get_mut().get_mut()
    };
}

pub(crate) use zstd_get_mut;

macro_rules! zstd_into_inner {
    (BufRead, $r: ident) => {
        $r.finish().into_inner()
    };
    (Read, $r: ident) => {
        $r.finish().into_inner().into_inner()
    };
}

pub(crate) use zstd_into_inner;
