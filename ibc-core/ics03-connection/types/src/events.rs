//! Types for the IBC events emitted from Tendermint Websocket by the connection module.

use data_types_macros::{define_attribute, define_event};
use ibc_core_host_types::identifiers::{ClientId, ConnectionId};
use ibc_primitives::prelude::*;

define_attribute!(
    "connection_id" => ConnectionIdAttribute(ConnectionId) {
        friendly_name = "connection ID",
        into = String::from,
        parse = |connection_id: &str| connection_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "client_id" => ClientIdAttribute(ClientId) {
        friendly_name = "connection ID",
        into = String::from,
        parse = |client_id: &str| client_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "counterparty_connection_id" => CounterpartyConnectionIdAttribute(ConnectionId) {
        friendly_name = "connection ID",
        into = String::from,
        parse = |connection_id: &str| connection_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "counterparty_client_id" => CounterpartyClientIdAttribute(ClientId) {
        friendly_name = "connection ID",
        into = String::from,
        parse = |client_id: &str| client_id.parse().map_err(Into::into),
    }
);

define_event!(
    "connection_open_init" => OpenInit {
        conn_id_on_a: ConnectionIdAttribute,
        client_id_on_a: ClientIdAttribute,
        client_id_on_b: CounterpartyClientIdAttribute,
    }
);

define_event!(
    "connection_open_try" => OpenTry {
        conn_id_on_a: ConnectionIdAttribute,
        client_id_on_a: ClientIdAttribute,
        conn_id_on_b: CounterpartyConnectionIdAttribute,
        client_id_on_b: CounterpartyClientIdAttribute,
    }
);

define_event!(
    "connection_open_ack" => OpenAck {
        conn_id_on_a: ConnectionIdAttribute,
        client_id_on_a: ClientIdAttribute,
        conn_id_on_b: CounterpartyConnectionIdAttribute,
        client_id_on_b: CounterpartyClientIdAttribute,
    }
);

define_event!(
    "connection_open_confirm" => OpenConfirm {
        conn_id_on_a: ConnectionIdAttribute,
        client_id_on_a: ClientIdAttribute,
        conn_id_on_b: CounterpartyConnectionIdAttribute,
        client_id_on_b: CounterpartyClientIdAttribute,
    }
);

#[cfg(test)]
mod tests {

    use core::str::FromStr;

    use ibc_core_host_types::identifiers::ClientType;
    use tendermint::abci::Event as AbciEvent;

    use super::*;

    #[test]
    fn ibc_to_abci_connection_events() {
        struct Test {
            kind: &'static str,
            event: AbciEvent,
            expected_keys: Vec<&'static str>,
            expected_values: Vec<&'static str>,
        }

        let client_type = ClientType::from_str("07-tendermint")
            .expect("never fails because it's a valid client type");
        let conn_id_on_a = ConnectionId::zero();
        let client_id_on_a = client_type.build_client_id(0);
        let conn_id_on_b = ConnectionId::new(1);
        let client_id_on_b = client_type.build_client_id(1);
        let expected_keys = vec![
            "connection_id",
            "client_id",
            "counterparty_connection_id",
            "counterparty_client_id",
        ];

        let tests: Vec<Test> = vec![
            Test {
                kind: OpenInit::EVENT_KIND,
                event: OpenInit {
                    conn_id_on_a: conn_id_on_a.clone().into(),
                    client_id_on_a: client_id_on_a.clone().into(),
                    client_id_on_b: client_id_on_b.clone().into(),
                }
                .into(),
                expected_keys: vec!["connection_id", "client_id", "counterparty_client_id"],
                expected_values: vec!["connection-0", "07-tendermint-0", "07-tendermint-1"],
            },
            Test {
                kind: OpenTry::EVENT_KIND,
                event: OpenTry {
                    conn_id_on_a: conn_id_on_b.clone().into(),
                    client_id_on_a: client_id_on_b.clone().into(),
                    conn_id_on_b: conn_id_on_a.clone().into(),
                    client_id_on_b: client_id_on_a.clone().into(),
                }
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: vec![
                    "connection-1",
                    "07-tendermint-1",
                    "connection-0",
                    "07-tendermint-0",
                ],
            },
            Test {
                kind: OpenAck::EVENT_KIND,
                event: OpenAck {
                    conn_id_on_a: conn_id_on_a.clone().into(),
                    client_id_on_a: client_id_on_a.clone().into(),
                    conn_id_on_b: conn_id_on_b.clone().into(),
                    client_id_on_b: client_id_on_b.clone().into(),
                }
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: vec![
                    "connection-0",
                    "07-tendermint-0",
                    "connection-1",
                    "07-tendermint-1",
                ],
            },
            Test {
                kind: OpenConfirm::EVENT_KIND,
                event: OpenConfirm {
                    conn_id_on_a: conn_id_on_b.into(),
                    client_id_on_a: client_id_on_b.into(),
                    conn_id_on_b: conn_id_on_a.into(),
                    client_id_on_b: client_id_on_a.into(),
                }
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: vec![
                    "connection-1",
                    "07-tendermint-1",
                    "connection-0",
                    "07-tendermint-0",
                ],
            },
        ];

        for t in tests {
            assert_eq!(t.kind, t.event.kind);
            assert_eq!(t.expected_keys.len(), t.event.attributes.len());
            for (i, e) in t.event.attributes.iter().enumerate() {
                assert_eq!(
                    e.key_str().unwrap(),
                    t.expected_keys[i],
                    "key mismatch for {:?}",
                    t.kind
                );
            }
            for (i, e) in t.event.attributes.iter().enumerate() {
                assert_eq!(
                    e.value_str().unwrap(),
                    t.expected_values[i],
                    "value mismatch for {:?}",
                    t.kind
                );
            }
        }
    }
}
