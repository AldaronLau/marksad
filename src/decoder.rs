use std::{borrow::Cow, io::Read, str};

use crate::{decode::Result, line_reader::LineReader, Md};

const HEADING1: &str = "#";
const HEADING2: &str = "##";
const HEADING3: &str = "###";
const HEADING4: &str = "####";
const HEADING5: &str = "#####";
const HEADING6: &str = "######";
// This one is invalid, should warn and output a paragraph
const HEADING7: &str = "#######";

/// Markdown decoder
pub struct Decoder<'a> {
    line_reader: LineReader<'a>,
    paragraph_starting: bool,
    queued_stack: Vec<Md<'a>>,
}

impl<'a> Decoder<'a> {
    /// Create markdown decoder from I/O reader.
    pub fn from_reader(md: impl Read + 'a) -> Self {
        Self::from(LineReader::from_reader(md))
    }

    /// Create markdown decoder from string slice.
    pub fn from_str(md: &'a str) -> Self {
        Self::from_slice(md.as_bytes())
    }

    /// Create markdown decoder from byte slice.
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

        let mut line = loop {
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

        'headings: {
            if line.starts_with(HEADING7) {
                break 'headings;
            }

            // Check for headings
            for (heading_prefix, heading_md) in [
                (HEADING6, Md::Heading6),
                (HEADING5, Md::Heading5),
                (HEADING4, Md::Heading4),
                (HEADING3, Md::Heading3),
                (HEADING2, Md::Heading2),
                (HEADING1, Md::Heading1),
            ] {
                line = match line {
                    Cow::Borrowed(line) => {
                        if let Some(line) = line.strip_prefix(heading_prefix) {
                            self.queued_stack
                                .push(Md::Text(line.trim_start().into()));
                            self.paragraph_starting = true;
                            return Some(Ok(heading_md));
                        }

                        Cow::Borrowed(line)
                    }
                    Cow::Owned(mut line) => {
                        if line.strip_prefix(heading_prefix).is_some() {
                            let slice = &line[heading_prefix.len()..];
                            let ws = slice.len() - slice.trim_start().len();

                            line.drain(0..heading_prefix.len() + ws);
                            self.queued_stack.push(Md::Text(line.into()));
                            self.paragraph_starting = true;
                            return Some(Ok(heading_md));
                        }

                        Cow::Owned(line)
                    }
                };
            }
        }

        if self.paragraph_starting {
            self.queued_stack.push(Md::Text(line));

            self.paragraph_starting = false;
            Some(Ok(Md::Paragraph))
        } else {
            Some(Ok(Md::Text(line)))
        }
    }
}

impl<'a> From<LineReader<'a>> for Decoder<'a> {
    fn from(line_reader: LineReader<'a>) -> Self {
        Self {
            line_reader,
            paragraph_starting: true,
            queued_stack: Vec::new(),
        }
    }
}
