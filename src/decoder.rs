use std::{io::Read, str};

use crate::{decode::Result, line_reader::LineReader, Md};

/// Markdown decoder
pub struct Decoder<'a> {
    line_reader: LineReader<'a>,
    paragraph_starting: bool,
    queued_stack: Vec<Md<'a>>,
}

impl<'a> Decoder<'a> {
    /// Create markdown decoder from I/O reader
    pub fn from_reader(md: impl Read + 'a) -> Self {
        Self::from(LineReader::from_reader(md))
    }

    /// Create markdown decoder from string slice
    pub fn from_str(md: &'a str) -> Self {
        Self::from_slice(md.as_bytes())
    }

    /// Create markdown decoder from byte slice
    pub fn from_slice(md: &'a [u8]) -> Self {
        Self::from(LineReader::from_slice(md))
    }
}

impl<'a> Iterator for Decoder<'a> {
    type Item = Result<'a, Md<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(queued) = self.queued_stack.pop() {
            return Some(Ok(queued));
        };

        let line = loop {
            let line = self.line_reader.next()?;
            let line = match line {
                Ok(text) => text,
                Err(e) => return Some(Err(e)),
            };

            if line.is_empty() {
                self.paragraph_starting = true;
                continue;
            }

            break line;
        };

        self.queued_stack.push(Md::Text(line));

        self.paragraph_starting = false;
        Some(Ok(Md::Paragraph))
    }
}

impl<'a> From<LineReader<'a>> for Decoder<'a> {
    fn from(line_reader: LineReader<'a>) -> Self {
        Self {
            line_reader,
            paragraph_starting: false,
            queued_stack: Vec::new(),
        }
    }
}
