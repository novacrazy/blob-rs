extern crate blob;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use blob::Blob;

const DATA: [u8; 5] = [1, 2, 3, 4, 5];

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BlobFixture {
    my_blob: Blob,
}

#[test]
fn test_blob() {
    let blob: Blob = Blob::from(&DATA[..]);

    let encoded = blob.encode_base64();

    let decoded_blob = Blob::decode_base64(encoded.as_str()).unwrap();

    assert_eq!(blob, decoded_blob);
}

#[test]
fn test_blob_serde() {
    use serde_json::{from_str, to_string_pretty};

    let fixture = BlobFixture {
        my_blob: Blob::from(&DATA[..]),
    };

    let encoded = to_string_pretty(&fixture).unwrap();

    println!("Encoded {}", encoded);

    let decoded = from_str(encoded.as_str()).unwrap();

    assert_eq!(fixture, decoded);
}

#[test]
fn test_blob_array() {
    use serde_json::from_str;

    let fixture_struct = BlobFixture {
        my_blob: Blob::from(&DATA[..]),
    };

    let fixture_str = r#"{"my_blob": [1, 2, 3, 4, 5]}"#;

    let decoded: BlobFixture = from_str(fixture_str).unwrap();

    assert_eq!(decoded, fixture_struct);
}

#[test]
#[should_panic]
fn test_blob_array_overflow() {
    use serde_json::from_str;

    let fixture_str = r#"{"my_blob": [1, 2, 3000, 4, 5]}"#;

    let _: BlobFixture = from_str(fixture_str).unwrap();
}
