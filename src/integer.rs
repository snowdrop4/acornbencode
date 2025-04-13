use nom::{
    character::complete::{char, digit1},
    combinator::{map_res, opt},
    error::VerboseError,
    sequence::{delimited, tuple},
    IResult,
};
use std::str;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn from_integer_digits((sign, digits): (Option<char>, &[u8])) -> Result<isize, &'static str> {
    // Convert bytes to string for parsing
    let digits_str = match str::from_utf8(digits) {
        Ok(s) => s,
        Err(_) => return Err("Invalid UTF-8 in integer digits"),
    };

    // Check for leading zeros (any number that starts with 0 but is not just 0)
    if digits_str.len() > 1 && digits_str.starts_with('0') {
        return Err("No leading zeros");
    }

    // Check for negative zero
    if sign.is_some() && digits_str == "0" {
        return Err("No negative zero");
    }

    match digits_str.parse::<isize>() {
        Ok(x) => match sign {
            Some(_) => Ok(-x),
            None => Ok(x),
        },
        _ => Err("Invalid integer"),
    }
}

fn integer_digits(input: &[u8]) -> Res<&[u8], isize> {
    let sign = opt(char('-'));
    let digits = digit1;

    map_res(tuple((sign, digits)), from_integer_digits)(input)
}

pub fn integer(input: &[u8]) -> Res<&[u8], isize> {
    delimited(char('i'), integer_digits, char('e'))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_integers() {
        assert_eq!(integer(b"i42e"), Ok((&b""[..], 42)));
        assert_eq!(integer(b"i0e"), Ok((&b""[..], 0)));
        assert_eq!(integer(b"i123456789e"), Ok((&b""[..], 123456789)));
        assert_eq!(integer(b"i-42e"), Ok((&b""[..], -42)));
        assert_eq!(integer(b"i-1e"), Ok((&b""[..], -1)));
    }

    #[test]
    fn test_integers_with_remaining_input() {
        assert_eq!(integer(b"i42eextra"), Ok((&b"extra"[..], 42)));
        assert_eq!(integer(b"i-10edata"), Ok((&b"data"[..], -10)));
    }

    #[test]
    fn test_invalid_integers() {
        // Invalid format (missing end marker)
        assert!(integer(b"i42").is_err());

        // Invalid format (missing start marker)
        assert!(integer(b"42e").is_err());

        // Invalid format (missing both markers)
        assert!(integer(b"42").is_err());

        // Leading zeros not allowed
        assert!(integer(b"i01e").is_err());
        assert!(integer(b"i00123e").is_err());

        // Negative zero not allowed
        assert!(integer(b"i-0e").is_err());

        // Non-digit characters
        assert!(integer(b"iabce").is_err());
        assert!(integer(b"i1a2e").is_err());
    }
}
