use std::collections::VecDeque;
use std::io::BufRead;

use arbitrary::Unstructured;
use arbtest::arbtest;

pub fn test_bufread_all<F, R>(f: F)
where
    F: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> R,
    R: BufRead,
{
    test_fill_buf(f);
}

fn test_fill_buf<F, R>(mut f: F)
where
    F: for<'a> FnMut(VecDeque<u8>, &mut Unstructured<'a>) -> R,
    R: BufRead,
{
    arbtest(|u| {
        let expected: VecDeque<u8> = u.arbitrary()?;
        let mut reader = f(expected.clone(), u);
        let mut actual = Vec::new();
        loop {
            let buf = reader.fill_buf().unwrap();
            if buf.is_empty() {
                break;
            }
            let n = u.int_in_range(0..=buf.len())?;
            actual.extend(&buf[..n]);
            reader.consume(n);
        }
        assert_eq!(expected.iter().cloned().collect::<Vec<_>>(), actual);
        Ok(())
    });
}
