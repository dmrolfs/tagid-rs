//! The `prettifier` module transforms long numerical IDs into human-friendly, readable formats.
//! It segments IDs, encodes parts, and incorporates a checksum for validation.
//!
//! # Features
//! - **Encoding:** Uses [`AlphabetCodec`] or other [`Codec`] implementations to encode ID segments.
//! - **Segmentation:** Splits the ID into parts for better readability.
//! - **Checksum Validation:** Ensures integrity using the [Damm algorithm](super::damm).
//! - **Configurable Formatting:** Supports optional leading zeros and custom delimiters.
//!
//! # Example Usage
//! ```rust
//! use tagid::snowflake::pretty::{BASE_23, IdPrettifier};
//!
//! // Initialize a global prettifier instance
//! let prettifier = IdPrettifier::global_initialize(BASE_23.clone());
//!
//! // Convert an ID into its pretty format
//! let pretty_id = prettifier.prettify(824227036833910784);
//! println!("Pretty ID: {}", pretty_id);
//!
//! // Validate and decode a pretty ID back into an integer
//! let decoded_id = prettifier.to_id_seed(&pretty_id).unwrap();
//! assert_eq!(decoded_id, 824227036833910784);
//! ```
//!
//! # Errors
//! - [`ConversionError`] occurs if an ID cannot be properly parsed or validated.
//!
//! # Implementation Details
//! - **Alternating Encoding:** Alternates between raw numeric representation and encoded format.
//! - **Fixed-Length Parts:** Uses a predefined `parts_size` for consistent segmentation.
//! - **Delimiter Separation:** Defaults to `-` but can be customized.
//! - **Thread-Safe Initialization:** Uses [`OnceCell`] for global prettifier instance setup.
//!
//! # Example Pretty ID Format
//! Given `824227036833910784`, the formatted output might be:
//! ```text
//! ARPJ-27036-GVQS-07849
//! ```
//! This improves readability while retaining uniqueness.
//!
//! # Structs & Enums
//! - [`IdPrettifier`]: Handles formatting and validation.
//! - [`ConversionError`]: Represents errors during conversion.

use super::codec::Codec;
use super::damm;
use crate::id::snowflake::pretty::codec::{Alphabet, AlphabetCodec};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use std::str::FromStr;
use thiserror::Error;

/// Represents errors that can occur during ID conversion.
#[derive(Debug, Error)]
pub enum ConversionError {
    /// Error when the provided ID is invalid or cannot be parsed.
    #[error("Not a valid ID: {0}")]
    InvalidId(String),

    /// Error occurring due to integer parsing issues.
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

/// A structure that prettifies numerical IDs into a more human-friendly format.
///
/// This struct supports segmenting and encoding IDs while ensuring validation via a checksum.
/// It can be initialized globally using [`IdPrettifier::global_initialize`].
#[derive(Debug, Clone)]
pub struct IdPrettifier<C: Codec> {
    /// The codec used to encode ID parts.
    pub encoder: C,

    /// The size of each segment when splitting the ID.
    pub parts_size: usize,

    /// The delimiter used between segments (default: `-`).
    pub delimiter: String,

    /// Whether to add leading zeros for a consistent format.
    pub leading_zeros: bool,

    /// The character used for leading zeros.
    pub zero_char: char,

    /// Maximum length of an encoded segment.
    pub max_encoder_length: usize,
}

#[allow(dead_code)]
static PRETTIFIER: OnceCell<IdPrettifier<AlphabetCodec>> = OnceCell::new();

impl IdPrettifier<AlphabetCodec> {
    /// Retrieves the globally initialized prettifier.
    #[inline]
    pub fn summon() -> &'static Self {
        PRETTIFIER
            .get()
            .expect("Alphabetic prettifier is not initialized - initialize via IdPrettifier::<AlphabetCodec>::global_initialize()")
    }

