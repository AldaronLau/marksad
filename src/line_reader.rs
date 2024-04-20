use std::{
    borrow::Cow,
    io::{self, BufRead, BufReader, ErrorKind, Read},
    str,
};

use crate::{Error, Result};

pub(crate) struct LineReader<'a>(
    Box<dyn Iterator<Item = Result<'a, Cow<'a, [u8]>>> + 'a>,
);

impl<'a> LineReader<'a> {
    pub(crate) fn from_slice(md: &'a [u8]) -> Self {
        Self(Box::new(
            md.split(|x| *x == b'\n').map(Cow::<'a, [u8]>::from).map(Ok),
        ))
    }

    pub(crate) fn from_reader(md: impl Read + 'a) -> Self {
        Self(Box::new(BufReader::new(md).lines().map(|l| {
            l.map_err(Error::Io)
                .map(|x| Cow::<'static, [u8]>::from(x.into_bytes()))
        })))
    }
}

impl<'a> Iterator for LineReader<'a> {
    type Item = Result<'a, Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        let text = match self.0.next()? {
            Ok(text) => text,
            Err(e) => return Some(Err(e)),
        };
        let text = match text {
            Cow::Borrowed(text) => match str::from_utf8(text) {
                Ok(text) => Cow::<'a, str>::from(text),
                Err(e) => {
                    return Some(Err(Error::Io(io::Error::new(
                        ErrorKind::InvalidData,
                        e,
                    ))))
                }
            },
            Cow::Owned(text) => match String::from_utf8(text) {
                Ok(text) => Cow::<'a, str>::from(text),
                Err(e) => {
                    return Some(Err(Error::Io(io::Error::new(
                        ErrorKind::InvalidData,
                        e,
                    ))))
                }
            },
        };

        Some(Ok(text))
    }
}
