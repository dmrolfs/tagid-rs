use super::codec::Codec;
use super::damm;
use crate::id::snowflake::pretty::codec::{Alphabet, AlphabetCodec};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Not a valid ID: {0}")]
    InvalidId(String),

    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

/// It makes Long ids more readable and user friendly, it also adds checksum.
/// Params:
/// encoder – it the result needs to be monotonic, use monotonic Coded e.g. AlphabetCoded with
///     alphabet where char values are monotonic
/// partsSize – the long is chopped on the parts, here you specify the part length (only even parts
///     are encoded with codec)
/// delimiter – sign between parts
/// leadingZeros – prettifier will make id with constant length
#[derive(Debug, Clone)]
pub struct IdPrettifier<C: Codec> {
    pub encoder: C,
    pub parts_size: usize,
    pub delimiter: String,
    pub leading_zeros: bool,
    pub zero_char: char,
    pub max_encoder_length: usize,
}

static PRETTIFIER: OnceCell<IdPrettifier<AlphabetCodec>> = OnceCell::new();

impl IdPrettifier<AlphabetCodec> {
    #[inline]
    pub fn summon() -> &'static Self {
        PRETTIFIER
            .get()
            .expect("Alphabetic prettifier is not initialized - initialize via IdPrettifier::<AlphabetCodec>::global_initialize()")
    }

    #[allow(dead_code)]
    pub fn global_initialize(alphabet: Alphabet) -> &'static Self {
        PRETTIFIER.get_or_init(|| Self::from_alphabet(alphabet))
    }

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
    pub fn prettify(&self, id_seed: i64) -> String {
        let id_rep = id_seed.to_string();
        let parts = self.divide(damm::encode(id_rep.as_str()));
        let parts_to_convert =
            self.convert_with_leading_zeros(parts, |item| self.add_leading_zeros_parts(item));
        self.convert_parts(parts_to_convert)
    }

    #[allow(dead_code)]
    pub fn is_valid(&self, id: &str) -> bool {
        damm::is_valid(self.decode_seed_with_check_digit(id).as_str())
    }

    pub fn to_id_seed(&self, id: &str) -> Result<i64, ConversionError> {
        self.convert_to_id(id)
    }

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

    fn add_leading_zeros_parts(&self, mut parts: Vec<String>) -> Vec<String> {
        let max_parts = (20_f64 / self.parts_size as f64).ceil() as usize;
        parts.reverse();
        parts
            .into_iter()
            .pad_using(max_parts, |_idx| "0".to_string())
            .rev()
            .collect()
    }

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

    fn convert_with_leading_zeros<T, F>(&self, item: T, mut for_leading_zeros: F) -> T
    where
        F: FnMut(T) -> T,
    {
        if self.leading_zeros {
            for_leading_zeros(item)
        } else {
            item
        }
    }

    fn convert_parts(&self, parts: Vec<String>) -> String {
        let encode_odd = parts.len() % 2 == 0;
        let padded_converted_parts =
            parts
                .into_iter()
                .fold(Vec::<String>::new(), |mut acc, part| {
                    let is_odd = acc.len() % 2 != 0;
                    let direct_part = if encode_odd { is_odd } else { !is_odd }; // acc.len() % 2 != 0;
                    let converted_part = if direct_part {
                        self.convert_with_leading_zeros(part, |p| {
                            Self::add_leading_zeros(p, '0', self.parts_size)
                        })
                    } else {
                        let encoded = self.encoder.encode(
                            i64::from_str(&part).expect("failed to parse part of id into number"),
                        );

                        self.convert_with_leading_zeros(encoded, |e| {
                            Self::add_leading_zeros(e, self.zero_char, self.max_encoder_length)
                        })
                    };
                    acc.push(converted_part);
                    acc
                });

        let formatted = padded_converted_parts
            .into_iter()
            .format_with(&self.delimiter, |ps, f| f(&ps));

        formatted.to_string()
    }

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
    const EXAMPLE_REP: &'static str = "824227036833910784";

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
