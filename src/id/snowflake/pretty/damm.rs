//! Damm algorithm-based checksum for numeric identifiers.
//!
//! This module provides a checksum-based validation mechanism for numeric strings
//! using the Damm algorithm. The algorithm detects all single-digit errors and adjacent transpositions,
//! making it suitable for validating IDs without requiring additional arithmetic operations.

use tailcall::tailcall;

/// Encodes a numeric string by appending a Damm checksum digit.
///
/// # Parameters
/// - `rep`: A numeric string to be encoded.
///
/// # Returns
/// A new `String` containing the input followed by its checksum digit.
///
/// # Example
/// ```rust, ignore
/// let encoded = encode("572");
/// assert_eq!(encoded, "5724");
/// ```
pub fn encode(rep: &str) -> String {
    let mut base = rep.to_string();
    base.push_str(checksum(rep).to_string().as_str());
    base
}

/// Decodes a numeric string by removing its checksum digit if valid.
///
/// # Parameters
/// - `rep`: A string containing a numeric identifier with a checksum digit.
///
/// # Returns
/// - `Some(&str)`: The original numeric string without the checksum if valid.
/// - `None`: If the input string fails validation.
///
/// # Example
/// ```rust, ignore
/// assert_eq!(decode("5724"), Some("572"));
/// assert_eq!(decode("5723"), None);
/// ```
#[allow(dead_code)]
pub fn decode(rep: &str) -> Option<&str> {
    if is_valid(rep) {
        rep.get(..(rep.len() - 1))
    } else {
        None
    }
}

/// Validates whether a given numeric string has a correct Damm checksum digit.
///
/// # Parameters
/// - `rep`: A string containing a numeric identifier with a checksum digit.
///
/// # Returns
/// - `true` if the checksum is valid.
/// - `false` otherwise.
///
/// # Example
/// ```rust, ignore
/// assert!(is_valid("5724"));
/// assert!(!is_valid("5723"));
/// ```
pub fn is_valid(rep: &str) -> bool {
    checksum(rep) == 0
}

/// The Damm algorithm substitution table (10×10 quasi-group matrix).
///
/// This matrix is used to compute the checksum by iterating over the digits
/// and using their values as indices in a lookup operation.
const MATRIX: [[usize; 10]; 10] = [
    [0, 3, 1, 7, 5, 9, 8, 6, 4, 2],
    [7, 0, 9, 2, 1, 5, 4, 8, 6, 3],
    [4, 2, 0, 6, 8, 7, 1, 3, 5, 9],
    [1, 7, 5, 0, 9, 8, 3, 4, 2, 6],
    [6, 1, 2, 3, 0, 4, 5, 9, 7, 8],
    [3, 6, 7, 4, 2, 0, 9, 5, 8, 1],
    [5, 8, 6, 9, 7, 2, 0, 1, 3, 4],
    [8, 9, 4, 5, 3, 6, 2, 0, 1, 7],
    [9, 4, 3, 8, 6, 1, 7, 2, 0, 5],
    [2, 5, 8, 1, 4, 3, 6, 7, 9, 0],
];

/// Computes the Damm checksum for a numeric string.
///
/// This function iterates through the input string and updates the checksum
/// using the predefined quasi-group matrix.
///
/// # Parameters
/// - `rep`: A numeric string.
///
/// # Returns
/// - The computed checksum digit (0-9).
///
/// # Example
/// ```rust, ignore
/// assert_eq!(checksum("572"), 4);
/// ```
fn checksum(rep: &str) -> usize {
    do_checksum(rep.as_bytes(), 0, 0)
}

/// Recursive function to compute the checksum using tail recursion.
///
/// # Parameters
/// - `rep`: Byte slice representation of the numeric string.
/// - `interim`: The current checksum value.
/// - `idx`: The index in the byte slice.
///
/// # Returns
/// - The final checksum value after processing all digits.
#[tailcall]
fn do_checksum(rep: &[u8], interim: usize, idx: usize) -> usize {
    if rep.len() <= idx {
        interim
    } else {
        let c = rep[idx] as char;
        let new_interim = if c.is_ascii_digit() {
            MATRIX[interim][c as usize - 48]
        } else {
            interim
        };
        do_checksum(rep, new_interim, idx + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_calculate_check_digit() {
        let actual = encode("572");
        assert_eq!(&actual, "5724");
        let actual = encode("43881234567");
        assert_eq!(&actual, "438812345679");
        let with_checksum = encode(&format!("{}", i64::MAX));
        assert_eq!(is_valid(&with_checksum), true);
    }

    #[test]
    fn test_fail_on_checking_check_digit() {
        let with_checksum = encode(&format!("{}", i64::MAX));

        for i in 0..with_checksum.len() {
            let mut sb_bytes: Vec<u8> = with_checksum.as_bytes().to_vec();
            let old_char = sb_bytes[i];
            let new_char = 47 + ((old_char + 1) % 10);
            // println!(
            //     "old_char:[{}]:[{}] new_char:[{}]:[{}]",
            //     old_char as char, old_char, new_char as char, new_char
            // );
            let item = &mut sb_bytes[i];
            *item = new_char;
            let corrupted = String::from_utf8_lossy(sb_bytes.as_slice());
            // println!("orig:[{}]\ncorr:[{}]\n", with_checksum, corrupted);
            assert_eq!(is_valid(corrupted.as_ref()), false);
        }
    }
}
