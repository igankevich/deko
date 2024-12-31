macro_rules! define_decoder {
    ($trait: ident) => {
        #[cfg(feature = "nightly")]
        use std::io::BorrowedCursor;
        use std::io::Empty;
        use std::io::Error;
        use std::io::ErrorKind;
        use std::io::IoSliceMut;
        use std::io::Read;

        #[cfg(feature = "nightly")]
        use crate::dispatch;
        use crate::dispatch_mut;
        use crate::Format;

        /// A decoder that decompresses the supplied input stream using any of the supported formats.
        ///
        /// The format is detected using the _magic bytes_ at the start of the stream.
        /// By default, if the format is not supported, the data is read verbatim.
        /// Use [fail_on_unknown_format](AnyDecoder::fail_on_unknown_format) to change this behaviour.
        pub struct AnyDecoder<R: $trait> {
            reader: Option<MagicReader<R>>,
            inner: InnerDecoder<MagicReader<R>>,
            fail_on_unknown_format: bool,
        }

        impl<R: $trait> AnyDecoder<R> {
            /// Create new decoder from the supplied `reader`.
            pub fn new(reader: R) -> Self {
                Self {
                    reader: Some(MagicReader::new(reader)),
                    inner: InnerDecoder::Empty(std::io::empty()),
                    fail_on_unknown_format: false,
                }
            }

            /// Throw an error when the decoder fails to detect compression format.
            ///
            /// By default no error is thrown, and the data is read verbatim.
            pub fn fail_on_unknown_format(&mut self, value: bool) {
                self.fail_on_unknown_format = value;
            }

            #[inline]
            fn get_kind(&self) -> Format {
                match self.inner {
                    InnerDecoder::Reader(..) => Format::Verbatim,
                    #[cfg(feature = "flate2")]
                    InnerDecoder::Gz(..) => Format::Gz,
                    #[cfg(feature = "bzip2")]
                    InnerDecoder::Bz(..) => Format::Bz,
                    #[cfg(feature = "flate2")]
                    InnerDecoder::Zlib(..) => Format::Zlib,
                    #[cfg(feature = "xz")]
                    InnerDecoder::Xz(..) => Format::Xz,
                    #[cfg(feature = "zstd")]
                    InnerDecoder::Zstd(..) => Format::Zstd,
                    InnerDecoder::Empty(..) => unreachable!(),
                }
            }

            /// Get the input stream format.
            ///
            /// The format is detected automatically when the data is read from the decoder.
            /// If nothing was read before calling this method, a small amount of data is read from the
            /// stream to detect the format.
            /// If the format has already been detected, this method merely returns it.
            pub fn kind(&mut self) -> Result<Format, Error> {
                self.detect()?;
                Ok(self.get_kind())
            }

            /// Get immutable reference to the underlying reader.
            pub fn get_ref(&self) -> &R {
                if let Some(r) = self.reader.as_ref() {
                    return r.get_ref();
                }
                match self.inner {
                    InnerDecoder::Reader(ref r) => r.get_ref(),
                    #[cfg(feature = "flate2")]
                    InnerDecoder::Gz(ref r) => r.get_ref().get_ref(),
                    #[cfg(feature = "bzip2")]
                    InnerDecoder::Bz(ref r) => r.get_ref().get_ref(),
                    #[cfg(feature = "flate2")]
                    InnerDecoder::Zlib(ref r) => r.get_ref().get_ref(),
                    #[cfg(feature = "xz")]
                    InnerDecoder::Xz(ref r) => r.get_ref().get_ref(),
                    #[cfg(feature = "zstd")]
                    InnerDecoder::Zstd(ref r) => crate::zstd_get_ref!($trait, r),
                    InnerDecoder::Empty(..) => unreachable!(),
                }
            }

            /// Get mutable reference to the underlying reader.
            pub fn get_mut(&mut self) -> &mut R {
                if let Some(r) = self.reader.as_mut() {
                    return r.get_mut();
                }
                match self.inner {
                    InnerDecoder::Reader(ref mut r) => r.get_mut(),
                    #[cfg(feature = "flate2")]
                    InnerDecoder::Gz(ref mut r) => r.get_mut().get_mut(),
                    #[cfg(feature = "bzip2")]
                    InnerDecoder::Bz(ref mut r) => r.get_mut().get_mut(),
                    #[cfg(feature = "flate2")]
                    InnerDecoder::Zlib(ref mut r) => r.get_mut().get_mut(),
                    #[cfg(feature = "xz")]
                    InnerDecoder::Xz(ref mut r) => r.get_mut().get_mut(),
                    #[cfg(feature = "zstd")]
                    InnerDecoder::Zstd(ref mut r) => crate::zstd_get_mut!($trait, r),
                    InnerDecoder::Empty(..) => unreachable!(),
                }
            }

            /// Return the underlying reader.
            pub fn into_inner(mut self) -> R {
                if let Some(r) = self.reader.take() {
                    return r.into_inner();
                }
                match self.inner {
                    InnerDecoder::Reader(r) => r.into_inner(),
                    #[cfg(feature = "flate2")]
                    InnerDecoder::Gz(r) => r.into_inner().into_inner(),
                    #[cfg(feature = "bzip2")]
                    InnerDecoder::Bz(r) => r.into_inner().into_inner(),
                    #[cfg(feature = "flate2")]
                    InnerDecoder::Zlib(r) => r.into_inner().into_inner(),
                    #[cfg(feature = "xz")]
                    InnerDecoder::Xz(r) => r.into_inner().into_inner(),
                    #[cfg(feature = "zstd")]
                    InnerDecoder::Zstd(r) => crate::zstd_into_inner!($trait, r),
                    InnerDecoder::Empty(..) => unreachable!(),
                }
            }

            #[inline]
            fn detect(&mut self) -> Result<(), Error> {
                if let Some(r) = self.reader.take() {
                    self.inner = InnerDecoder::new(r, self.fail_on_unknown_format)?;
                }
                Ok(())
            }
        }

        impl<R: $trait> Read for AnyDecoder<R> {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
                self.detect()?;
                dispatch_mut!(self.inner, Read::read, buf)
            }

            fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize, Error> {
                self.detect()?;
                dispatch_mut!(self.inner, Read::read_vectored, bufs)
            }

            #[cfg(feature = "nightly")]
            fn is_read_vectored(&self) -> bool {
                dispatch!(self.inner, Read::is_read_vectored)
            }

            fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
                self.detect()?;
                dispatch_mut!(self.inner, Read::read_to_end, buf)
            }

            fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Error> {
                self.detect()?;
                dispatch_mut!(self.inner, Read::read_to_string, buf)
            }

            fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
                self.detect()?;
                dispatch_mut!(self.inner, Read::read_exact, buf)
            }

            #[cfg(feature = "nightly")]
            fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> Result<(), Error> {
                self.detect()?;
                dispatch_mut!(self.inner, Read::read_buf, buf)
            }

            #[cfg(feature = "nightly")]
            fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> Result<(), Error> {
                self.detect()?;
                dispatch_mut!(self.inner, Read::read_buf_exact, buf)
            }
        }

        crate::define_inner_decoder!($trait);
    };
}

