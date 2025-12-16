use core::time::Duration;

use borsh::{
    io::{Read, Result, Write},
    BorshDeserialize, BorshSerialize,
};

pub fn serialize_duration<W: Write>(number: &Duration, writer: &mut W) -> Result<()> {
    number
        .as_secs()
        .serialize(writer)
        .and_then(|()| number.subsec_nanos().serialize(writer))
}

pub fn deserialize_duration<R: Read>(reader: &mut R) -> Result<Duration> {
    BorshDeserialize::deserialize_reader(reader).and_then(|secs: u64| {
        BorshDeserialize::deserialize_reader(reader).map(|nanos: u32| Duration::new(secs, nanos))
    })
}
