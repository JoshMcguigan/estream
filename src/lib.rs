use std::{io, io::{Read, Write}};

pub struct Tee<R, W> {
    reader: R,
    writer: W,
    buf: [u8; 8192],
    len: usize,
}

impl<R, W> Tee<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer, buf: [0; 8192], len: 0 }
    }

    #[cfg(test)]
    fn get_writer_ref(&self) -> &W {
        &self.writer
    }
}

impl<R, W> Read for Tee<R, W> 
    where R: Read,
          W: Write
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let total_len = self.reader.read(&mut self.buf)?;
        let newline_index = self.buf[0..total_len].iter().position(|b| *b == '\n' as u8);
        let len = if let Some(newline_index) = newline_index {
            newline_index + 1
        } else {
            total_len
        };
        self.writer.write(&self.buf[0..len])?;
        &mut buf[0..len].copy_from_slice(&self.buf[0..len]);

        if len < total_len {
            // This means we didn't write out all of the bytes we got in. This is done
            // to allow the reader a chance to process each line before we print any bytes
            // on the next line, which gives the reader a chance to write their own bytes to
            // standard out, without interleaving.

            // copy total_len - len elements to start of self.buf, then set self.len
        }

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
    fn single_read() {
        let std_in = vec![
            String::from("testing"),
        ];
        let mock_std_in = MockStdIn::new(std_in);
        let mock_std_out = vec![];

        let mut tee = Tee::new(mock_std_in, mock_std_out);

        let mut buf = [0; 100];
        assert_eq!(7, tee.read(&mut buf).unwrap());
        assert_eq!(b"testing", &buf[0..7]);

        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing", &mock_std_out[0..7]);
    }

    #[test]
    fn single_read_ends_in_newline() {
        let std_in = vec![
            String::from("testing\n"),
        ];
    }

    // TODO handle windows newlines

    #[test]
    fn single_read_with_newline() {
        let std_in = vec![
            String::from("testing\nis great"),
        ];
        let mock_std_in = MockStdIn::new(std_in);
        let mock_std_out = vec![];

        let mut tee = Tee::new(mock_std_in, mock_std_out);

        let mut buf = [0; 100];
        assert_eq!(8, tee.read(&mut buf).unwrap());
        assert_eq!(b"testing\n", &buf[0..8]);

        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing\n", &mock_std_out[0..8]);

        assert_eq!(8, tee.read(&mut buf).unwrap());
        assert_eq!(b"is great", &buf[0..8]);

        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"is great", &mock_std_out[0..8]);
    }

    #[test]
    fn multiline() {
        let std_in = vec![
            String::from("testing.."),
            String::from("."),
            String::from(".\n."),
            String::from("is fun"),
        ];
    }
}