pub(crate) use define_decoder;

macro_rules! dispatch_mut {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            InnerDecoder::Reader(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "flate2")]
            InnerDecoder::Gz(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "bzip2")]
            InnerDecoder::Bz(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "flate2")]
            InnerDecoder::Zlib(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "xz")]
            InnerDecoder::Xz(ref mut r) => $method(r, $($args),*),
            #[cfg(feature = "zstd")]
            InnerDecoder::Zstd(ref mut r) => $method(r, $($args),*),
            InnerDecoder::Empty(ref mut r) => $method(r, $($args),*),
        }
    }
}

pub(crate) use dispatch_mut;

#[cfg(feature = "nightly")]
macro_rules! dispatch {
    ($inner:expr, $method:expr $(,$args:ident)*) => {
        match $inner {
            InnerDecoder::Reader(ref r) => $method(r, $($args),*),
            #[cfg(feature = "flate2")]
            InnerDecoder::Gz(ref r) => $method(r, $($args),*),
            #[cfg(feature = "bzip2")]
            InnerDecoder::Bz(ref r) => $method(r, $($args),*),
            #[cfg(feature = "flate2")]
            InnerDecoder::Zlib(ref r) => $method(r, $($args),*),
            #[cfg(feature = "xz")]
            InnerDecoder::Xz(ref r) => $method(r, $($args),*),
            #[cfg(feature = "zstd")]
            InnerDecoder::Zstd(ref r) => $method(r, $($args),*),
            InnerDecoder::Empty(ref r) => $method(r, $($args),*),
        }
    }
}

#[cfg(feature = "nightly")]
pub(crate) use dispatch;
