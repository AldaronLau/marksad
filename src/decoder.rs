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
const UNORDERED_LIST: &[&str] = &["- ", "+ ", "* "];
const HORIZONTAL_RULE: &[&str] = &["-", "*", "_"];

/// Markdown decoder
pub struct Decoder<'a> {
    line_reader: LineReader<'a>,
    paragraph_starting: bool,
    last_list: bool,
    queued_stack: Vec<Md<'a>>,
    line: Option<Cow<'a, str>>,
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
            if let Some(line) = self.line.take() {
                break line;
            }

            let Some(line) = self.line_reader.next() else {
                if self.last_list {
                    self.last_list = false;
                    return Some(Ok(Md::ListClose));
                } else {
                    return None;
                }
            };
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

        let last_list = self.last_list;

        self.last_list = false;

        // unordered list
        {
            for ul_prefix in UNORDERED_LIST {
                line = match line {
                    Cow::Borrowed(line) => {
                        let trimmed = line.trim_start_matches(' ');
                        if let Some(line) = trimmed.strip_prefix(ul_prefix) {
                            self.last_list = true;
                            return if last_list {
                                self.queued_stack
                                    .push(Md::Text(line.trim_start().into()));
                                Some(Ok(Md::ListItem))
                            } else {
                                self.queued_stack
                                    .push(Md::Text(line.trim_start().into()));
                                self.queued_stack.push(Md::ListItem);
                                Some(Ok(Md::UnorderedList))
                            };
                        }

                        Cow::Borrowed(line)
                    }
                    Cow::Owned(mut line) => {
                        let trimmed = line.trim_start_matches(' ');
                        if trimmed.strip_prefix(ul_prefix).is_some() {
                            let s = line.len() - trimmed.len();
                            let prefix = s + ul_prefix.len();
                            let slice = &line[prefix..];
                            let ws = slice.len() - slice.trim_start().len();

                            line.drain(0..(prefix + ws));
                            self.last_list = true;
                            return if last_list {
                                self.queued_stack.push(Md::Text(line.into()));
                                Some(Ok(Md::ListItem))
                            } else {
                                self.queued_stack.push(Md::Text(line.into()));
                                self.queued_stack.push(Md::ListItem);
                                Some(Ok(Md::UnorderedList))
                            };
                        }

                        Cow::Owned(line)
                    }
                };
            }

            if last_list {
                self.line = Some(line);
                return Some(Ok(Md::ListClose));
            }
        }

        // horizontal rule
        {
            for hz_prefix in HORIZONTAL_RULE {
                if line.len() >= 3
                    && line.trim_start_matches(hz_prefix).is_empty()
                {
                    return Some(Ok(Md::HorizontalRule));
                }
            }
        }

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

        if line.contains('[') {
            'links: {
                match line {
                    Cow::Owned(ref line) => {
                        let Some((text, rest)) = line.split_once('[') else {
                            break 'links;
                        };
                        let Some((link_ref, rest)) = rest.split_once(']')
                        else {
                            break 'links;
                        };
                        let Some((empty, rest)) = rest.split_once('(') else {
                            break 'links;
                        };
                        let Some((link_val, rest)) = rest.split_once(')')
                        else {
                            break 'links;
                        };

                        if !empty.is_empty() {
                            break 'links;
                        }

                        let text = text.to_owned();
                        let link_ref = link_ref.to_owned();
                        let link_val = link_val.to_owned();
                        let rest = rest.to_owned();

                        self.queued_stack.push(Md::Text(text.into()));
                        self.queued_stack.push(Md::LinkRef(link_ref.into()));
                        self.queued_stack.push(Md::LinkVal(link_val.into()));

                        // FIXME: Same as below
                        return if self.paragraph_starting {
                            self.queued_stack.push(Md::Text(rest.into()));
                            self.paragraph_starting = false;
                            Some(Ok(Md::Paragraph))
                        } else {
                            Some(Ok(Md::Text(rest.into())))
                        };
                    },
                    Cow::Borrowed(line) => {
                        let Some((text, rest)) = line.split_once('[') else {
                            break 'links;
                        };
                        let Some((link_ref, rest)) = rest.split_once(']')
                        else {
                            break 'links;
                        };
                        let Some((empty, rest)) = rest.split_once('(') else {
                            break 'links;
                        };
                        let Some((link_val, rest)) = rest.split_once(')')
                        else {
                            break 'links;
                        };

                        if !empty.is_empty() {
                            break 'links;
                        }

                        self.queued_stack.push(Md::Text(rest.into()));
                        self.queued_stack.push(Md::LinkVal(link_val.into()));
                        self.queued_stack.push(Md::LinkRef(link_ref.into()));

                        // FIXME: Same as below
                        return if self.paragraph_starting {
                            self.queued_stack.push(Md::Text(text.into()));
                            self.paragraph_starting = false;
                            Some(Ok(Md::Paragraph))
                        } else {
                            Some(Ok(Md::Text(text.into())))
                        };
                    }
                }
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
            last_list: false,
            queued_stack: Vec::new(),
            line: None,
        }
    }
}
