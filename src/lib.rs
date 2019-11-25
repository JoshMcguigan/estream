use std::{io, io::{Read, Write}};

struct Tee<R, W> {
    reader: R,
    writer: W,
}

impl<R, W> Tee<R, W> {
    fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    fn into_inner(self) -> (R, W) {
        (self.reader, self.writer)
    }
}

impl<R, W> Read for Tee<R, W> 
    where R: Read,
          W: Write
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.writer.write(&buf[0..len]);

        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use std::{io, {io::Read}};

    use super::Tee;

    struct MockStdIn {
        inner: Vec<String>,
    }

    impl MockStdIn {
        fn new(mut inner: Vec<String>) -> Self {
            // Reverse the order of the strings because we will pop
            // them off later and we want them to come off in the order
            // the user entered them.
            inner.reverse();
            Self { inner }
        }
    }

    impl Read for MockStdIn {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if let Some(s) = self.inner.pop() {
                let bytes = s.as_bytes();
                // This only works if buf is larger than our string, but since
                // this is only test code and we don't need to exercise that behavior
                // it is okay.
                let (left, _right) = buf.split_at_mut(bytes.len());
                left.copy_from_slice(bytes);
                Ok(bytes.len())
            } else {
                Ok(0) // EOF
            }
        }
    }

    #[test]
    fn single_line() {
        let std_in = vec![
            String::from("testing"),
        ];
        let mut mock_std_in = MockStdIn::new(std_in);
        let mut mock_std_out = vec![];

        let mut tee = Tee::new(mock_std_in, mock_std_out);

        let mut buf = [0; 100];
        assert_eq!(7, tee.read(&mut buf).unwrap());
        assert_eq!(b"testing", &buf[0..7]);

        let (_, mock_std_out) = tee.into_inner();
        assert_eq!(b"testing", &mock_std_out[0..7]);
    }
}
