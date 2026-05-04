//! Types for the IBC events emitted from Tendermint Websocket by the client module.

#![expect(deprecated)]

use core::fmt::{self, Display, Formatter};

use data_types_macros::{define_attribute, define_event};
use ibc_core_host_types::{
    error::DecodingError,
    identifiers::{ClientId, ClientType},
};
use ibc_primitives::prelude::*;
use subtle_encoding::hex;

use crate::height::Height;

define_attribute!(
    "client_id" => ClientIdAttribute(ClientId) {
        friendly_name = "client ID",
        into = String::from,
        parse = |client_id: &str| client_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "client_type" => ClientTypeAttribute(ClientType) {
        friendly_name = "client type",
        into = String::from,
        parse = |client_type: &str| client_type.parse().map_err(Into::into),
    }
);

define_attribute!(
    "consensus_height" => ConsensusHeightAttribute(Height) {
        friendly_name = "consensus height",
        into = String::from,
        parse = str::parse,
    }
);

define_attribute!(
    "consensus_heights" => ConsensusHeightsAttribute(Vec<Height>) {
        friendly_name = "consensus heights",
        into = |heights: Vec<Height>| DisplayList(&heights).to_string(),
        parse = |heights: &str| heights.split(',').map(Height::from_str).collect::<Result<_, _>>(),
    }
);

define_attribute!(
    "header" => HeaderAttribute(Vec<u8>) {
        friendly_name = "header",
        into = |header: Vec<u8>| str::from_utf8(&hex::encode(header))
            .expect("never fails because hexadecimal is valid UTF-8")
            .to_string(),
        parse = |header| hex::decode(header).map_err(|e| {
            DecodingError::invalid_raw_data(format!("header attribute value: {e}"))
        }),
    }
);

// CreateClient event signals the creation of a new on-chain client (IBC client).
define_event!(
    "create_client" => CreateClient {
        client_id: ClientIdAttribute,
        client_type: ClientTypeAttribute,
        consensus_height: ConsensusHeightAttribute,
    }
);

// UpdateClient event signals a recent update of an on-chain client (IBC Client).
define_event!(
    "update_client" => UpdateClient {
        client_id: ClientIdAttribute,
        client_type: ClientTypeAttribute,
        #[deprecated = "Will be removed in a future release. Use `consensus_heights` instead."]
        consensus_height: ConsensusHeightAttribute,
        consensus_heights: ConsensusHeightsAttribute,
        header: HeaderAttribute,
    }
);

// ClientMisbehaviour event signals the update of an on-chain client (IBC Client) with evidence of
// misbehaviour.
define_event!(
    "client_misbehaviour" => ClientMisbehaviour {
        client_id: ClientIdAttribute,
        client_type: ClientTypeAttribute,
    }
);

// Signals a recent upgrade of an on-chain client (IBC Client).
define_event!(
    "upgrade_client" => UpgradeClient {
        client_id: ClientIdAttribute,
        client_type: ClientTypeAttribute,
        consensus_height: ConsensusHeightAttribute,
    }
);

struct DisplayList<'r, T>(&'r [T])
where
    T: Display;

impl<T> Display for DisplayList<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let [head, tail @ ..] = self.0 {
            head.fmt(f)?;

            for element in tail {
                f.write_str(",")?;

                element.fmt(f)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::{any::Any, str::FromStr as _};

    use rstest::*;
    use tendermint::abci::{Event, EventAttribute};

    use super::*;

    #[rstest]
    #[case(
        Event {
            kind: CreateClient::EVENT_KIND.to_owned(),
            attributes: vec![
                EventAttribute::from(("client_id", "07-tendermint-0")),
                EventAttribute::from(("client_type", "07-tendermint")),
                EventAttribute::from(("consensus_height", "1-10")),
            ],
        },
        Ok(CreateClient {
            client_id: ClientId::from_str("07-tendermint-0").expect("should parse").into(),
            client_type: ClientType::from_str("07-tendermint").expect("should parse").into(),
            consensus_height: Height::new(1, 10).unwrap().into(),
        }),
    )]
    #[case(
        Event {
            kind: "some_other_event".to_owned(),
            attributes: vec![
                EventAttribute::from(("client_id", "07-tendermint-0")),
                EventAttribute::from(("client_type", "07-tendermint")),
                EventAttribute::from(("consensus_height", "1-10")),
            ],
        },
        Err(DecodingError::MismatchedResourceName {
            expected: CreateClient::EVENT_KIND.to_string(),
            actual: "some_other_event".to_string(),
        })
    )]
    #[case(
        Event {
            kind: CreateClient::EVENT_KIND.to_owned(),
            attributes: vec![
                EventAttribute::from(("client_type", "07-tendermint")),
                EventAttribute::from(("consensus_height", "1-10")),
            ],
        },
        Err(DecodingError::missing_raw_data("attribute key")),
    )]
    fn test_create_client_try_from(
        #[case] event: Event,
        #[case] expected: Result<CreateClient, DecodingError>,
    ) {
        let result = CreateClient::try_from(event);
        if expected.is_err() {
            assert_eq!(
                result.unwrap_err().type_id(),
                expected.unwrap_err().type_id()
            );
        } else {
            assert_eq!(result.unwrap(), expected.unwrap());
        }
    }

    #[rstest]
    #[case(
        Event {
            kind: UpdateClient::EVENT_KIND.to_owned(),
            attributes: vec![
                EventAttribute::from(("client_id", "07-tendermint-0")),
                EventAttribute::from(("client_type", "07-tendermint")),
                EventAttribute::from(("consensus_height", "1-10")),
                EventAttribute::from(("consensus_heights", "1-10,1-11")),
                EventAttribute::from(("header", "1234")),
            ],
        },
        Ok(UpdateClient {
            client_id: ClientId::from_str("07-tendermint-0").expect("should parse").into(),
            client_type: ClientType::from_str("07-tendermint").expect("should parse").into(),
            consensus_height: Height::new(1, 10).unwrap().into(),
            consensus_heights: vec![Height::new(1, 10).unwrap(), Height::new(1, 11).unwrap()].into(),
            header: vec![0x12, 0x34].into(),
        }),
    )]
    #[case(
        Event {
            kind: "some_other_event".to_owned(),
            attributes: vec![
                EventAttribute::from(("client_id", "07-tendermint-0")),
                EventAttribute::from(("client_type", "07-tendermint")),
                EventAttribute::from(("consensus_height", "1-10")),
                EventAttribute::from(("consensus_heights", "1-10,1-11")),
                EventAttribute::from(("header", "1234")),
            ],
        },
        Err(DecodingError::MismatchedResourceName {
            expected: UpdateClient::EVENT_KIND.to_string(),
            actual: "some_other_event".to_owned(),
        }),
    )]
    #[case(
        Event {
            kind: UpdateClient::EVENT_KIND.to_owned(),
            attributes: vec![
                EventAttribute::from(("client_type", "07-tendermint")),
                EventAttribute::from(("consensus_height", "1-10")),
                EventAttribute::from(("consensus_heights", "1-10,1-11")),
                EventAttribute::from(("header", "1234")),
            ],
        },
        Err(DecodingError::missing_raw_data("attribute key")),
    )]
    fn test_update_client_try_from(
        #[case] event: Event,
        #[case] expected: Result<UpdateClient, DecodingError>,
    ) {
        let result = UpdateClient::try_from(event);
        if expected.is_err() {
            assert_eq!(
                result.unwrap_err().type_id(),
                expected.unwrap_err().type_id()
            );
        } else {
            assert_eq!(result.unwrap(), expected.unwrap());
        }
    }
}
