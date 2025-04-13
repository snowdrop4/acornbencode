use nom::error::VerboseError;
use nom::IResult;
use nom::{character::complete::char, multi::many0, sequence::delimited};

use crate::common::{bencode_value, BencodeValue};

type Res<T, U> = IResult<T, U, VerboseError<T>>;

// Parse a bencode list
pub fn list(input: &[u8]) -> Res<&[u8], Vec<BencodeValue>> {
    delimited(char('l'), many0(bencode_value), char('e'))(input)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_empty_list() {
        assert_eq!(list(b"le"), Ok((&b""[..], vec![])));
    }

    #[test]
    fn test_list_with_integers() {
        let result = list(b"li1ei2ei3ee");
        let expected = vec![
            BencodeValue::Integer(1),
            BencodeValue::Integer(2),
            BencodeValue::Integer(3),
        ];
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_list_with_strings() {
        let result = list(b"l4:spam4:eggse");
        let expected = vec![
            BencodeValue::ByteString(b"spam"),
            BencodeValue::ByteString(b"eggs"),
        ];
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_mixed_list() {
        let result = list(b"l4:spami42ee");
        let expected = vec![BencodeValue::ByteString(b"spam"), BencodeValue::Integer(42)];
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_nested_lists() {
        let result = list(b"ll4:spami42eeli1ei2eee");
        let expected = vec![
            BencodeValue::List(vec![
                BencodeValue::ByteString(b"spam"),
                BencodeValue::Integer(42),
            ]),
            BencodeValue::List(vec![BencodeValue::Integer(1), BencodeValue::Integer(2)]),
        ];
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_list_with_dictionary() {
        let result = list(b"ld3:fooi42e3:bar4:spamee");

        let mut dict = BTreeMap::new();
        dict.insert(b"foo".as_slice(), BencodeValue::Integer(42));
        dict.insert(b"bar".as_slice(), BencodeValue::ByteString(b"spam"));

        let expected = vec![BencodeValue::Dictionary(dict)];
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_complex_list() {
        // Create a complex list with mixed types: string, int, list, dictionary
        let mut result = Vec::new();
        result.push(b'l');
        result.extend_from_slice(b"4:spam"); // bytes_spam
        result.extend_from_slice(b"i42e"); // bytes_42
        result.push(b'l');
        result.extend_from_slice(b"3:abc"); // bytes_abc
        result.extend_from_slice(b"3:xyz"); // bytes_xyz
        result.push(b'e');
        result.push(b'd');
        result.extend_from_slice(b"4:list"); // bytes_list
        result.push(b'l');
        result.extend_from_slice(b"i1e"); // bytes_1
        result.extend_from_slice(b"i2e"); // bytes_2
        result.push(b'e');
        result.push(b'e');
        result.push(b'e');
        let result = list(&result);

        let mut expected_inner_dict = BTreeMap::new();
        expected_inner_dict.insert(
            b"list".as_slice(),
            BencodeValue::List(vec![BencodeValue::Integer(1), BencodeValue::Integer(2)]),
        );

        let expected = vec![
            BencodeValue::ByteString(b"spam"),
            BencodeValue::Integer(42),
            BencodeValue::List(vec![
                BencodeValue::ByteString(b"abc"),
                BencodeValue::ByteString(b"xyz"),
            ]),
            BencodeValue::Dictionary(expected_inner_dict),
        ];

        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_lists_with_remaining_input() {
        let result = list(b"li1ei2eeextra");
        let expected = vec![BencodeValue::Integer(1), BencodeValue::Integer(2)];
        assert_eq!(result, Ok((&b"extra"[..], expected)));
    }

    #[test]
    fn test_invalid_lists() {
        // Missing end marker
        assert!(list(b"li1ei2e").is_err());

        // Missing start marker
        assert!(list(b"i1ei2ee").is_err());

        // Invalid content
        assert!(list(b"li1eXi2ee").is_err());
    }
}
