//! Blob
//!
//! This crate provides a dedicated `Blob` structure for use in storing,
//! encoding and decoding to/from base-64, with support for type-level encoding
//! configurations suitable for url-safe base-64.
//!
//! When serializing, it will encode the binary data as base-64, and when deserializing it
//! can either read and decode a base-64 encoded string or a raw sequence of bytes.
//!
//! Example using `FromStr::from_str`:
//!
//! ```
//! extern crate blob;
//!
//! use std::str::FromStr;
//!
//! use blob::Blob;
//!
//! fn main() {
//!     let my_blob: Blob = Blob::from_str("AQIDBAU=").unwrap();
//!
//!     assert_eq!(my_blob, [1, 2, 3, 4, 5]);
//! }
//! ```

#![deny(missing_docs)]

extern crate base64;
extern crate serde;

use std::borrow::{Borrow, BorrowMut};
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::iter::{Extend, FromIterator, IntoIterator};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice::{Iter, IterMut};
use std::str::FromStr;
use std::vec::IntoIter;

/// Trait used for statically typed Blob encoding configs
pub trait Config: Send + Sync {
    /// Associated base-64 config
    const CONFIG: base64::Config;
}

macro_rules! impl_configs {
    ($($(#[$($attrs:tt)*])* $name:ident: $config:ident,)*) => {
        $(
            $(#[$($attrs)*])*
            pub enum $name {}

            impl Config for $name {
                const CONFIG: base64::Config = base64::$config;
            }
        )*
    }
}

impl_configs! {
    /// As per `crypt(3)` requirements
    Crypt: CRYPT,

    /// Standard character set with padding.
    Standard: STANDARD,

    /// Standard character set without padding.
    StandardNoPad: STANDARD_NO_PAD,

    /// URL-safe character set with padding
    UrlSafe: URL_SAFE,

    /// URL-safe character set without padding
    UrlSafeNoPad: URL_SAFE_NO_PAD,
}

/// Blob structure containing binary data
///
/// Interally, the blob is stored as a plain `Vec<u8>`, and some
/// methods are exposed from that. If you need full access to the
/// underlying `Vec`, use `borrow()` or `borrow_mut()`
pub struct Blob<C: Config = Standard> {
    data: Vec<u8>,
    _config: PhantomData<C>,
}

impl<C: Config> Default for Blob<C> {
    #[inline]
    fn default() -> Self {
        Blob {
            data: Vec::new(),
            _config: PhantomData,
        }
    }
}

impl<C: Config> Blob<C> {
    /// Create a new empty `Blob`
    #[inline]
    pub fn new() -> Blob<C> {
        Blob::default()
    }

    /// Create a `Blob` from an underlying `Vec`
    #[inline]
    pub fn from_vec(vec: Vec<u8>) -> Blob<C> {
        Blob {
            data: vec,
            _config: PhantomData,
        }
    }

    /// Create a new `Blob` with the given capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Blob<C> {
        Blob::from_vec(Vec::with_capacity(capacity))
    }

    /// Returns the number of bytes the `Blob` can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserves capacity for at least additional more bytes to be inserted in the given `Blob`
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional)
    }

    /// Use a different encoding configuration for the `Blob`
    #[inline(always)]
    pub fn with_config<E: Config>(self) -> Blob<E> {
        Blob {
            data: self.data,
            _config: PhantomData,
        }
    }

    /// Encode the `Blob` to a base-64 string
    #[inline]
    pub fn encode_base64(&self) -> String {
        base64::encode_config(&self.data, C::CONFIG)
    }

    /// Encodes the `Blob` as base-64 to an `io::Writer`, avoiding intermediate allocations
    pub fn encode_to<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        let mut encoder = base64::write::EncoderWriter::new(&mut writer, C::CONFIG);

        encoder.write_all(&self.data)
    }

    /// Decode base-64 encoded data into a `Blob`
    pub fn decode_base64<T>(encoded: T) -> Result<Blob<C>, base64::DecodeError>
    where
        T: AsRef<[u8]>,
    {
        // perform as_ref here to only monomorphize the decoder once
        base64::decode_config(encoded.as_ref(), C::CONFIG).map(Blob::from_vec)
    }

    /// Decodes some base-64 data and appends it to the `Blob`
    #[inline]
    pub fn append_base64<T>(&mut self, encoded: T) -> Result<(), base64::DecodeError>
    where
        T: AsRef<[u8]>,
    {
        // perform as_ref here to only monomorphize the decoder once
        base64::decode_config_buf(encoded.as_ref(), C::CONFIG, &mut self.data)
    }

    /// Consume self and return the inner `Vec<u8>`
    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

