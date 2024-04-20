/// A markdown warning
#[non_exhaustive]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum WarningKind {
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

/// Markdown warning
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Warning<'a> {
    line_text: &'a str,
    line_number: u16,
    warning: WarningKind,
}
