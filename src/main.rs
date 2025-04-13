use acornbencode::common::BencodeValue;
use acornbencode::encoder;
use acornbencode::parser::parse_bencode;
use std::collections::BTreeMap;

fn main() {
    // -------------------------------------------------------------------------
    // Example: parsing
    // -------------------------------------------------------------------------
    let input = b"d3:fooi42e3:bar4:spame";
    println!("Parsing: {}", String::from_utf8_lossy(input));
    match parse_bencode(input) {
        Ok((_, value)) => println!("Parsed value: {:?}", value),
        Err(e) => println!("Error parsing: {:?}", e),
    }

    // -------------------------------------------------------------------------
    // Example: encoding
    // -------------------------------------------------------------------------
    let integer = BencodeValue::Integer(42);
    match encoder::encode_to_string(&integer) {
        Ok(encoded) => println!("Encoded integer: {}", encoded),
        Err(e) => println!("Error encoding: {:?}", e),
    }

    let bytestring = BencodeValue::ByteString(b"spam");
    match encoder::encode_to_string(&bytestring) {
        Ok(encoded) => println!("Encoded string: {}", encoded),
        Err(e) => println!("Error encoding: {:?}", e),
    }

    let list = BencodeValue::List(vec![
        BencodeValue::Integer(1),
        BencodeValue::Integer(2),
        BencodeValue::Integer(3),
    ]);
    match encoder::encode_to_string(&list) {
        Ok(encoded) => println!("Encoded list: {}", encoded),
        Err(e) => println!("Error encoding: {:?}", e),
    }

    let mut dict = BTreeMap::new();
    dict.insert(b"key".as_slice(), BencodeValue::ByteString(b"value"));
    dict.insert(b"num".as_slice(), BencodeValue::Integer(42));
    let dictionary = BencodeValue::Dictionary(dict);
    match encoder::encode_to_string(&dictionary) {
        Ok(encoded) => println!("Encoded dictionary: {}", encoded),
        Err(e) => println!("Error encoding: {:?}", e),
    }

    // -------------------------------------------------------------------------
    // Example: binary data
    // -------------------------------------------------------------------------
    let binary_data = vec![0xC0, 0x7F]; // Invalid UTF-8 bytes
    let binary_value = BencodeValue::ByteString(&binary_data);
    match encoder::encode_to_bytes(&binary_value) {
        Ok(encoded) => println!("Encoded binary data length: {}", encoded.len()),
        Err(e) => println!("Error encoding: {:?}", e),
    }
}