    /// Initializes a global prettifier instance.
    #[allow(dead_code)]
    pub fn global_initialize(alphabet: Alphabet) -> &'static Self {
        PRETTIFIER.get_or_init(|| Self::from_alphabet(alphabet))
    }

    /// Creates a prettifier from an alphabet codec.
    #[allow(dead_code)]
    pub fn from_alphabet(alphabet: Alphabet) -> Self {
        let encoder = AlphabetCodec::new(alphabet);
        let parts_size = 5;
        let zero_char = encoder
            .encode(0)
            .get(0..=0)
            .and_then(|s| s.chars().next())
            .expect("failed to encode id prettifier zero character");
        let max_encoder_length = encoder.encode(10_i64.pow(parts_size as u32) - 1_i64).len();

        Self {
            encoder,
            parts_size,
            delimiter: '-'.to_string(),
            leading_zeros: true,
            zero_char,
            max_encoder_length,
        }
    }
}

impl<C: Codec + Default> Default for IdPrettifier<C> {
    fn default() -> Self {
        let encoder = C::default();
        let parts_size: usize = 5;
        let zero_char = encoder
            .encode(0)
            .get(0..=0)
            .and_then(|s| s.chars().next())
            .expect("failed to encode zero character");
        let max_encoder_length = encoder.encode(10_i64.pow(parts_size as u32) - 1_i64).len();
        Self {
            encoder,
            parts_size,
            delimiter: '-'.to_string(),
            leading_zeros: true,
            zero_char,
            max_encoder_length,
        }
    }
}

impl<C: Codec> IdPrettifier<C> {
    /// Converts an ID into a prettified format.
    pub fn prettify(&self, id_seed: i64) -> String {
        let id_rep = id_seed.to_string();
        let parts = self.divide(damm::encode(id_rep.as_str()));

        let parts_to_convert = if self.leading_zeros {
            self.add_leading_zeros_parts(parts)
        } else {
            parts
        };

        self.convert_parts(parts_to_convert)
    }

    /// Validates if an ID follows the expected format.
    #[allow(dead_code)]
    pub fn is_valid(&self, id: &str) -> bool {
        damm::is_valid(self.decode_seed_with_check_digit(id).as_str())
    }

    /// Decodes a pretty ID back into an integer.
    pub fn to_id_seed(&self, id: &str) -> Result<i64, ConversionError> {
        self.convert_to_id(id)
    }

    /// Splits the numeric representation of the ID into segments.
    ///
    /// This function breaks a numeric string into equally sized parts
    /// based on `parts_size`. The segmentation is done from right to left
    /// to preserve the order of digits while ensuring proper grouping.
    ///
    /// # Arguments
    /// * `rep` - A numeric string representation of an ID with a checksum.
    ///
    /// # Returns
    /// * A vector of string segments, preserving order.
    fn divide(&self, rep: String) -> Vec<String> {
        let mut parts = Vec::with_capacity(rep.len() / self.parts_size + 1);

        for p in &rep.chars().rev().chunks(self.parts_size) {
            #[allow(clippy::needless_collect)]
            let sub_parts: Vec<char> = p.collect();
            let part: String = sub_parts.into_iter().rev().collect();
            parts.push(part);
        }

        parts.into_iter().rev().collect()
    }

    /// Ensures that the ID parts maintain consistent length by padding with leading zeros.
    ///
    /// If `leading_zeros` is enabled, this function adds padding to ensure that each
    /// part of the ID has the expected length, making it visually uniform.
    ///
    /// # Arguments
    /// * `parts` - A vector of string segments representing the ID.
    ///
    /// # Returns
    /// * A vector of properly formatted string segments.
    fn add_leading_zeros_parts(&self, mut parts: Vec<String>) -> Vec<String> {
        let max_parts = (20_f64 / self.parts_size as f64).ceil() as usize;
        parts.reverse();
        parts
            .into_iter()
            .pad_using(max_parts, |_idx| "0".to_string())
            .rev()
            .collect()
    }

