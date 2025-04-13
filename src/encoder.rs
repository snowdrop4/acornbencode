use std::collections::BTreeMap;
use std::io::{self, Write};
use std::str;
use std::string::ToString;

use crate::common::BencodeValue;

/// Error type for encoding operations
#[derive(Debug)]
pub enum EncodingError {
    FormatError(std::fmt::Error),
    IoError(io::Error),
    CustomError(String),
    Utf8Error(std::str::Utf8Error),
}

impl From<std::fmt::Error> for EncodingError {
    fn from(error: std::fmt::Error) -> Self {
        EncodingError::FormatError(error)
    }
}

impl From<io::Error> for EncodingError {
    fn from(error: io::Error) -> Self {
        EncodingError::IoError(error)
    }
}

impl From<String> for EncodingError {
    fn from(error: String) -> Self {
        EncodingError::CustomError(error)
    }
}

impl From<std::str::Utf8Error> for EncodingError {
    fn from(error: std::str::Utf8Error) -> Self {
        EncodingError::Utf8Error(error)
    }
}

impl std::fmt::Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingError::FormatError(e) => write!(f, "Format error: {}", e),
            EncodingError::IoError(e) => write!(f, "IO error: {}", e),
            EncodingError::CustomError(e) => write!(f, "Error: {}", e),
            EncodingError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
        }
    }
}

impl std::error::Error for EncodingError {}

/// Trait for types that can be encoded to bencode format
pub trait ToBencode {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError>;
}

/// Encode a BencodeValue to a string
pub fn encode_to_string(value: &BencodeValue) -> Result<String, EncodingError> {
    let bytes = encode_to_bytes(value)?;
    let result = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            // For best effort display, use lossy conversion
            return Ok(String::from_utf8_lossy(e.as_bytes()).into_owned());
        }
    };
    Ok(result)
}

/// Encode a BencodeValue directly to raw bytes
pub fn encode_to_bytes(value: &BencodeValue) -> Result<Vec<u8>, EncodingError> {
    let mut output = Vec::new();
    encode_value(value, &mut output)?;
    Ok(output)
}

/// Write a BencodeValue to a writer
pub fn encode_to_writer<W: Write>(
    value: &BencodeValue,
    writer: &mut W,
) -> Result<(), EncodingError> {
    let bytes = encode_to_bytes(value)?;
    writer.write_all(&bytes)?;
    Ok(())
}

// Internal helper function to encode a value to a byte buffer
fn encode_value(value: &BencodeValue, output: &mut Vec<u8>) -> Result<(), EncodingError> {
    match value {
        BencodeValue::Integer(i) => {
            // Format the integer as a string first
            let int_str = format!("i{}e", i);
            output.extend_from_slice(int_str.as_bytes());
        }
        BencodeValue::ByteString(bytes) => {
            // Format the length as a string first
            let len_str = format!("{}:", bytes.len());
            output.extend_from_slice(len_str.as_bytes());

            // Then add the raw bytes
            output.extend_from_slice(bytes);
        }
        BencodeValue::List(list) => {
            output.push(b'l');
            for item in list {
                encode_value(item, output)?;
            }
            output.push(b'e');
        }
        BencodeValue::Dictionary(dict) => {
            output.push(b'd');
            // BTreeMap guarantees keys are sorted
            for (key, value) in dict {
                // Format the length as a string first
                let len_str = format!("{}:", key.len());
                output.extend_from_slice(len_str.as_bytes());

                // Then add the key bytes
                output.extend_from_slice(key);

                // Then encode the value
                encode_value(value, output)?;
            }
            output.push(b'e');
        }
    }
    Ok(())
}

// Implementations for ToBencode trait

impl ToBencode for BencodeValue<'_> {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(self)
    }
}

impl ToBencode for i64 {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::Integer(*self as isize))
    }
}

impl ToBencode for isize {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::Integer(*self))
    }
}

impl ToBencode for i32 {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::Integer(*self as isize))
    }
}

impl ToBencode for u64 {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        if *self > isize::MAX as u64 {
            return Err(EncodingError::CustomError(
                "Integer too large for bencode encoding".to_string(),
            ));
        }
        encode_to_bytes(&BencodeValue::Integer(*self as isize))
    }
}

impl ToBencode for u32 {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::Integer(*self as isize))
    }
}

impl ToBencode for u16 {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::Integer(*self as isize))
    }
}

impl ToBencode for String {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::ByteString(self.as_bytes()))
    }
}

impl ToBencode for &str {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::ByteString(self.as_bytes()))
    }
}

impl ToBencode for Vec<u8> {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::ByteString(self))
    }
}

impl ToBencode for &[u8] {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        encode_to_bytes(&BencodeValue::ByteString(self))
    }
}

impl<T: ToBencode> ToBencode for Vec<T> {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        let mut output = Vec::new();
        output.push(b'l');
        for item in self {
            let item_bytes = item.to_bencode()?;
            output.extend_from_slice(&item_bytes);
        }
        output.push(b'e');
        Ok(output)
    }
}

