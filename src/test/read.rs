use std::collections::VecDeque;
use std::io::IoSliceMut;
use std::io::Read;

use arbitrary::Unstructured;
use arbtest::arbtest;

pub fn test_read_all<F, R>(mut f: F)
where
    F: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> R,
    R: Read,
{
    test_read(&mut f);
    test_read_exact(&mut f);
    test_read_vectored(&mut f);
    test_read_to_end(&mut f);
    test_read_to_string(&mut f);
    // TODO read_buf, read_buf_exact
}

fn test_read<F, R>(f: &mut F)
where
    F: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> R,
    R: Read,
{
    arbtest(|u| {
        let expected: VecDeque<u8> = u.arbitrary()?;
        let buf_len: usize = u.int_in_range(0..=16 * 4096)?;
        let mut reader = f(expected.clone(), u);
        let mut actual: Vec<u8> = vec![0_u8; buf_len];
        let n = reader.read(&mut actual[..]).unwrap();
        assert_eq!(
            &expected.iter().cloned().collect::<Vec<_>>()[..n],
            &actual[..n],
            "n = {}, expected len = {}, buffer len = {}",
            n,
            expected.len(),
            buf_len
        );
        assert!(
            n <= expected.len().min(buf_len)
                && ((n != 0 && buf_len != 0 && expected.len() != 0)
                    || (n == 0 && (buf_len == 0 || expected.len() == 0))),
            "n = {}, expected len = {}, buffer len = {}",
            n,
            expected.len(),
            buf_len
        );
        Ok(())
    });
}

fn test_read_vectored<F, R>(f: &mut F)
where
    F: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> R,
    R: Read,
{
    arbtest(|u| {
        // When `num_buffers == 0` or `buf_len == 0` zstd fails with
        // "Operation made no progress over multiple calls, due to output buffer being full".
        let expected: VecDeque<u8> = u.arbitrary()?;
        let num_buffers = u.int_in_range(1..=3)?;
        let mut buffers: Vec<Vec<u8>> = Vec::from_iter((0..num_buffers).map(|_| {
            let buf_len = u.int_in_range(1..=100).unwrap();
            vec![0_u8; buf_len]
        }));
        let mut slices: Vec<IoSliceMut> = Vec::new();
        for buf in buffers.iter_mut() {
            slices.push(IoSliceMut::new(&mut buf[..]));
        }
        let mut reader = f(expected.clone(), u);
        let n = reader.read_vectored(&mut slices[..]).unwrap();
        let buf_len: usize = buffers.iter().map(|x| x.len()).sum();
        assert_eq!(
            &expected.iter().cloned().collect::<Vec<_>>()[..n],
            &buffers.iter().cloned().flatten().collect::<Vec<_>>()[..n],
            "n = {}, expected len = {}, buffer len = {}",
            n,
            expected.len(),
            buf_len
        );
        assert!(
            n <= expected.len().min(buf_len)
                && ((n != 0 && buf_len != 0 && expected.len() != 0)
                    || (n == 0 && (buf_len == 0 || expected.len() == 0))),
            "n = {}, expected len = {}, buffer len = {}",
            n,
            expected.len(),
            buf_len
        );
        Ok(())
    });
}

fn test_read_to_end<F, R>(f: &mut F)
where
    F: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> R,
    R: Read,
{
    arbtest(|u| {
        let expected: VecDeque<u8> = u.arbitrary()?;
        let mut reader = f(expected.clone(), u);
        let mut actual: Vec<u8> = Vec::new();
        let n = reader.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        assert_eq!(n, expected.len());
        Ok(())
    });
}

fn test_read_to_string<F, R>(f: &mut F)
where
    F: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> R,
    R: Read,
{
    arbtest(|u| {
        let expected: String = u.arbitrary()?;
        let bytes: VecDeque<u8> = VecDeque::from_iter(expected.as_bytes().to_vec().into_iter());
        let mut reader = f(bytes.clone(), u);
        let mut actual = String::new();
        let n = reader.read_to_string(&mut actual).unwrap();
        let expected_len = expected.as_bytes().len();
        assert_eq!(expected, actual);
        assert_eq!(n, expected_len);
        Ok(())
    });
}

fn test_read_exact<F, R>(f: &mut F)
where
    F: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> R,
    R: Read,
{
    arbtest(|u| {
        let expected: VecDeque<u8> = u.arbitrary()?;
        let mut reader = f(expected.clone(), u);
        let buf_len = u.int_in_range(0..=expected.len()).unwrap();
        let mut actual: Vec<u8> = vec![0_u8; buf_len];
        reader.read_exact(&mut actual[..]).unwrap();
        assert_eq!(
            &expected.iter().cloned().collect::<Vec<_>>()[..buf_len],
            &actual[..]
        );
        Ok(())
    });
}
