use std::{string::ToString, vec::Vec};

use borsh::{
    io::{Error as BorshError, ErrorKind, Read, Result, Write},
    BorshDeserialize, BorshSerialize,
};
use ics23::ProofSpec;
use prost::Message;

/// Borsh serialization of [ProofSpec] as its prost representation
pub fn serialize_spec<W: Write>(spec: &ProofSpec, writer: &mut W) -> Result<()> {
    spec.encode_to_vec().serialize(writer)
}

/// Borsh deserialization of [ProofSpec] from its prost representation
pub fn deserialize_spec<R: Read>(reader: &mut R) -> Result<ProofSpec> {
    Vec::deserialize_reader(reader).and_then(|prost_raw| {
        ProofSpec::decode(prost_raw.as_slice())
            .map_err(|prost_err| BorshError::new(ErrorKind::InvalidData, prost_err.to_string()))
    })
}