impl<K: AsRef<[u8]>, V: ToBencode> ToBencode for BTreeMap<K, V> {
    fn to_bencode(&self) -> Result<Vec<u8>, EncodingError> {
        let mut output = Vec::new();
        output.push(b'd');

        // BTreeMap already keeps keys in sorted order
        for (key, value) in self {
            let key_bytes = key.as_ref();
            let len_str = format!("{}:", key_bytes.len());
            output.extend_from_slice(len_str.as_bytes());
            output.extend_from_slice(key_bytes);

            let value_bytes = value.to_bencode()?;
            output.extend_from_slice(&value_bytes);
        }

        output.push(b'e');
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_encode_integer() {
        assert_eq!(42.to_bencode().unwrap(), b"i42e");
        assert_eq!((-42).to_bencode().unwrap(), b"i-42e");
        assert_eq!(0.to_bencode().unwrap(), b"i0e");
    }

    #[test]
    fn test_encode_string() {
        assert_eq!("spam".to_bencode().unwrap(), b"4:spam");
        assert_eq!("".to_bencode().unwrap(), b"0:");
    }

    #[test]
    fn test_encode_bytes() {
        let binary_data = BencodeValue::ByteString(&[0x00, 0x01, 0x02, 0x03]);
        let expected = b"4:\x00\x01\x02\x03";
        assert_eq!(binary_data.to_bencode().unwrap(), expected);

        // Test with binary data that's not valid UTF-8
        let invalid_utf8 = BencodeValue::ByteString(&[0xC0, 0x7F]);
        let expected = b"2:\xC0\x7F";
        assert_eq!(invalid_utf8.to_bencode().unwrap(), expected);
    }

    #[test]
    fn test_encode_list() {
        let list = vec![1, 2, 3];
        assert_eq!(list.to_bencode().unwrap(), b"li1ei2ei3ee");

        let empty_list: Vec<i32> = vec![];
        assert_eq!(empty_list.to_bencode().unwrap(), b"le");
    }

    #[test]
    fn test_encode_dictionary() {
        let mut dict = BTreeMap::new();
        dict.insert("cow", "moo");
        dict.insert("spam", "eggs");

        // Keys are sorted lexicographically in the output
        assert_eq!(dict.to_bencode().unwrap(), b"d3:cow3:moo4:spam4:eggse");
    }

    #[test]
    fn test_encode_byte_dictionary() {
        let mut dict = BTreeMap::new();
        dict.insert(b"cow".as_slice(), BencodeValue::ByteString(b"moo"));
        dict.insert(b"spam".as_slice(), BencodeValue::ByteString(b"eggs"));

        // Keys are sorted lexicographically in the output
        assert_eq!(dict.to_bencode().unwrap(), b"d3:cow3:moo4:spam4:eggse");

        // Test with binary data that's not valid UTF-8
        let mut binary_dict = BTreeMap::new();
        binary_dict.insert(b"bin".as_slice(), BencodeValue::ByteString(&[0xC0, 0x7F]));

        let encoded = binary_dict.to_bencode().unwrap();
        assert_eq!(encoded, b"d3:bin2:\xC0\x7Fe");
    }

    #[test]
    fn test_encode_nested_structures() {
        let mut inner_dict = BTreeMap::new();
        inner_dict.insert(b"key".as_slice(), BencodeValue::ByteString(b"value"));

        let mut dict = BTreeMap::new();
        dict.insert(b"dict".as_slice(), BencodeValue::Dictionary(inner_dict));
        dict.insert(
            b"list".as_slice(),
            BencodeValue::List(vec![
                BencodeValue::Integer(1),
                BencodeValue::Integer(2),
                BencodeValue::Integer(3),
            ]),
        );

        assert_eq!(
            dict.to_bencode().unwrap(),
            b"d4:dictd3:key5:valuee4:listli1ei2ei3eee"
        );
    }

    #[test]
    fn test_encode_bencode_value() {
        let integer = BencodeValue::Integer(42);
        assert_eq!(integer.to_bencode().unwrap(), b"i42e");

        let string = BencodeValue::ByteString(b"spam");
        assert_eq!(string.to_bencode().unwrap(), b"4:spam");

        let list = BencodeValue::List(vec![
            BencodeValue::Integer(1),
            BencodeValue::Integer(2),
            BencodeValue::Integer(3),
        ]);
        assert_eq!(list.to_bencode().unwrap(), b"li1ei2ei3ee");

        let mut dict_map = BTreeMap::new();
        dict_map.insert(b"key".as_slice(), BencodeValue::ByteString(b"value"));
        let dict = BencodeValue::Dictionary(dict_map);
        assert_eq!(dict.to_bencode().unwrap(), b"d3:key5:valuee");
    }

    #[test]
    fn test_encode_binary_data() {
        // Test with non-UTF8 data as a byte string
        let binary_data = BencodeValue::ByteString(&[0xC0, 0x7F]);
        assert_eq!(binary_data.to_bencode().unwrap(), b"2:\xC0\x7F");

        // Test with a mix of printable and non-printable characters
        let mixed_data = BencodeValue::ByteString(&[b'h', b'i', 0, 1, 2, 3]);
        assert_eq!(mixed_data.to_bencode().unwrap(), b"6:hi\x00\x01\x02\x03");
    }
}