    /// Converts the formatted pretty ID back into its original numeric form.
    ///
    /// This function validates and decodes the given formatted ID while checking the
    /// embedded checksum. If the checksum validation fails, an error is returned.
    ///
    /// # Arguments
    /// * `rep` - A pretty-formatted ID string.
    ///
    /// # Returns
    /// * `Ok(i64)` - The decoded numeric ID.
    /// * `Err(ConversionError)` - If the ID fails to parse or validate.
    fn convert_to_id(&self, rep: &str) -> Result<i64, ConversionError> {
        let decoded_with_check_digit = self.decode_seed_with_check_digit(rep);
        if damm::is_valid(&decoded_with_check_digit) {
            decoded_with_check_digit
                .get(..(decoded_with_check_digit.len() - 1))
                .ok_or_else(|| ConversionError::InvalidId(rep.to_string()))
                .and_then(|decoded| i64::from_str(decoded).map_err(|err| err.into()))
        } else {
            Err(ConversionError::InvalidId(rep.to_string()))
        }
    }

    // fn convert_with_leading_zeros<T, F>(&self, item: T, mut for_leading_zeros: F) -> T
    // where
    //     F: FnMut(T) -> T,
    // {
    //     if self.leading_zeros {
    //         for_leading_zeros(item)
    //     } else {
    //         item
    //     }
    // }

    /// Transforms numeric ID segments into a formatted, human-readable string.
    ///
    /// This function processes each segment of an ID, encoding alternating segments
    /// for better readability, and joins them using the configured delimiter.
    ///
    /// # Arguments
    /// * `parts` - A vector of numeric ID segments.
    ///
    /// # Returns
    /// * A prettified ID string with encoded and raw numeric segments.
    fn convert_parts(&self, parts: Vec<String>) -> String {
        let encode_odd = parts.len() % 2 == 0;
        let padded_converted_parts =
            parts
                .into_iter()
                .fold(Vec::<String>::new(), |mut acc, part| {
                    let is_odd = acc.len() % 2 != 0;
                    let direct_part = if encode_odd { is_odd } else { !is_odd }; // acc.len() % 2 != 0;
                    let converted_part = if direct_part {
                        if self.leading_zeros {
                            Self::add_leading_zeros(part, '0', self.parts_size)
                        } else {
                            part
                        }
                    } else {
                        let encoded = self.encoder.encode(
                            i64::from_str(&part).expect("failed to parse part of id into number"),
                        );

                        if self.leading_zeros {
                            Self::add_leading_zeros(
                                encoded,
                                self.zero_char,
                                self.max_encoder_length,
                            )
                        } else {
                            encoded
                        }
                    };
                    acc.push(converted_part);
                    acc
                });

        let formatted = padded_converted_parts.into_iter().join(&self.delimiter);

        formatted.to_string()
    }

    /// Adds leading zero padding to an encoded segment for uniformity.
    ///
    /// Ensures that all parts of the ID maintain the same length by padding with a
    /// specified `zero_char`.
    ///
    /// # Arguments
    /// * `encoded_part` - The string to be padded.
    /// * `zero_char` - The character used for padding.
    /// * `max_part_size` - The required segment length.
    ///
    /// # Returns
    /// * A zero-padded string of the correct length.
    fn add_leading_zeros(
        encoded_part: impl AsRef<str>,
        zero_char: char,
        max_part_size: usize,
    ) -> String {
        let rev_encoded_part: String = encoded_part.as_ref().chars().rev().collect();
        let padded_reversed: String = rev_encoded_part
            .chars()
            .pad_using(max_part_size, |_idx| zero_char)
            .collect();
        let lead_padded: String = padded_reversed.chars().rev().collect();
        lead_padded
    }

