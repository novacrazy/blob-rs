blob
====

This crate provides a simple `Blob` structure for use in converting binary data to/from a human-readable
form using the [serde](https://serde.rs/) library.

When serializing, it will encode the binary data as base-64, and when deserializing it
can either read a base-64 encoded string or a sequence of 8-bit integers.

In essence, `Blob` is just a wrapper around a `Vec<u8>` with custom serialization functionality.

Additionally, thanks to Rust's wonderful conversion system, any type which can be converted into
a `Vec<u8>` can be converted into a `Blob`

For example, to create an empty blob with space allocated:

```
extern crate blob;

use blob::Blob;

fn main() {
    let my_blob = Blob::from(Vec::with_capacity(100));

    assert_eq!(my_blob.capacity(), 100);
}
```

Additionally, blobs can be created directly from base-64 encoded strings
either via the `decode_base64` associated function or via the `FromStr` trait

For example:

```
extern crate blob;

use std::str::FromStr;

use blob::Blob;

fn main() {
    let my_blob = Blob::from_str("AQIDBAU=").unwrap();

    assert_eq!(my_blob, [1, 2, 3, 4, 5]);
}
```

##### Other semi-related usage notes:

Since the `Blob` is expected to contain binary data, you may want to convert it into other forms.

After deserialization, `into_vec` can be called to retrieve the inner vector of data, and that can be fed into
[`std::io::Cursor`](https://doc.rust-lang.org/std/io/struct.Cursor.html) to create
an [`io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html)` + `[`io::Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) reader.