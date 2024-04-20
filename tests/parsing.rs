use std::fs::{self, File};

use marksad::{decode::Result, Decoder, Md};

fn test_reader_string(path: &str, f: impl Fn(&[Md<'_>])) {
    let file = File::open(path).unwrap();
    let mds = Decoder::from_reader(file)
        .collect::<Result<'_, Vec<_>>>()
        .unwrap();

    f(mds.as_slice());

    let string = fs::read_to_string(path).unwrap();
    let mds = Decoder::from_str(&string)
        .collect::<Result<'_, Vec<_>>>()
        .unwrap();

    f(mds.as_slice());
}

#[test]
fn paragraphs() {
    for path in [
        "tests/data/PARAGRAPHS.md",
        "tests/data/PARAGRAPHS_LEADING_NEWLINE.md",
        "tests/data/PARAGRAPHS_LEADING_NEWLINES.md",
    ] {
        test_reader_string(path, |mds| {
            let expected = [
                Md::Paragraph,
                Md::Text("Paragraph 1".into()),
                Md::Paragraph,
                Md::Text("Paragraph 2".into()),
            ];

            assert_eq!(mds, expected);
        });
    }
}

#[test]
fn headings() {
    for path in [
        "tests/data/HEADINGS.md",
        "tests/data/HEADINGS_LEADING_NEWLINE.md",
        "tests/data/HEADINGS_LEADING_NEWLINES.md",
    ] {
        test_reader_string(path, |mds| {
            let expected = [
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
            ];

            assert_eq!(mds, expected);
        });
    }
}
