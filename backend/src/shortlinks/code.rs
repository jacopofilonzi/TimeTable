//! Short link codes: Crockford base32 of a SHA-256 hash of the link's canonical query.
//!
//! Crockford base32 (`0-9`, `A-Z` without `I L O U`) keeps codes case-insensitive, so a code
//! written in uppercase fits the QR alphanumeric mode, and look-alike characters typed by hand
//! (`o`, `i`, `l`) are mapped back to the digits they resemble.

use sha2::{Digest, Sha256};

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Length of a code; on a hash collision the code grows one character at a time up to
/// [`MAX_LEN`].
pub const MIN_LEN: usize = 8;
pub const MAX_LEN: usize = 16;

/// SHA-256 of the canonical query: the same settings always hash to the same code.
pub fn hash(canonical: &str) -> [u8; 32] {
    Sha256::digest(canonical.as_bytes()).into()
}

/// The first `len` base32 digits (5 bits each) of `hash`.
pub fn encode(hash: &[u8; 32], len: usize) -> String {
    (0..len)
        .map(|i| {
            let index = (0..5).fold(0usize, |acc, b| {
                let bit = i * 5 + b;
                (acc << 1) | usize::from(hash[bit / 8] >> (7 - bit % 8) & 1)
            });
            char::from(ALPHABET[index])
        })
        .collect()
}

/// Canonical (uppercase) form of a code typed by a user, or `None` if it can't be one.
pub fn normalize(input: &str) -> Option<String> {
    if !(MIN_LEN..=MAX_LEN).contains(&input.len()) {
        return None;
    }
    input
        .chars()
        .map(|c| match c.to_ascii_uppercase() {
            'O' => Some('0'),
            'I' | 'L' => Some('1'),
            c if c.is_ascii() && ALPHABET.contains(&(c as u8)) => Some(c),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_code() {
        let a = encode(&hash("uni=unicam&weeks=4"), MIN_LEN);
        let b = encode(&hash("uni=unicam&weeks=4"), MIN_LEN);
        let c = encode(&hash("uni=unicam&weeks=2"), MIN_LEN);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), MIN_LEN);
    }

    #[test]
    fn longer_codes_extend_shorter_ones() {
        let h = hash("x");
        let short = encode(&h, MIN_LEN);
        let long = encode(&h, MIN_LEN + 1);
        assert!(long.starts_with(&short));
    }

    #[test]
    fn encodes_bits_in_order() {
        let mut h = [0u8; 32];
        h[0] = 0b0000_0100; // first digit 00000 = '0', second 10000 = 16 = 'G'
        assert_eq!(&encode(&h, 2), "0G");
        assert_eq!(encode(&[0xFF; 32], MAX_LEN), "Z".repeat(MAX_LEN));
    }

    #[test]
    fn normalizes_user_input() {
        assert_eq!(normalize("7k2qm9xa").as_deref(), Some("7K2QM9XA"));
        assert_eq!(normalize("oil12345").as_deref(), Some("01112345"));
        assert_eq!(normalize("UUUUUUUU"), None);
        assert_eq!(normalize("ABC"), None);
        assert_eq!(normalize("ABCDEFG!"), None);
        assert_eq!(normalize("ÀBCDEFGH"), None);
    }
}
