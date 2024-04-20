use std::{fs::File, io::Result};

use marksad::Md;

#[test]
fn paragraphs() {
    let file = File::open("tests/PARAGRAPHS.md").unwrap();
    let mds = marksad::from_reader(file)
        .collect::<Result<Vec<_>>>()
        .unwrap();
    let expected = [
        Md::Paragraph,
        Md::Text("Paragraph 1".into()),
        Md::Paragraph,
        Md::Text("Paragraph 2".into()),
    ];

    assert_eq!(mds.as_slice(), expected.as_slice());
}
