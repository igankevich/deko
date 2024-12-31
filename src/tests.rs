macro_rules! define_decoder_tests {
    () => {
        #[cfg(test)]
        mod tests {
            use std::collections::VecDeque;
            use std::io::Write;

            use arbitrary::Unstructured;
            use arbtest::arbtest;

            use super::*;
            use crate::test::test_read_trait;
            use crate::test::Finish;
            use crate::test::NBytesReader;

            #[cfg(feature = "flate2")]
            #[test]
            fn write_gz_read_any() {
                use flate2::write::GzEncoder;
                use flate2::Compression;
                arbtest(|u| {
                    let compression = Compression::new(u.int_in_range(0..=9)?);
                    let writer = GzEncoder::new(Vec::new(), compression);
                    write_some_read_any(writer, u)
                });
            }

            #[cfg(feature = "bzip2")]
            #[test]
            fn write_bz_read_any() {
                use bzip2::write::BzEncoder;
                use bzip2::Compression;
                arbtest(|u| {
                    let compression = Compression::new(u.int_in_range(1..=9)?);
                    let writer = BzEncoder::new(Vec::new(), compression);
                    write_some_read_any(writer, u)
                });
            }

            #[cfg(feature = "flate2")]
            #[test]
            fn write_zlib_read_any() {
                use flate2::write::ZlibEncoder;
                use flate2::Compression;
                arbtest(|u| {
                    let compression = Compression::new(u.int_in_range(1..=9)?);
                    let writer = ZlibEncoder::new(Vec::new(), compression);
                    write_some_read_any(writer, u)
                });
            }

            #[cfg(feature = "xz")]
            #[test]
            fn write_xz_read_any() {
                use xz::write::XzEncoder;
                arbtest(|u| {
                    let compression = u.int_in_range(0..=9)?;
                    let writer = XzEncoder::new(Vec::new(), compression);
                    write_some_read_any(writer, u)
                });
            }

            #[cfg(feature = "zstd")]
            #[test]
            fn write_zstd_read_any() {
                use zstd::stream::write::Encoder;
                arbtest(|u| {
                    let compression = u.int_in_range(0..=22)?;
                    let writer = Encoder::new(Vec::new(), compression).unwrap();
                    write_some_read_any(writer, u)
                });
            }

            #[test]
            fn test_any_decoder() {
                #[cfg(feature = "flate2")]
                test_read_trait(new_gz_reader);
                #[cfg(feature = "flate2")]
                test_read_trait(new_zlib_reader);
                #[cfg(feature = "bzip2")]
                test_read_trait(new_bz_reader);
                #[cfg(feature = "xz")]
                test_read_trait(new_xz_reader);
                #[cfg(feature = "zstd")]
                test_read_trait(new_zstd_reader);
            }

            #[cfg(feature = "flate2")]
            fn new_gz_reader(
                vec: VecDeque<u8>,
                u: &mut Unstructured,
            ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
                use flate2::write::GzEncoder;
                use flate2::Compression;
                let compression = Compression::new(u.int_in_range(0..=9).unwrap());
                let mut writer = GzEncoder::new(Vec::new(), compression);
                let bytes = vec.into_iter().collect::<Vec<_>>();
                writer.write_all(&bytes).unwrap();
                let compressed: VecDeque<u8> = writer.finish().unwrap().into();
                let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
                AnyDecoder::new(reader)
            }

            #[cfg(feature = "flate2")]
            fn new_zlib_reader(
                vec: VecDeque<u8>,
                u: &mut Unstructured,
            ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
                use flate2::write::ZlibEncoder;
                use flate2::Compression;
                let compression = Compression::new(u.int_in_range(0..=9).unwrap());
                let mut writer = ZlibEncoder::new(Vec::new(), compression);
                let bytes = vec.into_iter().collect::<Vec<_>>();
                writer.write_all(&bytes).unwrap();
                let compressed: VecDeque<u8> = writer.finish().unwrap().into();
                let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
                AnyDecoder::new(reader)
            }

            #[cfg(feature = "bzip2")]
            fn new_bz_reader(
                vec: VecDeque<u8>,
                u: &mut Unstructured,
            ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
                use bzip2::write::BzEncoder;
                use bzip2::Compression;
                let compression = Compression::new(u.int_in_range(1..=9).unwrap());
                let mut writer = BzEncoder::new(Vec::new(), compression);
                let bytes = vec.into_iter().collect::<Vec<_>>();
                writer.write_all(&bytes).unwrap();
                let compressed: VecDeque<u8> = writer.finish().unwrap().into();
                let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
                AnyDecoder::new(reader)
            }

            #[cfg(feature = "xz")]
            fn new_xz_reader(
                vec: VecDeque<u8>,
                u: &mut Unstructured,
            ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
                use xz::write::XzEncoder;
                let compression = u.int_in_range(0..=9).unwrap();
                let mut writer = XzEncoder::new(Vec::new(), compression);
                let bytes = vec.into_iter().collect::<Vec<_>>();
                writer.write_all(&bytes).unwrap();
                let compressed: VecDeque<u8> = writer.finish().unwrap().into();
                let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
                AnyDecoder::new(reader)
            }

            #[cfg(feature = "zstd")]
            fn new_zstd_reader(
                vec: VecDeque<u8>,
                u: &mut Unstructured,
            ) -> AnyDecoder<NBytesReader<VecDeque<u8>>> {
                use zstd::stream::write::Encoder;
                let compression = u.int_in_range(0..=22).unwrap();
                let mut writer = Encoder::new(Vec::new(), compression).unwrap();
                let bytes = vec.into_iter().collect::<Vec<_>>();
                writer.write_all(&bytes).unwrap();
                let compressed: VecDeque<u8> = writer.finish().unwrap().into();
                let reader = NBytesReader::new(compressed, u.int_in_range(1..=100).unwrap());
                AnyDecoder::new(reader)
            }

            fn write_some_read_any<W: Write + Finish<Vec<u8>>>(
                mut writer: W,
                u: &mut Unstructured,
            ) -> arbitrary::Result<()> {
                let expected: Vec<u8> = u.arbitrary()?;
                writer.write_all(&expected).unwrap();
                let compressed = writer.finish().unwrap();
                let capacity = u.int_in_range(1..=4096)?;
                let reader = NBytesReader::new(&compressed[..], capacity);
                //eprintln!("compressed {:#x?}", compressed);
                let mut reader = AnyDecoder::new(reader);
                let mut actual = Vec::new();
                reader.read_to_end(&mut actual).unwrap();
                assert_eq!(expected, actual);
                Ok(())
            }
        }
    };
}

pub(crate) use define_decoder_tests;
