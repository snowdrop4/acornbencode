use nom::{branch::alt, combinator::map, error::VerboseError, IResult};

use crate::byte_string::byte_string;
use crate::common::BencodeValue;
use crate::dictionary::dictionary;
use crate::integer::integer;
use crate::list::list;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

/// Parse any bencode value (integer, byte string, list, or dictionary)
pub fn parse_bencode(input: &[u8]) -> Res<&[u8], BencodeValue> {
    alt((
        map(integer, BencodeValue::Integer),
        map(byte_string, BencodeValue::ByteString),
        map(list, BencodeValue::List),
        map(dictionary, BencodeValue::Dictionary),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_parse_integer() {
        assert_eq!(
            parse_bencode(b"i42e"),
            Ok((&b""[..], BencodeValue::Integer(42)))
        );
    }

    #[test]
    fn test_parse_byte_string() {
        assert_eq!(
            parse_bencode(b"4:spam"),
            Ok((&b""[..], BencodeValue::ByteString(b"spam")))
        );
    }

    #[test]
    fn test_parse_list() {
        let result = parse_bencode(b"li1ei2ei3ee");
        let expected = BencodeValue::List(vec![
            BencodeValue::Integer(1),
            BencodeValue::Integer(2),
            BencodeValue::Integer(3),
        ]);
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_parse_dictionary() {
        let result = parse_bencode(b"d3:fooi42e3:bar4:spame");

        let mut expected_dict = BTreeMap::new();
        expected_dict.insert(b"foo".as_slice(), BencodeValue::Integer(42));
        expected_dict.insert(b"bar".as_slice(), BencodeValue::ByteString(b"spam"));

        let expected = BencodeValue::Dictionary(expected_dict);
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_complex_nested_structure() {
        let mut result = Vec::new();
        result.push(b'd');
        result.extend_from_slice(b"4:info"); // bytes_info_key
        result.extend_from_slice(b"4:name"); // bytes_info_value
        result.extend_from_slice(b"4:list"); // bytes_list_key
        result.extend_from_slice(b"li1ei2ei3ee"); // bytes_list_value
        result.push(b'e');
        let result = parse_bencode(&result);

        let mut expected = BTreeMap::new();

        expected.insert(b"info".as_slice(), BencodeValue::ByteString(b"name"));
        expected.insert(
            b"list".as_slice(),
            BencodeValue::List(vec![
                BencodeValue::Integer(1),
                BencodeValue::Integer(2),
                BencodeValue::Integer(3),
            ]),
        );

        let expected = BencodeValue::Dictionary(expected);

        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_binary_data_parsing() {
        // Test binary data in a byte string
        let binary_data1 = &[b'4', b':', 0x00, 0x01, 0x02, 0x03];
        assert_eq!(
            parse_bencode(binary_data1),
            Ok((
                &b""[..],
                BencodeValue::ByteString(&[0x00, 0x01, 0x02, 0x03])
            ))
        );

        // Test binary data with null bytes
        let binary_data2 = &[b'4', b':', b'a', 0, b'b', 0];
        assert_eq!(
            parse_bencode(binary_data2),
            Ok((&b""[..], BencodeValue::ByteString(&[b'a', 0, b'b', 0])))
        );

        // Test binary data in a list
        let mut list_data = Vec::new();
        list_data.extend_from_slice(b"l4:");
        list_data.extend_from_slice(&[0x00, 0x01, 0x02, 0x03]);
        list_data.extend_from_slice(b"4:a");
        list_data.push(0);
        list_data.extend_from_slice(b"b");
        list_data.push(0);
        list_data.push(b'e');

        let expected = BencodeValue::List(vec![
            BencodeValue::ByteString(&[0x00, 0x01, 0x02, 0x03]),
            BencodeValue::ByteString(&[b'a', 0, b'b', 0]),
        ]);

        assert_eq!(parse_bencode(&list_data), Ok((&b""[..], expected)));

        // Test binary data in a dictionary
        let mut dict_data = Vec::new();
        dict_data.extend_from_slice(b"d3:bin4:");
        dict_data.extend_from_slice(&[0x00, 0x01, 0x02, 0x03]);
        dict_data.extend_from_slice(b"3:key5:value3:mix4:a");
        dict_data.push(0);
        dict_data.extend_from_slice(b"b");
        dict_data.push(0);
        dict_data.push(b'e');

        let mut expected_dict = BTreeMap::new();
        expected_dict.insert(
            b"bin".as_slice(),
            BencodeValue::ByteString(&[0x00, 0x01, 0x02, 0x03]),
        );
        expected_dict.insert(b"key".as_slice(), BencodeValue::ByteString(b"value"));
        expected_dict.insert(
            b"mix".as_slice(),
            BencodeValue::ByteString(&[b'a', 0, b'b', 0]),
        );

        let expected = BencodeValue::Dictionary(expected_dict);
        assert_eq!(parse_bencode(&dict_data), Ok((&b""[..], expected)));
    }

    #[test]
    fn test_utf8_conversion() {
        // Test valid UTF-8 data
        let result = parse_bencode(b"4:spam").unwrap().1;
        if let BencodeValue::ByteString(bytes) = result {
            assert_eq!(std::str::from_utf8(bytes).ok(), Some("spam"));
        } else {
            panic!("Expected ByteString");
        }

        // Test invalid UTF-8 data
        let invalid_utf8 = &[b'2', b':', 0xC0, 0x7F];
        let result = parse_bencode(invalid_utf8).unwrap().1;
        if let BencodeValue::ByteString(bytes) = result {
            assert!(std::str::from_utf8(bytes).is_err());
        } else {
            panic!("Expected ByteString");
        }
    }
}
