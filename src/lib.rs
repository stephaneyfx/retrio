// Copyright (C) 2018 Stephane Raux. Distributed under the MIT license.

#![deny(missing_docs)]
#![deny(warnings)]

//! This crate defines a wrapper around readers (buffered or not) and writers
//! to retry on IO errors of kind `Interrupted`.

#[cfg(test)]
extern crate partial_io;

use std::fmt;
use std::io::{BufRead, ErrorKind, Read, self, Write};

/// Wrapper around readers, buffered readers and writers to automatically retry
/// on IO errors of kind `Interrupted`.
///
/// All methods are forwarded to the wrapped type.
#[derive(Clone, Debug)]
pub struct Retry<T> {
    inner: T,
}

impl<T> Retry<T> {
    /// Wraps a value.
    pub fn new(inner: T) -> Self {
        Retry {inner}
    }

    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Read> Read for Retry<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.inner.read(buf) {
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                res => return res,
            }
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }
}

impl<T: BufRead> BufRead for Retry<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        loop {
            match self.inner.fill_buf() {
                Ok(_) => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        self.inner.fill_buf()
    }

    fn consume(&mut self, n: usize) {
        self.inner.consume(n)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_until(byte, buf)
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_line(buf)
    }
}

impl<T: Write> Write for Retry<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        loop {
            match self.inner.write(buf) {
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                res => return res,
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> io::Result<()> {
        self.inner.write_fmt(args)
    }
}

#[cfg(test)]
mod tests {
    use partial_io::{PartialOp, PartialRead, PartialWrite};
    use std::io::BufReader;
    use super::*;

    #[test]
    fn reads() {
        let input = &b"Read test"[..];
        let ops = vec![PartialOp::Err(ErrorKind::Interrupted)];
        let mut reader = Retry::new(PartialRead::new(input, ops));
        let mut out = vec![0u8; input.len()];
        assert_eq!(reader.read(&mut out).unwrap(), input.len());
        assert_eq!(&out[..], input);
    }

    #[test]
    fn reads_buffered() {
        let input = &b"Read test"[..];
        let ops = vec![PartialOp::Err(ErrorKind::Interrupted)];
        let mut reader = Retry::new(BufReader::with_capacity(input.len(),
            PartialRead::new(input, ops)));
        assert_eq!(reader.fill_buf().unwrap(), input);
    }

    #[test]
    fn writes() {
        let input = &b"Write test"[..];
        let ops = vec![PartialOp::Err(ErrorKind::Interrupted)];
        let mut writer = Retry::new(PartialWrite::new(Vec::<u8>::new(), ops));
        assert_eq!(writer.write(input).unwrap(), input.len());
        assert_eq!(&writer.into_inner().into_inner()[..], input);
    }
}
