//! Blob
//!
//! This crate provides a simple `Blob` structure for use in converting binary data to/from a human-readable
//! form using the [serde](https://serde.rs/) library.
//!
//! When serializing, it will encode the binary data as base-64, and when deserializing it
//! can either read a base-64 encoded string or a sequence of 8-bit integers.
//!
//! In essence, `Blob` is just a wrapper around a `Vec<u8>` with custom serialization functionality.
//!
//! Additionally, thanks to Rust's wonderful conversion system, any type which can be converted into
//! a `Vec<u8>` can be converted into a `Blob`
//!
//! For example, to create an empty blob with space allocated:
//!
//! ```
//! extern crate blob;
//!
//! use blob::Blob;
//!
//! fn main() {
//!     let my_blob = Blob::from(Vec::with_capacity(100));
//!
//!     assert_eq!(my_blob.capacity(), 100);
//! }
//! ```
//!
//! Additionally, blobs can be created directly from base-64 encoded strings
//! either via the `decode_base64` associated function or via the `FromStr` trait
//!
//! For example:
//!
//! ```
//! extern crate blob;
//!
//! use std::str::FromStr;
//!
//! use blob::Blob;
//!
//! fn main() {
//!     let my_blob = Blob::from_str("AQIDBAU=").unwrap();
//!
//!     assert_eq!(my_blob, [1, 2, 3, 4, 5]);
//! }
//! ```
//!
//! ##### Other semi-related usage notes:
//!
//! Since the `Blob` is expected to contain binary data, you may want to convert it into other forms.
//!
//! After deserialization, `into_vec` can be called to retrieve the inner vector of data, and that can be fed into
//! [`std::io::Cursor`](https://doc.rust-lang.org/std/io/struct.Cursor.html) to create
//! an [`io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html)` + `[`io::Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) reader.

#![deny(missing_docs)]

extern crate base64;
extern crate serde;

use std::ops::{Deref, DerefMut};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

/// Blob structure containing binary data
#[derive(Debug, Default, Clone, Hash, PartialEq)]
pub struct Blob(Vec<u8>);

impl Blob {
    /// Encode the blob to a base-64 string
    pub fn encode_base64(&self) -> String {
        base64::encode(&self.0)
    }

    /// Decode a base-64 encoded string into a `Blob`
    pub fn decode_base64(encoded: &str) -> Result<Blob, base64::DecodeError> {
        Ok(Blob(base64::decode(encoded)?))
    }

    /// Consume self and return the inner `Vec<u8>`
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl FromStr for Blob {
    type Err = base64::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Blob(base64::decode(s)?))
    }
}

impl Display for Blob {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let encoded = self.encode_base64();

        f.write_str(encoded.as_str())
    }
}

impl serde::Serialize for Blob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let encoded = self.encode_base64();

        serializer.serialize_str(encoded.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Blob {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        struct BlobVisitor;

        impl<'de> serde::de::Visitor<'de> for BlobVisitor {
            type Value = Blob;

            fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str("base64 encoded string or byte sequence")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: serde::de::Error {
                FromStr::from_str(value).map_err(E::custom)
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> where E: serde::de::Error {
                Ok(Blob::from(value))
            }

            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E> where E: serde::de::Error {
                Ok(Blob(value))
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error> where V: serde::de::SeqAccess<'de> {
                // Preallocate the bytes vec if possible
                let mut bytes = Vec::with_capacity(visitor.size_hint().unwrap_or(0));

                while let Some(byte) = visitor.next_element()? {
                    bytes.push(byte);
                }

                Ok(Blob(bytes))
            }
        }

        deserializer.deserialize_any(BlobVisitor)
    }
}

impl Deref for Blob {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Blob {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Blob where T: Into<Vec<u8>> {
    fn from(value: T) -> Blob {
        Blob(value.into())
    }
}

impl<T> PartialEq<T> for Blob where Vec<u8>: PartialEq<T> {
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl AsRef<Vec<u8>> for Blob {
    fn as_ref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl AsMut<Vec<u8>> for Blob {
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}