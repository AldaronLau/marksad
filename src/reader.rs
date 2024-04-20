use std::{io::Read, str};

use crate::{line_reader::LineReader, Md, Result};

/// Markdown reader
pub struct Reader<'a> {
    line_reader: LineReader<'a>,
    paragraph_starting: bool,
    queued_stack: Vec<Md<'a>>,
}

impl<'a> Iterator for Reader<'a> {
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

impl<'a> From<LineReader<'a>> for Reader<'a> {
    fn from(line_reader: LineReader<'a>) -> Self {
        Self {
            line_reader,
            paragraph_starting: false,
            queued_stack: Vec::new(),
        }
    }
}

/// Create markdown reader from string slice
pub fn from_str(md: &str) -> Reader<'_> {
    from_slice(md.as_bytes())
}

/// Create markdown reader from byte slice
pub fn from_slice(md: &[u8]) -> Reader<'_> {
    Reader::from(LineReader::from_slice(md))
}

/// Create markdown reader from I/O reader
pub fn from_reader<'a>(md: impl Read + 'a) -> Reader<'a> {
    Reader::from(LineReader::from_reader(md))
}
