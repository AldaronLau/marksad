//! HTML encoding of markdown

use std::{
    borrow::Cow,
    io::{self, Write},
    result,
};

use crate::Md;

/// `Result` type alias for convenience
pub type Result<T = (), E = Error> = result::Result<T, E>;

/// HTML conversion error
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Default)]
enum ListKind {
    #[default]
    Ordered,
    Unordered,
    Definition,
    #[allow(dead_code)]
    Task,
}

/// A markdown to HTML encoder
pub struct HtmlEncoder<'a, W: Write> {
    iter: Box<dyn Iterator<Item = Md<'a>> + 'a>,
    writer: W,
    open_paragraph: bool,
    open_h1: bool,
    open_h2: bool,
    open_h3: bool,
    open_h4: bool,
    open_h5: bool,
    open_h6: bool,
    open_li: bool,
    last_text: bool,
    list_kinds: [ListKind; 6],
    list_level: u8,
    link_ref: Option<Cow<'a, str>>,
}

impl<'a, W: Write> HtmlEncoder<'a, W> {
    /// Create a new HTML encoder
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
            open_li: false,
            list_kinds: [ListKind::default(); 6],
            list_level: 0,
            last_text: false,
            link_ref: None,
        }
    }

    /// Encode from the iterator some HTML
    pub fn encode_html(&mut self) -> Result {
        fn close<W>(flag: &mut bool, text: &str, writer: &mut W) -> Result
        where
            W: Write,
        {
            if *flag {
                *flag = false;
                writer.write_all(text.as_bytes())?;
            }

            Ok(())
        }

        for md in &mut self.iter {
            let last_text = self.last_text;

            self.last_text = false;

            let mut open = |text: &str| -> Result {
                close(&mut self.open_paragraph, "</p>", &mut self.writer)?;
                close(&mut self.open_h1, "</h1>", &mut self.writer)?;
                close(&mut self.open_h2, "</h2>", &mut self.writer)?;
                close(&mut self.open_h3, "</h3>", &mut self.writer)?;
                close(&mut self.open_h4, "</h4>", &mut self.writer)?;
                close(&mut self.open_h5, "</h5>", &mut self.writer)?;
                close(&mut self.open_h6, "</h6>", &mut self.writer)?;
                // FIXME - could panic
                if self.open_li {
                    close(
                        &mut self.open_li,
                        match self.list_kinds[usize::from(self.list_level - 1)]
                        {
                            ListKind::Ordered => "</li>",
                            ListKind::Unordered => "</li>",
                            ListKind::Definition => "</dd>",
                            ListKind::Task => unimplemented!(),
                        },
                        &mut self.writer,
                    )?;
                }
                // FIXME: Escape HTML
                Ok(self.writer.write_all(text.as_bytes())?)
            };

            match md {
                Md::Paragraph => {
                    open("<p>")?;
                    self.open_paragraph = true;
                }
                Md::Heading1 => {
                    open("<h1>")?;
                    self.open_h1 = true;
                }
                Md::Heading2 => {
                    open("<h2>")?;
                    self.open_h2 = true;
                }
                Md::Heading3 => {
                    open("<h3>")?;
                    self.open_h3 = true;
                }
                Md::Heading4 => {
                    open("<h4>")?;
                    self.open_h4 = true;
                }
                Md::Heading5 => {
                    open("<h5>")?;
                    self.open_h5 = true;
                }
                Md::Heading6 => {
                    open("<h6>")?;
                    self.open_h6 = true;
                }
                Md::OrderedList => {
                    open("<ol>")?;
                    self.list_kinds[usize::from(self.list_level)] =
                        ListKind::Ordered;
                    self.list_level += 1;
                    // FIXME: Panic
                    assert!(self.list_level < 6);
                }
                Md::UnorderedList => {
                    open("<ul>")?;
                    self.list_kinds[usize::from(self.list_level)] =
                        ListKind::Unordered;
                    self.list_level += 1;
                    // FIXME: Panic
                    assert!(self.list_level < 6);
                }
                Md::DefinitionList => {
                    open("<dl>")?;
                    self.list_kinds[usize::from(self.list_level)] =
                        ListKind::Definition;
                    self.list_level += 1;
                    // FIXME: Panic
                    assert!(self.list_level < 6);
                }
                Md::ListItem => {
                    open("<li>")?;
                    self.open_li = true;
                }
                Md::ListClose => {
                    // FIXME - could panic
                    self.list_level -= 1;

                    close(
                        &mut self.open_li,
                        match self.list_kinds[usize::from(self.list_level)] {
                            ListKind::Ordered => "</li></ol>",
                            ListKind::Unordered => "</li></ul>",
                            ListKind::Definition => "</dd></dl>",
                            ListKind::Task => unimplemented!(),
                        },
                        &mut self.writer,
                    )?;
                }
                Md::Text(text) => {
                    if last_text {
                        self.writer.write_all(b" ")?;
                    }

                    self.writer.write_all(text.as_bytes())?;
                    self.last_text = true;
                }
                Md::HorizontalRule => {
                    self.writer.write_all(b"<hr>")?;
                }
                Md::LinkRef(text) => {
                    if self.link_ref.is_none() {
                        self.link_ref = Some(text);
                    } else {
                        unimplemented!()
                    }
                }
                Md::LinkVal(text) => {
                    if let Some(link_ref) = self.link_ref.take() {
                        self.writer.write_all(b"<a href='")?;
                        self.writer.write_all(text.as_bytes())?;
                        self.writer.write_all(b"'>")?;
                        self.writer.write_all(link_ref.as_bytes())?;
                        self.writer.write_all(b"</a>")?;
                    } else {
                        unimplemented!()
                    }
                }
                _ => unimplemented!(),
            }
        }

        close(&mut self.open_paragraph, "</p>", &mut self.writer)?;
        close(&mut self.open_h1, "</h1>", &mut self.writer)?;
        close(&mut self.open_h2, "</h2>", &mut self.writer)?;
        close(&mut self.open_h3, "</h3>", &mut self.writer)?;
        close(&mut self.open_h4, "</h4>", &mut self.writer)?;
        close(&mut self.open_h5, "</h5>", &mut self.writer)?;
        close(&mut self.open_h6, "</h6>", &mut self.writer)?;

        if self.open_li {
            close(
                &mut self.open_li,
                match self.list_kinds[usize::from(self.list_level - 1)] {
                    ListKind::Ordered => "</li>",
                    ListKind::Unordered => "</li>",
                    ListKind::Definition => "</dd>",
                    ListKind::Task => unimplemented!(),
                },
                &mut self.writer,
            )?;
        }

        Ok(())
    }
}
