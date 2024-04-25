use std::io::Write;

use crate::{encode::Result, Md};

/// Markdown encoder
pub struct Encoder<'a, W: Write> {
    iter: Box<dyn Iterator<Item = Md<'a>> + 'a>,
    writer: W,
    open_paragraph: bool,
    open_h1: bool,
    open_h2: bool,
    open_h3: bool,
    open_h4: bool,
    open_h5: bool,
    open_h6: bool,
    not_first: bool,
    last_text: bool,
}

impl<'a, W: Write> Encoder<'a, W> {
    /// Create markdown encoder.
    pub fn new(iter: impl IntoIterator<Item = Md<'a>> + 'a, writer: W) -> Self {
        Self {
            iter: Box::new(iter.into_iter()),
            writer,
            open_paragraph: false,
            open_h1: false,
            open_h2: false,
            open_h3: false,
            open_h4: false,
            open_h5: false,
            open_h6: false,
            not_first: false,
            last_text: false,
        }
    }

    /// Encode from the iterator some markdown.
    pub fn encode_md(&mut self) -> Result {
        for md in &mut self.iter {
            let not_first = self.not_first;
            let last_text = self.last_text;

            self.last_text = false;
            self.not_first = true;

            let mut open = |text: &str| -> Result {
                if not_first {
                    self.writer.write_all(b"\n\n")?;
                }
                self.open_paragraph = false;
                self.open_h1 = false;
                self.open_h2 = false;
                self.open_h3 = false;
                self.open_h4 = false;
                self.open_h5 = false;
                self.open_h6 = false;
                Ok(self.writer.write_all(text.as_bytes())?)
            };

            match md {
                Md::Paragraph => {
                    open("")?;
                    self.open_paragraph = true;
                }
                Md::Heading1 => {
                    open("# ")?;
                    self.open_h1 = true;
                }
                Md::Heading2 => {
                    open("## ")?;
                    self.open_h2 = true;
                }
                Md::Heading3 => {
                    open("### ")?;
                    self.open_h3 = true;
                }
                Md::Heading4 => {
                    open("#### ")?;
                    self.open_h4 = true;
                }
                Md::Heading5 => {
                    open("##### ")?;
                    self.open_h5 = true;
                }
                Md::Heading6 => {
                    open("###### ")?;
                    self.open_h6 = true;
                }
                Md::Text(text) => {
                    if last_text {
                        self.writer.write_all(b" ")?;
                    }

                    self.writer.write_all(text.as_bytes())?;
                    self.last_text = true;
                }
                _ => unimplemented!(),
            }
        }

        self.writer.write_all(b"\n")?;
        Ok(())
    }
}
