blob
====

This crate provides a dedicated `Blob` structure for use in storing,
encoding and decoding to/from base-64, with support for type-level encoding
configurations suitable for url-safe base-64.

When serializing, it will encode the binary data as base-64, and when deserializing it
can either read and decode a base-64 encoded string or a raw sequence of bytes.

Example using `FromStr::from_str`:

```rust
extern crate blob;

use std::str::FromStr;

use blob::Blob;

fn main() {
    let my_blob: Blob = Blob::from_str("AQIDBAU=").unwrap();

    assert_eq!(my_blob, [1, 2, 3, 4, 5]);
}
```