use std::{io::Result, fs::File};

#[test]
fn parsing() {
    let file = File::open("README.md").unwrap();
    let mds = marksad::from_reader(file).collect::<Result<Vec<_>>>().unwrap();

    panic!("{mds:?}");
}
