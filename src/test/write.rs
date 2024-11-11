use std::collections::VecDeque;
use std::io::IoSlice;
use std::io::Read;
use std::io::Write;

use arbitrary::Unstructured;
use arbtest::arbtest;

pub fn test_write_trait<F1, F2, W, R>(mut make_writer: F1, mut make_reader: F2)
where
    F1: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> arbitrary::Result<W>,
    F2: for<'a> FnMut(W, &mut Unstructured<'a>) -> arbitrary::Result<R>,
    W: Write,
    R: Read,
{
    test_write(&mut make_writer, &mut make_reader);
    test_write_vectored(&mut make_writer, &mut make_reader);
    test_write_all(&mut make_writer, &mut make_reader);
}

fn test_write<F1, F2, W, R>(mut make_writer: F1, mut make_reader: F2)
where
    F1: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> arbitrary::Result<W>,
    F2: for<'a> FnMut(W, &mut Unstructured<'a>) -> arbitrary::Result<R>,
    W: Write,
    R: Read,
{
    arbtest(|u| {
        let mut writer = make_writer(VecDeque::new(), u)?;
        let expected: Vec<u8> = u.arbitrary()?;
        let n: usize = u.int_in_range(0..=expected.len())?;
        writer.write(&expected[..n]).unwrap();
        let mut reader = make_reader(writer, u)?;
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(&expected[..n], &actual[..n]);
        Ok(())
    });
}

fn test_write_vectored<F1, F2, W, R>(mut make_writer: F1, mut make_reader: F2)
where
    F1: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> arbitrary::Result<W>,
    F2: for<'a> FnMut(W, &mut Unstructured<'a>) -> arbitrary::Result<R>,
    W: Write,
    R: Read,
{
    arbtest(|u| {
        // When `num_buffers == 0` or `buf_len == 0` some encoders
        // just don't write anything to the output stream.
        let mut writer = make_writer(VecDeque::new(), u)?;
        let expected: Vec<u8> = u.arbitrary()?;
        let num_buffers: usize = u.int_in_range(1..=3)?;
        let mut buffers: Vec<IoSlice> = Vec::new();
        let mut offset: usize = 0;
        for _ in 0..num_buffers {
            let remaining = expected.len() - offset;
            if remaining == 0 {
                break;
            }
            let buf_len = u.int_in_range(1..=remaining)?;
            if buf_len == 0 {
                break;
            }
            buffers.push(IoSlice::new(&expected[offset..(offset + buf_len)]));
            offset += buf_len;
        }
        let n = writer.write_vectored(&buffers[..]).unwrap();
        assert!(n <= offset);
        let mut reader = make_reader(writer, u)?;
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(&expected[..n], &actual[..n]);
        Ok(())
    });
}

fn test_write_all<F1, F2, W, R>(mut make_writer: F1, mut make_reader: F2)
where
    F1: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> arbitrary::Result<W>,
    F2: for<'a> FnMut(W, &mut Unstructured<'a>) -> arbitrary::Result<R>,
    W: Write,
    R: Read,
{
    arbtest(|u| {
        let mut writer = make_writer(VecDeque::new(), u)?;
        let expected: Vec<u8> = u.arbitrary()?;
        writer.write_all(&expected[..]).unwrap();
        let mut reader = make_reader(writer, u)?;
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        Ok(())
    });
}