impl<C: Config> FromStr for Blob<C> {
    type Err = base64::DecodeError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Blob::decode_base64(s)
    }
}

impl<C: Config> Clone for Blob<C> {
    #[inline]
    fn clone(&self) -> Blob<C> {
        Blob {
            data: self.data.clone(),
            _config: PhantomData,
        }
    }
}

impl<C: Config> fmt::Debug for Blob<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Blob").field(&self.data).finish()
    }
}

impl<C: Config> Display for Blob<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        base64::display::Base64Display::with_config(&self.data, C::CONFIG).fmt(f)
    }
}

impl<C: Config> Hash for Blob<C> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<C: Config> Write for Blob<C> {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.write(buf)
    }

    #[inline(always)]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.data.write_all(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()
    }
}

impl<C: Config> FromIterator<u8> for Blob<C> {
    fn from_iter<I>(iter: I) -> Blob<C>
    where
        I: IntoIterator<Item = u8>,
    {
        Blob::from_vec(Vec::from_iter(iter))
    }
}

impl<C: Config> Extend<u8> for Blob<C> {
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = u8>,
    {
        self.data.extend(iter)
    }
}

impl<'a, C: Config> Extend<&'a u8> for Blob<C> {
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a u8>,
    {
        self.data.extend(iter)
    }
}

impl<C: Config> IntoIterator for Blob<C> {
    type Item = u8;
    type IntoIter = IntoIter<u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, C: Config> IntoIterator for &'a Blob<C> {
    type Item = &'a u8;
    type IntoIter = Iter<'a, u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, C: Config> IntoIterator for &'a mut Blob<C> {
    type Item = &'a mut u8;
    type IntoIter = IterMut<'a, u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<C: Config> Deref for Blob<C> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<C: Config> DerefMut for Blob<C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T, C: Config> From<T> for Blob<C>
where
    T: Into<Vec<u8>>,
{
    #[inline(always)]
    fn from(value: T) -> Blob<C> {
        Blob::from_vec(value.into())
    }
}

impl<C: Config> PartialEq<Self> for Blob<C> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl<C: Config> Eq for Blob<C> {}

impl<T, C: Config> PartialEq<T> for Blob<C>
where
    Vec<u8>: PartialEq<T>,
{
    #[inline(always)]
    fn eq(&self, other: &T) -> bool {
        self.data == *other
    }
}

impl<C: Config> AsRef<[u8]> for Blob<C> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<C: Config> AsRef<Vec<u8>> for Blob<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Vec<u8> {
        &self.data
    }
}

impl<C: Config> AsMut<[u8]> for Blob<C> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl<C: Config> AsMut<Vec<u8>> for Blob<C> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
}

impl<C: Config> Borrow<Vec<u8>> for Blob<C> {
    fn borrow(&self) -> &Vec<u8> {
        &self.data
    }
}

impl<C: Config> BorrowMut<Vec<u8>> for Blob<C> {
    fn borrow_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
}

impl<C: Config> serde::Serialize for Blob<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded = self.encode_base64();

        serializer.serialize_str(encoded.as_str())
    }
}

impl<'de, C: Config> serde::Deserialize<'de> for Blob<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BlobVisitor<C: Config>(PhantomData<C>);

        impl<'de, C: Config> serde::de::Visitor<'de> for BlobVisitor<C> {
            type Value = Blob<C>;

            fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str("base64 encoded string or byte sequence")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                FromStr::from_str(value).map_err(E::custom)
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Blob::from_vec(value.to_owned()))
            }

            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Blob::from_vec(value))
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                // Preallocate the bytes vec if possible, but remain conservative
                let mut bytes = Vec::with_capacity(visitor.size_hint().unwrap_or(0).min(4096));

                while let Some(byte) = visitor.next_element()? {
                    bytes.push(byte);
                }

                Ok(Blob::from_vec(bytes))
            }
        }

        deserializer.deserialize_any(BlobVisitor(PhantomData))
    }
}
