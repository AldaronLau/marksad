use std::{
    borrow::Cow,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Result},
    str,
};

/// A markdown warning
#[non_exhaustive]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Warning {
    /// Depending on markdown flavor may be a heading, may not
    ///
    /// `#Ambiguous Heading`
    ///
    /// To disambiguate, use either
    ///
    ///  - `# My Heading`
    ///  - `\# Line that starts with a #`
    AmbiguousHeading,
}

/// Types of elements in a markdown file.
///
/// First documented is preferred
#[non_exhaustive]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Md<'a> {
    /// Markdown warning
    // FIXME: Maybe not here
    Warning {
        line_text: &'a str,
        line_number: u16,
        warning: Warning,
    },
    /// Line ends in `  `, or `<br>` or `\\`
    LineBreak,
    /// Line exclusively `---` or more `-`, `___` or more `_`, `***` or more
    /// `*`
    HorizontalRule,
    /// Separated by two newlines
    Paragraph,
    /// Start with `#` or underlined with any number of `=`
    Heading1,
    /// Start with `##` or underlined with any number of `-`
    Heading2,
    /// Start with `###`
    Heading3,
    /// Start with `####`
    Heading4,
    /// Start with `#####`
    Heading5,
    /// Start with `######`
    Heading6,
    /// Start with `>`, open a quote block
    QuoteOpen,
    /// Same as above, close a quote block
    QuoteClose,
    /// Open an ordered list, with any of `1234567890`, followed by `. `
    OrderedList,
    /// Open an unordered list, with any of `-+*`, followed by ` `
    UnorderedList,
    /// Definition list (followed by term as `Text`, then `ListItem` which is
    /// any following line starting with `: `)
    DefinitionList,
    /// List item, continues item as long as indented 4 spaces or tab
    ListItem,
    /// In an unordered list, `[x] ` or `[ ] `
    ListTask(bool),
    /// Close a list
    ListClose,
    /// When image is not followed by a new paragraph
    Caption,
    /// Italic style text, `*` or `_`
    Italic(bool),
    /// Bold style text, `**` or `__`
    Bold(bool),
    /// Bold and italic style text, `***` or `___`
    BoldItalic(bool),
    /// Superscript style text `^`
    Superscript(bool),
    /// Subscript style text  `~`
    Subscript(bool),
    /// Strikethrough style text `~~`
    Strikethrough(bool),
    /// Highlight style text `==`
    Highlight(bool),
    /// Underline `--`, `<ins></ins>`, `<u></u>`
    Underline(bool),
    /// Same line warning text:
    ///
    /// ```md
    /// > [!WARNING]
    /// > My warning text
    /// >
    /// > Some more information about the warning
    /// ```
    ///
    /// ```md
    /// > [!WARNING]
    /// > My warning text
    /// ```
    ///
    /// Separate line more information:
    ///
    /// ```md
    /// > [!WARNING]
    /// >
    /// > Some more information about the warning
    /// ```
    ///
    /// Can also use double space:
    ///
    /// ```md
    /// > [!WARNING]  
    /// > Some more information about the warning
    /// ```
    ///
    /// This also works (but not recommended)
    ///
    /// ```md
    /// > [!WARNING]
    /// My warning text
    ///
    /// > [!WARNING]  
    /// Some more information about the warning
    /// ```
    ///
    /// Alternative syntax:
    ///
    /// ```md
    /// !!! warning "My warning text"
    ///
    ///     Some more information about the warning
    /// ```
    ///
    /// Can be
    ///
    ///  - `NOTE`
    ///  - `TIP`
    ///  - `WARNING`
    ///  - `CAUTION`
    ///  - `IMPORTANT`
    ///
    /// Or with `!!!` syntax
    ///
    ///  - `note`
    ///  - `astract`
    ///  - `info`
    ///  - `tip`
    ///  - `success`
    ///  - `question`
    ///  - `warning`
    ///  - `failure`
    ///  - `danger`
    ///  - `bug`
    ///  - `example`
    ///  - `quote`
    Admonition(Cow<'a, str>),
    /// Unexpanded:
    ///
    /// ```md
    /// ++  My summary
    ///     Continued summary
    ///
    ///     Details that can be expanded
    /// ```
    ///
    /// Expanded:
    ///
    /// ```md
    /// +++ My summary
    ///     Continued summary
    ///
    ///     Details that can be collapsed
    /// ```
    ///
    /// `<details><summary>My summary</summary></details>`
    ///
    /// Alternative syntax unexpanded:
    ///
    /// ```md
    /// ??? info "My summary"
    ///
    ///     Details that can be expanded
    /// ```
    ///
    /// Alternative syntax expanded:
    ///
    /// ```md
    /// ???+ info "My summary"
    ///
    ///     Details that can be collapsed
    /// ```
    Details(Cow<'a, str>, bool),
    /// `[Some comment text]: #`
    Comment(Cow<'a, str>),
    /// Open and close with one to two backticks
    Code(Cow<'a, str>),
    /// Open and close with tripple+ backtick or `~`, or 4 spaces
    Codeblock(Cow<'a, str>),
    /// Select syntax highlighting for following codeblock
    SyntaxHighlighting(Cow<'a, str>),
    /// Plain text after any of the above markers
    Text(Cow<'a, str>),
    /// Image alt text, referenced by number
    ImageNum(Cow<'a, str>, u16),
    /// Image alt text, file referenced by alt text
    ImageRef(Cow<'a, str>),
    /// Link text `<https://example.org>` or `https://example.org`
    Link(Cow<'a, str>),
    /// Link text with number `[My link][1]`
    LinkNum(Cow<'a, str>, u16),
    /// Link reference `[My link]`
    LinkRef(Cow<'a, str>),
    /// Link definition key `[My link]: https://example.org` or
    /// `[1]: https://example.org`
    ///
    /// Also; `[My link]: <https://example.org>` or
    /// `[1]: <https://example.org>`
    LinkKey(Cow<'a, str>),
    /// Link definition value (same as above), `(https://example.org)` when
    /// directly following a `LinkRef` or `ImageRef`
    LinkVal(Cow<'a, str>),
    /// `[This is a link with a title](https://example.org "My Title")`,
    /// `[This is a link with a title](https://example.org 'My Title')`,
    /// `[This is a link with a title](https://example.org (My Title))`
    ///
    /// `[My link]: <https://example.org> "My Title"`,
    /// `[1]: <https://example.org> "My Title"`,
    /// `[My link]: <https://example.org> 'My Title'`,
    /// `[1]: <https://example.org> 'My Title'`,
    /// `[My link]: <https://example.org> (My Title)`,
    /// `[1]: <https://example.org> (My Title)`
    Title(Cow<'a, str>),
    /// `Something[^reftext]`
    FootnoteRef(Cow<'a, str>),
    /// `[^reftext]: First paragraph in footnote`
    FootnoteOpen(Cow<'a, str>),
    /// No longer indented, footnote has ended
    FootnoteClose,
    /// Custom ID for heading `# Heading {#custom-id}`
    HeadingId(Cow<'a, str>),
    /// Start table column `|` or start table align left `:---`
    TableLeft,
    /// Start table column `|` or start table align center `:---:`
    TableCentered,
    /// Start table column `|` or start table align right `---:`
    TableRight,
    /// Next cell in row, `|` or next column if end of row
    TableCell,
}

/// Markdown reader
pub struct Reader<'a> {
    line_reader: LineReader<'a>,
    paragraph_starting: bool,
    queued_stack: Vec<Md<'a>>,
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<Md<'a>>;

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

struct LineReader<'a>(Box<dyn Iterator<Item = Result<Cow<'a, [u8]>>> + 'a>);

impl<'a> Iterator for LineReader<'a> {
    type Item = Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        let text = match self.0.next()? {
            Ok(text) => text,
            Err(e) => return Some(Err(e)),
        };
        let text = match text {
            Cow::Borrowed(text) => match str::from_utf8(text) {
                Ok(text) => Cow::<'a, str>::from(text),
                Err(e) => {
                    return Some(Err(Error::new(ErrorKind::InvalidData, e)))
                }
            },
            Cow::Owned(text) => match String::from_utf8(text) {
                Ok(text) => Cow::<'a, str>::from(text),
                Err(e) => {
                    return Some(Err(Error::new(ErrorKind::InvalidData, e)))
                }
            },
        };

        Some(Ok(text))
    }
}

/// Create markdown reader from string slice
pub fn from_str<'a>(md: &'a str) -> Reader<'a> {
    from_slice(md.as_bytes())
}

/// Create markdown reader from byte slice
pub fn from_slice<'a>(md: &'a [u8]) -> Reader<'a> {
    Reader::from(LineReader(Box::new(
        md.split(|x| *x == b'\n').map(Cow::<'a, [u8]>::from).map(Ok),
    )))
}

/// Create markdown reader from I/O reader
pub fn from_reader<'a>(md: impl Read + 'a) -> Reader<'a> {
    Reader::from(LineReader(Box::new(
        BufReader::new(md)
            .lines()
            .map(|l| l.map(|x| Cow::<'static, [u8]>::from(x.into_bytes()))),
    )))
}
