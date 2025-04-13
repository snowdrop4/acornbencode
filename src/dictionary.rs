use nom::{
    character::complete::char,
    combinator::map,
    error::VerboseError,
    multi::many0,
    sequence::{delimited, tuple},
    IResult,
};
use std::collections::BTreeMap;

use crate::byte_string::byte_string;
use crate::common::bencode_value;
use crate::common::BencodeValue;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

// Parse a key-value pair (key must be a byte string)
fn dict_pair(input: &[u8]) -> Res<&[u8], (&[u8], BencodeValue)> {
    tuple((byte_string, bencode_value))(input)
}

// Parse a bencode dictionary
pub fn dictionary(input: &[u8]) -> Res<&[u8], BTreeMap<&[u8], BencodeValue>> {
    map(delimited(char('d'), many0(dict_pair), char('e')), |pairs| {
        pairs.into_iter().collect()
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dictionary() {
        let result = dictionary(b"de");
        let expected: BTreeMap<&[u8], BencodeValue> = BTreeMap::new();
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_dictionary_with_integers() {
        let result = dictionary(b"d3:onei1e3:twoi2e5:threei3ee");
        let mut expected = BTreeMap::new();
        expected.insert(b"one".as_slice(), BencodeValue::Integer(1));
        expected.insert(b"two".as_slice(), BencodeValue::Integer(2));
        expected.insert(b"three".as_slice(), BencodeValue::Integer(3));
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_dictionary_with_strings() {
        let result = dictionary(b"d4:spam4:eggs3:bar3:baze");
        let mut expected = BTreeMap::new();
        expected.insert(b"spam".as_slice(), BencodeValue::ByteString(b"eggs"));
        expected.insert(b"bar".as_slice(), BencodeValue::ByteString(b"baz"));
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_mixed_dictionary() {
        let result = dictionary(b"d3:fooi42e3:bar4:spame");
        let mut expected = BTreeMap::new();
        expected.insert(b"foo".as_slice(), BencodeValue::Integer(42));
        expected.insert(b"bar".as_slice(), BencodeValue::ByteString(b"spam"));
        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_nested_dictionary() {
        let result = dictionary(b"d3:food3:bari1e3:bazi2eee");

        let mut inner_dict = BTreeMap::new();
        inner_dict.insert(b"bar".as_slice(), BencodeValue::Integer(1));
        inner_dict.insert(b"baz".as_slice(), BencodeValue::Integer(2));

        let mut expected = BTreeMap::new();
        expected.insert(b"foo".as_slice(), BencodeValue::Dictionary(inner_dict));

        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_dictionary_with_list() {
        let result = dictionary(b"d4:listli1ei2ei3eee");

        let list_values = vec![
            BencodeValue::Integer(1),
            BencodeValue::Integer(2),
            BencodeValue::Integer(3),
        ];

        let mut expected = BTreeMap::new();
        expected.insert(b"list".as_slice(), BencodeValue::List(list_values));

        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_complex_dictionary() {
        let mut result = Vec::new();
        result.push(b'd');
        result.extend_from_slice(b"4:info");
        result.push(b'd');
        result.extend_from_slice(b"4:name"); // info_name_key
        result.extend_from_slice(b"6:sample"); // bytes_info_name_value
        result.extend_from_slice(b"4:size"); // bytes_info_size_key
        result.extend_from_slice(b"i1024e"); //bytes_info_size_value
        result.push(b'e');
        result.extend_from_slice(b"5:files"); // bytes_files_key
        result.extend_from_slice(b"l6:file_1e"); // bytes_files_value
        result.extend_from_slice(b"8:checksum"); // bytes_checksum_key
        result.extend_from_slice(b"i12345e"); // bytes_checksum_value
        result.push(b'e');
        let result = dictionary(&result);

        let mut expected = BTreeMap::new();

        // Create info dict with name and size
        let mut expected_info_dict = BTreeMap::new();
        expected_info_dict.insert(b"name".as_slice(), BencodeValue::ByteString(b"sample"));
        expected_info_dict.insert(b"size".as_slice(), BencodeValue::Integer(1024));
        expected.insert(
            b"info".as_slice(),
            BencodeValue::Dictionary(expected_info_dict),
        );

        // Create files list and checksum
        expected.insert(
            b"files".as_slice(),
            BencodeValue::List(vec![BencodeValue::ByteString(b"file_1")]),
        );
        expected.insert(b"checksum".as_slice(), BencodeValue::Integer(12345));

        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn test_dictionary_with_remaining_input() {
        let result = dictionary(b"d3:fooi42eeextra");

        let mut expected = BTreeMap::new();
        expected.insert(b"foo".as_slice(), BencodeValue::Integer(42));

        assert_eq!(result, Ok((&b"extra"[..], expected)));
    }

    #[test]
    fn test_invalid_dictionaries() {
        // Missing end marker
        assert!(dictionary(b"d3:fooi42e").is_err());

        // Missing start marker
        assert!(dictionary(b"3:fooi42ee").is_err());

        // Invalid key (must be a byte string)
        assert!(dictionary(b"di1ei2ee").is_err());

        // Invalid value
        assert!(dictionary(b"d3:fooXe").is_err());

        // Incomplete key-value pair
        assert!(dictionary(b"d3:fooe").is_err());
    }
}
