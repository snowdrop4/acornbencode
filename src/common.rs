use crate::byte_string::byte_string;
use crate::dictionary::dictionary;
use crate::integer::integer;
use crate::list::list;
use nom::{branch::alt, combinator::map, error::VerboseError, IResult};
use std::collections::BTreeMap;

// Define BencodeValue enum to represent different types
#[derive(Debug, PartialEq)]
pub enum BencodeValue<'a> {
    Integer(isize),
    ByteString(&'a [u8]),
    List(Vec<BencodeValue<'a>>),
    Dictionary(BTreeMap<&'a [u8], BencodeValue<'a>>),
}

type Res<T, U> = IResult<T, U, VerboseError<T>>;

// Parse a single bencode value (integer, byte string, list, or dictionary)
pub fn bencode_value(input: &[u8]) -> Res<&[u8], BencodeValue> {
    alt((
        map(integer, BencodeValue::Integer),
        map(byte_string, BencodeValue::ByteString),
        map(list, BencodeValue::List),
        map(dictionary, BencodeValue::Dictionary),
    ))(input)
}
