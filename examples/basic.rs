extern crate blob;

use std::io::Write;

use blob::Blob;

const DATA: [u8; 5] = [0x1, 0x2, 0x3, 0x4, 0x5];

fn main() {
    let mut my_blob = Blob::from(&DATA[..]);

    println!("{}", my_blob);

    assert_eq!(my_blob, DATA);

    write!(my_blob, "Testing").unwrap();

    println!("{}", my_blob);
}