use std::{
    io,
    io::{Read, Write},
};

pub struct Tee<R, W> {
    reader: R,
    writer: W,
    buf: [u8; 8192],
    cap: usize,
}

impl<R, W> Tee<R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            buf: [0; 8192],
            cap: 0,
        }
    }

    /// This method must write and flush all bytes so we can
    /// be sure our output is written to the correct line of
    /// stdout.
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;

        Ok(())
    }

    #[cfg(test)]
    fn get_writer_ref(&self) -> &W {
        &self.writer
    }
}

impl<R, W> Read for Tee<R, W>
where
    R: Read,
    W: Write,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cap > 0 {
            // return old data
            // We know this will fit into the user buffer because
            // it gets in here first by entering their buffer in the
            // section below. This assumption will not hold if the user
            // first passes a large buffer, then a small one.
            buf[..self.cap].copy_from_slice(&self.buf[..self.cap]);
            self.write(&buf[..self.cap])?;
            let len = self.cap;
            self.cap = 0;
            return Ok(len);
        }
        let total_len = self.reader.read(buf)?;
        let newline_index = buf[0..total_len].iter().position(|b| *b == b'\n');
        if let Some(newline_index) = newline_index {
            let cutoff = newline_index + 1;
            self.write(&buf[0..cutoff])?;
            let len_remaining = total_len - cutoff;
            // We only save up to the length of our buffer. This should
            // be fine, because we'd only drop data if the user buffer
            // was larger than our own, which is unlikely.
            let len_to_save = if len_remaining < self.buf.len() {
                len_remaining
            } else {
                self.buf.len()
            };
            // save the bytes after the newline in our internal buffer
            self.buf[0..len_to_save].copy_from_slice(&buf[cutoff..(cutoff + len_to_save)]);
            self.cap = len_to_save;
            Ok(cutoff)
        } else {
            self.write(&buf[0..total_len])?;
            Ok(total_len)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io, io::Read};

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
        let std_in = vec![String::from("testing")];
        let mock_std_in = MockStdIn::new(std_in);
        let mock_std_out = vec![];

        let mut tee = Tee::new(mock_std_in, mock_std_out);

        let mut buf = [0; 100];
        assert_eq!(7, tee.read(&mut buf).unwrap());
        assert_eq!(b"testing", &buf[0..7]);

        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing", &mock_std_out.as_slice());
    }

    #[test]
    fn single_read_ends_in_newline() {
        let std_in = vec![String::from("testing\n")];
        let mock_std_in = MockStdIn::new(std_in);
        let mock_std_out = vec![];

        let mut tee = Tee::new(mock_std_in, mock_std_out);

        let mut buf = [0; 100];
        assert_eq!(8, tee.read(&mut buf).unwrap());
        assert_eq!(b"testing\n", &buf[0..8]);

        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing\n", &mock_std_out.as_slice());
    }

    #[test]
    fn single_read_with_newline() {
        let std_in = vec![String::from("testing\nis great")];
        let mock_std_in = MockStdIn::new(std_in);
        let mock_std_out = vec![];

        let mut tee = Tee::new(mock_std_in, mock_std_out);

        let mut buf = [0; 100];
        assert_eq!(8, tee.read(&mut buf).unwrap());
        assert_eq!(b"testing\n", &buf[0..8]);

        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing\n", &mock_std_out.as_slice());

        assert_eq!(8, tee.read(&mut buf).unwrap());
        assert_eq!(b"is great", &buf[0..8]);

        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing\nis great", &mock_std_out.as_slice());
    }

    #[test]
    fn multiline() {
        let std_in = vec![
            String::from("testing.."),
            String::from(".\n."),
            String::from("is fun"),
        ];
        let mock_std_in = MockStdIn::new(std_in);
        let mock_std_out = vec![];

        let mut tee = Tee::new(mock_std_in, mock_std_out);

        let mut buf = [0; 100];

        // first read
        assert_eq!(9, tee.read(&mut buf).unwrap());
        assert_eq!(b"testing..", &buf[0..9]);
        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing..", &mock_std_out.as_slice());

        // second read
        // should stop at the newline to allow our two outbound streams to sync
        assert_eq!(2, tee.read(&mut buf).unwrap());
        assert_eq!(b".\n", &buf[0..2]);
        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing...\n", &mock_std_out.as_slice());

        // third read
        assert_eq!(1, tee.read(&mut buf).unwrap());
        assert_eq!(b".", &buf[0..1]);
        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing...\n.", &mock_std_out.as_slice());

        // fourth read
        assert_eq!(6, tee.read(&mut buf).unwrap());
        assert_eq!(b"is fun", &buf[0..6]);
        let mock_std_out = tee.get_writer_ref();
        assert_eq!(b"testing...\n.is fun", &mock_std_out.as_slice());
    }
}
