/// (De-)Serialization of external for this crate types
///
/// For example, Rust standard library and Tendermint types.
use core::{ops::Add, time::Duration};
use std::{string::ToString, vec::Vec};

use borsh::{
    io::{Error as BorshError, ErrorKind, Read, Result, Write},
    BorshDeserialize, BorshSerialize,
};
use tendermint::{Error as TmError, Hash, Time};

pub fn serialize_duration<W: Write>(period: &Duration, writer: &mut W) -> Result<()> {
    period
        .as_secs()
        .serialize(writer)
        .and_then(|()| period.subsec_nanos().serialize(writer))
}

pub fn deserialize_duration<R: Read>(reader: &mut R) -> Result<Duration> {
    BorshDeserialize::deserialize_reader(reader).and_then(|secs| {
        BorshDeserialize::deserialize_reader(reader).map(|nanos| Duration::new(secs, nanos))
    })
}

pub fn serialize_time<W: Write>(time: &Time, writer: &mut W) -> Result<()> {
    time.duration_since(Time::unix_epoch())
        .map_err(invalid_tendermint_data)
        .and_then(|ref since_unix_epoch| serialize_duration(since_unix_epoch, writer))
}

pub fn deserialize_time<R: Read>(reader: &mut R) -> Result<Time> {
    deserialize_duration(reader).and_then(|since_unix_epoch| {
        Time::unix_epoch()
            .add(since_unix_epoch)
            .map_err(invalid_tendermint_data)
    })
}

pub fn serialize_hash<W: Write>(hash: &Hash, writer: &mut W) -> Result<()> {
    Vec::from(*hash).serialize(writer)
}

pub fn deserialize_hash<R: Read>(reader: &mut R) -> Result<Hash> {
    Vec::deserialize_reader(reader).and_then(|raw| raw.try_into().map_err(invalid_tendermint_data))
}

fn invalid_tendermint_data(err: TmError) -> BorshError {
    BorshError::new(ErrorKind::InvalidData, err.to_string())
}
