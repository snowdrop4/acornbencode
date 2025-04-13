use nom::{
    bytes::complete::take,
    character::complete::{char, digit1},
    combinator::map_res,
    error::VerboseError,
    sequence::terminated,
    IResult,
};
use std::str;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn parse_length(input: &[u8]) -> Res<&[u8], usize> {
    // Parse the length part of the bytestring (digits followed by :)
    map_res(terminated(digit1, char(':')), |digits: &[u8]| {
        // Convert bytes to string for parsing
        let s = str::from_utf8(digits).map_err(|_| "Invalid UTF-8 in length")?;

        // Don't allow leading zeros unless the length is 0
        if s.len() > 1 && s.starts_with('0') {
            return Err("No leading zeros in length");
        }
        s.parse::<usize>().map_err(|_| "Invalid length")
    })(input)
}

/// Parse a byte string from bencode format - returns a slice of the original bytes
pub fn byte_string(input: &[u8]) -> Res<&[u8], &[u8]> {
    // First parse the length, then take that many bytes
    let (remaining, length) = parse_length(input)?;

    // Take exactly 'length' bytes from the input
    take(length)(remaining)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_valid_byte_strings() {
        assert_eq!(byte_string(b"0:"),               Ok((&b""[..], &b""[..])));
        assert_eq!(byte_string(b"12:hello world!"),  Ok((&b""[..], &b"hello world!"[..])));
        assert_eq!(byte_string(b"12:hello\nworld!"), Ok((&b""[..], &b"hello\nworld!"[..])));
        assert_eq!(byte_string("21:ハローワールド".as_bytes()), Ok((&b""[..], "ハローワールド".as_bytes())));
    }

    #[test]
    #[rustfmt::skip]
    fn test_byte_strings_with_remaining_input() {
        assert_eq!(byte_string(b"4:spamextra"), Ok((&b"extra"[..], &b"spam"[..])));
        assert_eq!(byte_string(b"3:abcdefg"),   Ok((&b"defg"[..], &b"abc"[..])));
    }

    #[test]
    fn test_invalid_byte_strings() {
        // Invalid format (no colon)
        assert!(byte_string(b"4spam").is_err());

        // Invalid format (incorrect length)
        assert!(byte_string(b"10:hello").is_err());

        // Leading zeros in length not allowed
        assert!(byte_string(b"04:spam").is_err());
        assert!(byte_string(b"001:x").is_err());

        // Empty input
        assert!(byte_string(b"").is_err());

        // Just a number
        assert!(byte_string(b"42").is_err());
    }

    #[test]
    fn test_binary_data() {
        // Test with non-UTF8 data - ASCII control characters
        assert_eq!(
            byte_string(b"3:\x01\x02\x03"),
            Ok((&b""[..], &b"\x01\x02\x03"[..]))
        );

        // Test with null bytes
        assert_eq!(byte_string(b"4:a\0b\0"), Ok((&b""[..], &b"a\0b\0"[..])));

        // Test with a mix of printable and non-printable characters
        assert_eq!(
            byte_string(b"6:a\x7Fb\x1Fc\x0A"),
            Ok((&b""[..], &b"a\x7Fb\x1Fc\x0A"[..]))
        );

        // Test with binary data followed by more input
        assert_eq!(
            byte_string(b"4:\x00\x01\x02\x03extra"),
            Ok((&b"extra"[..], &b"\x00\x01\x02\x03"[..]))
        );

        // Test with full binary range (0-255)
        let mut input = Vec::from(b"256:".as_ref());
        let data: Vec<u8> = (0..=255).collect();
        input.extend_from_slice(&data);

        let expected_remaining = Vec::new();
        let expected_output = data;

        assert_eq!(
            byte_string(&input),
            Ok((&expected_remaining[..], &expected_output[..]))
        );
    }
}
