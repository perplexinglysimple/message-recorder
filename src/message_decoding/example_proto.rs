use prost::Message;

pub mod example {
    include!(concat!(env!("OUT_DIR"), "/example.rs"));
}

pub fn deserialize_address_book_proto(
    data: &Vec<u8>,
) -> Result<example::AddressBook, prost::DecodeError> {
    example::AddressBook::decode(data.as_slice())
}
