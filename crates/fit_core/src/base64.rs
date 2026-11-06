//! Minimal base64 (RFC 4648) used for URL-fragment sharing.
//!
//! Encoding uses the URL-safe alphabet without padding. Decoding accepts both the URL-safe and
//! the standard alphabet, with or without `=` padding, and ignores ASCII whitespace.

const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode bytes as URL-safe base64 without padding.
pub fn encode_url(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let chars = chunk.len() + 1;
        for i in 0..chars {
            let idx = (n >> (18 - 6 * i)) & 0x3f;
            out.push(URL_SAFE[idx as usize] as char);
        }
    }
    out
}

fn value(c: u8) -> Option<u32> {
    Some(match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'-' | b'+' => 62,
        b'_' | b'/' => 63,
        _ => return None,
    } as u32)
}

/// Decode base64 (URL-safe or standard, padding optional). Returns `None` on invalid input.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let clean: Vec<u8> = text.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
    let trimmed = {
        let mut end = clean.len();
        while end > 0 && clean[end - 1] == b'=' {
            end -= 1;
        }
        &clean[..end]
    };
    if trimmed.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(trimmed.len() * 3 / 4);
    for chunk in trimmed.chunks(4) {
        let mut n = 0u32;
        for (i, &c) in chunk.iter().enumerate() {
            n |= value(c)? << (18 - 6 * i);
        }
        let bytes = chunk.len() - 1;
        for i in 0..bytes {
            out.push((n >> (16 - 8 * i)) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc4648_vectors() {
        let cases = [
            ("", ""),
            ("f", "Zg"),
            ("fo", "Zm8"),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg"),
            ("fooba", "Zm9vYmE"),
            ("foobar", "Zm9vYmFy"),
        ];
        for (plain, enc) in cases {
            assert_eq!(encode_url(plain.as_bytes()), enc);
            assert_eq!(decode(enc).unwrap(), plain.as_bytes());
        }
        assert_eq!(decode("Zm9vYg==").unwrap(), b"foob");
    }

    #[test]
    fn round_trip_all_bytes_and_alphabets() {
        let data: Vec<u8> = (0..=255u8).collect();
        let enc = encode_url(&data);
        assert!(!enc.contains('+') && !enc.contains('/') && !enc.contains('='));
        assert_eq!(decode(&enc).unwrap(), data);
        let std = enc.replace('-', "+").replace('_', "/");
        assert_eq!(decode(&std).unwrap(), data);
        assert!(decode("a").is_none());
        assert!(decode("ab$d").is_none());
    }
}
