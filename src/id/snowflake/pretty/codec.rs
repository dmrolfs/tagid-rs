//! Encoding and decoding of numeric IDs using a custom alphabet.
//!
//! This module provides a `Codec` trait and implementations for encoding
//! and decoding numbers using a configurable alphabet-based encoding scheme.
//!
//! The primary implementation is `AlphabetCodec`, which allows transforming
//! `i64` values into a human-readable string representation and back.

use once_cell::sync::Lazy;
use tailcall::tailcall;

/// A predefined base-23 alphabet for encoding.
///
/// This alphabet excludes ambiguous characters such as `I`, `O`, and `L`
/// to reduce confusion in human-readable representations.
pub static BASE_23: Lazy<Alphabet> = Lazy::new(|| Alphabet::new("ABCDEFGHJKLMNPQRSTUVXYZ"));

/// Trait for encoding and decoding numeric identifiers.
pub trait Codec {
    /// Encodes a numeric ID into a string representation.
    ///
    /// # Parameters
    /// - `number`: The `i64` numeric ID to encode.
    ///
    /// # Returns
    /// A `String` containing the encoded representation.
    fn encode(&self, number: i64) -> String;

    /// Decodes a string representation back into a numeric ID.
    ///
    /// # Parameters
    /// - `rep`: The string-encoded representation of an ID.
    ///
    /// # Returns
    /// The decoded `i64` numeric ID.
    fn decode(&self, rep: &str) -> i64;
}

/// A codec that uses a custom alphabet for encoding and decoding.
///
/// `AlphabetCodec` provides a way to represent numeric IDs in a human-friendly
/// manner by using a predefined set of characters.
#[derive(Debug, Clone)]
pub struct AlphabetCodec(Alphabet);

impl Default for AlphabetCodec {
    /// Creates a default `AlphabetCodec` using the `BASE_23` alphabet.
    fn default() -> Self {
        Self::new(BASE_23.clone())
    }
}

impl AlphabetCodec {
    /// Creates a new `AlphabetCodec` with a specified alphabet.
    ///
    /// # Parameters
    /// - `alphabet`: The `Alphabet` to use for encoding and decoding.
    ///
    /// # Returns
    /// A new instance of `AlphabetCodec`.
    pub const fn new(alphabet: Alphabet) -> Self {
        Self(alphabet)
    }
}

/// A structure to track the result and position during decoding.
#[derive(Debug, Default)]
struct ResultWithIndex {
    /// The accumulated numeric result.
    pub result: i64,

    /// The position in the encoded string.
    pub pos: usize,
}

impl ResultWithIndex {
    /// Increments the position and updates the result.
    ///
    /// # Parameters
    /// - `result`: The updated numeric result.
    ///
    /// # Returns
    /// A new `ResultWithIndex` instance with an incremented position.
    pub const fn increment_w_result(self, result: i64) -> Self {
        Self {
            result,
            pos: self.pos + 1,
        }
    }
}

impl Codec for AlphabetCodec {
    fn encode(&self, number: i64) -> String {
        do_encode(&self.0, number, String::default())
    }

    fn decode(&self, rep: &str) -> i64 {
        rep.chars()
            .rev()
            .fold(ResultWithIndex::default(), |acc, c| {
                let encoded_part = self.0.index_of(c) as i64;
                let base_placement = (self.0.base as i64).pow(acc.pos as u32);
                let acc_inc = encoded_part * base_placement;
                let new_acc = acc.result + acc_inc;
                acc.increment_w_result(new_acc)
            })
            .result
    }
}

/// Recursively encodes a numeric value into a string representation.
///
/// # Parameters
/// - `alphabet`: The alphabet to use for encoding.
/// - `number`: The numeric value to encode.
/// - `acc`: The accumulated encoded string.
///
/// # Returns
/// The encoded string representation of the numeric value.
#[tailcall]
fn do_encode(alphabet: &Alphabet, number: i64, mut acc: String) -> String {
    let modulo = (number % alphabet.base as i64) as usize;
    let part = alphabet.value_of(modulo);
    acc.insert(0, part);
    if number < alphabet.base as i64 {
        acc
    } else {
        do_encode(alphabet, number / alphabet.base as i64, acc)
    }
}

/// Defines an alphabet used for encoding and decoding numbers.
///
/// `Alphabet` is a simple wrapper around a string of characters that provides
/// a mapping between numeric values and characters.
#[derive(Debug, Clone)]
pub struct Alphabet {
    /// The set of characters that make up the alphabet.
    pub elements: String,

    /// The base (size) of the alphabet.
    pub base: usize,
}

impl Alphabet {
    /// Creates a new `Alphabet` from a given string of characters.
    ///
    /// # Parameters
    /// - `base`: A string representing the set of characters to use.
    ///
    /// # Returns
    /// A new `Alphabet` instance.
    ///
    /// # Example
    /// ```rust
    /// use tagid::snowflake::pretty::Alphabet;
    ///
    /// let alphabet = Alphabet::new("ABCDEF");
    /// ```
    pub fn new(base: impl Into<String>) -> Self {
        let elements = base.into();
        let base = elements.len();
        Self { elements, base }
    }

    /// Retrieves the character corresponding to a given position in the alphabet.
    ///
    /// # Parameters
    /// - `pos`: The index position within the alphabet.
    ///
    /// # Returns
    /// The character at the specified position.
    ///
    /// # Panics
    /// Panics if the position is out of bounds.
    ///
    /// # Example
    /// ```rust
    /// use tagid::snowflake::pretty::Alphabet;
    ///
    /// let alphabet = Alphabet::new("ABCDEF");
    /// assert_eq!(alphabet.value_of(2), 'C');
    /// ```
    pub fn value_of(&self, pos: usize) -> char {
        self.elements
            .chars()
            .nth(pos)
            .expect("failed on attempted pretty id codec out-of-bounds access.")
    }

    /// Finds the index of a given character in the alphabet.
    ///
    /// # Parameters
    /// - `c`: The character to look up.
    ///
    /// # Returns
    /// The index of the character in the alphabet.
    ///
    /// # Panics
    /// Panics if the character is not found.
    ///
    /// # Example
    /// ```rust
    /// use tagid::snowflake::pretty::Alphabet;
    ///
    /// let alphabet = Alphabet::new("ABCDEF");
    /// assert_eq!(alphabet.index_of('C'), 2);
    /// ```
    pub fn index_of(&self, c: char) -> usize {
        let pos = self.elements.chars().position(|a| a == c);
        pos.expect("failed to pretty id character in alphabet")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::pretty::AlphabetCodec;
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;

    static CODEC: Lazy<AlphabetCodec> = Lazy::new(AlphabetCodec::default);

    #[test]
    fn test_encode_value() {
        assert_eq!(CODEC.encode(23), "BA".to_string());
        assert_eq!(CODEC.encode(529), "BAA".to_string());
        assert_eq!(CODEC.encode(12167), "BAAA".to_string());
    }

    #[test]
    fn test_decode_value() {
        assert_eq!(CODEC.decode("BA"), 23);
        assert_eq!(CODEC.decode("ABA"), 23);
        assert_eq!(CODEC.decode("BAA"), 529);
        assert_eq!(CODEC.decode("BAB"), 530);
        assert_eq!(CODEC.decode("BAAA"), 12167);
        assert_eq!(CODEC.decode("HAPK"), 85477);
        assert_eq!(CODEC.decode("HPJD"), 92233);
    }
}