    /// Decodes a prettified ID back into its original numeric representation.
    ///
    /// This function processes encoded segments by alternating between decoding and
    /// keeping raw numeric segments, reconstructing the original ID format.
    ///
    /// # Arguments
    /// * `rep` - The prettified ID string to be decoded.
    ///
    /// # Returns
    /// * A string containing the full decoded ID including its checksum.
    fn decode_seed_with_check_digit(&self, rep: impl AsRef<str>) -> String {
        let parts: Vec<&str> = rep.as_ref().split(&self.delimiter).collect();
        let decode_even = parts.len() % 2 != 0;
        let decoded_with_check_digit =
            parts
                .into_iter()
                .fold(Vec::<String>::new(), |mut acc, part| {
                    let is_even = acc.len() % 2 == 0;
                    let decode_part = if decode_even { is_even } else { !is_even };
                    if decode_part {
                        acc.push(part.to_string());
                    } else {
                        let encoded_part = format!("{}", self.encoder.decode(part));
                        let decoded = Self::add_leading_zeros(encoded_part, '0', self.parts_size);
                        acc.push(decoded);
                    }
                    acc
                });

        let formatted = decoded_with_check_digit
            .into_iter()
            .format_with("", |ps, f| f(&ps));
        formatted.to_string()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::id::snowflake::pretty::codec::AlphabetCodec;

    const EXAMPLE_ID: i64 = 824227036833910784;
    const EXAMPLE_REP: &str = "824227036833910784";

    #[test]
    fn test_divide() {
        let prettifier = IdPrettifier::<AlphabetCodec>::default();

        let damm_encoded = damm::encode("100");
        assert_eq!(&damm_encoded, "1007");
        let actual = prettifier.divide(damm_encoded);
        assert_eq!(actual, vec!["1007".to_string()]);

        let damm_encoded = damm::encode(EXAMPLE_REP);
        assert_eq!(damm_encoded, format!("{}9", EXAMPLE_REP));
        let actual = prettifier.divide(damm_encoded);
        assert_eq!(
            actual,
            vec![
                "8242".to_string(),
                "27036".to_string(),
                "83391".to_string(),
                "07849".to_string(),
            ]
        );
    }

    #[test]
    fn test_add_leading_zeros_parts() {
        let prettifier = IdPrettifier::<AlphabetCodec>::default();

        let actual = prettifier.add_leading_zeros_parts(vec!["1007".to_string()]);
        assert_eq!(
            actual,
            vec![
                "0".to_string(),
                "0".to_string(),
                "0".to_string(),
                "1007".to_string(),
            ]
        );

        let actual = prettifier.add_leading_zeros_parts(vec![
            "8242".to_string(),
            "27036".to_string(),
            "83391".to_string(),
            "07849".to_string(),
        ]);
        assert_eq!(
            actual,
            vec![
                "8242".to_string(),
                "27036".to_string(),
                "83391".to_string(),
                "07849".to_string(),
            ]
        );
    }

    #[test]
    fn test_convert_parts() {
        let prettifier = IdPrettifier::<AlphabetCodec>::default();

        let parts = vec!["0", "0", "0", "1007"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let actual = prettifier.convert_parts(parts);
        assert_eq!(actual, "AAAA-00000-AAAA-01007".to_string());

        let parts = vec![
            "8242".to_string(),
            "27036".to_string(),
            "83391".to_string(),
            "07849".to_string(),
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        let actual = prettifier.convert_parts(parts);
        assert_eq!(actual, "ARPJ-27036-GVQS-07849".to_string());
    }

    #[test]
    fn test_generate_pretty_ids_with_leading_zeros() {
        let default = IdPrettifier::<AlphabetCodec>::default();
        println!("### default: {:?}", default);

        let max_pretty_id = default.prettify(i64::MAX);
        assert_eq!(&max_pretty_id, "HPJD-72036-HAPK-58077");

        let example_pretty_id = default.prettify(EXAMPLE_ID);
        assert_eq!(&example_pretty_id, "ARPJ-27036-GVQS-07849");
        assert_eq!(&default.prettify(1), "AAAA-00000-AAAA-00013");

        let prettifier_by_8 = IdPrettifier {
            // encoder: AlphabetCodec::new(Alphabet::new("
            // ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789")),
            parts_size: 8,
            leading_zeros: true,
            ..default
        };
        println!("### prettifier_by_8: {:?}", prettifier_by_8);
        assert_eq!(&prettifier_by_8.prettify(1), "00000000-AAAA-00000013");
        assert_eq!(
            &prettifier_by_8.prettify(i64::MAX),
            "00009223-FTYTHN-47758077"
        );
    }
}
