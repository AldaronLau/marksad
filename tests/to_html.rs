use std::fs;

use marksad::{html::HtmlEncoder, Md};

#[test]
fn markdown_to_html() {
    let md = [
        Md::Heading1,
        Md::Text("Heading 1".into()),
        Md::Heading2,
        Md::Text("Heading 2".into()),
        Md::Paragraph,
        Md::Text("Some paragraph".into()),
        Md::Heading3,
        Md::Text("Heading 3".into()),
        Md::Heading4,
        Md::Text("Heading 4".into()),
        Md::Heading5,
        Md::Text("Heading 5".into()),
        Md::Heading6,
        Md::Text("Heading 6".into()),
        Md::Paragraph,
        Md::Text("This is a multi-line".into()),
        Md::Text("paragraph.".into()),
        Md::Text("Good talk.".into()),
        Md::Paragraph,
        Md::Text("New paragraph, single line".into()),
        Md::Paragraph,
        Md::Text("Another".into()),
        Md::Text("multi-line".into()),
        Md::Text("paragraph".into()),
        Md::Paragraph,
        Md::Text("And".into()),
        Md::Text("another.".into()),
    ];
    let mut string = Vec::new();

    HtmlEncoder::new(md, &mut string).encode_html().unwrap();
    string.push(b'\n');

    let string = String::from_utf8(string).unwrap();
    let expected = fs::read_to_string("tests/data/test.html").unwrap();

    assert_eq!(string, expected);
}
