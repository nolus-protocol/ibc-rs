//! Types for the IBC events emitted from Tendermint Websocket by the channels module.

use data_types_macros::{define_attribute, define_event};
use ibc_core_client_types::Height;
use ibc_core_host_types::error::DecodingError;
use ibc_core_host_types::identifiers::{ChannelId, ConnectionId, PortId, Sequence};
use ibc_primitives::prelude::*;
use subtle_encoding::hex;

use super::acknowledgement::Acknowledgement;
use super::channel::Order;
use super::timeout::TimeoutHeight;
use super::Version;
use crate::timeout::TimeoutTimestamp;

define_attribute!(
    "connection_id" => ConnectionIdAttribute(ConnectionId) {
        friendly_name = "connection ID",
        into = String::from,
        parse = |connection_id: &str| connection_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "port_id" => PortIdAttribute(PortId) {
        friendly_name = "port ID",
        into = String::from,
        parse = |port_id: &str| port_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "counterparty_port_id" => CounterpartyPortIdAttribute(PortId) {
        friendly_name = "counterparty port ID",
        into = String::from,
        parse = |port_id: &str| port_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "channel_id" => ChannelIdAttribute(ChannelId) {
        friendly_name = "channel ID",
        into = String::from,
        parse = |channel_id: &str| channel_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "counterparty_channel_id" => CounterpartyChannelIdAttribute(ChannelId) {
        friendly_name = "counterparty channel ID",
        into = String::from,
        parse = |channel_id: &str| channel_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "counterparty_channel_id" => MaybeCounterpartyChannelIdAttribute(Option<ChannelId>) {
        friendly_name = "counterparty channel ID",
        into = |channel_id: Option<ChannelId>| channel_id.map_or_else(String::new, Into::into),
        parse = |channel_id: &str| {
            if channel_id.is_empty() {
                Ok(None)
            } else {
                channel_id.parse().map(Some).map_err(Into::into)
            }
        },
    }
);

define_attribute!(
    "version" => VersionAttribute(Version) {
        friendly_name = "version",
        into = String::from,
        parse = |version: &str| Ok(String::from(version).into()),
    }
);

define_attribute!(
    "packet_sequence" => SequenceAttribute(Sequence) {
        friendly_name = "version",
        into = |sequence: Sequence| sequence.to_string(),
        parse = |sequence: &str| sequence.parse().map_err(Into::into),
    }
);

define_attribute!(
    "packet_data_hex" => PacketDataAttribute(Vec<u8>) {
        friendly_name = "hex-encoded packet data",
        into = |data: Vec<u8>| {
            str::from_utf8(&hex::encode(data))
                .expect("Never fails because hexadecimal is valid UTF8")
                .to_string()
        },
        parse = |data: &str| hex::decode(data).map_err(|e| {
            DecodingError::invalid_raw_data(format!("packet data attribute value: {e}"))
        }),
    }
);

define_attribute!(
    "packet_src_port" => SrcPortIdAttribute(PortId) {
        friendly_name = "packet source port ID",
        into = String::from,
        parse = |port_id: &str| port_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "packet_src_channel" => SrcChannelIdAttribute(ChannelId) {
        friendly_name = "packet source channel ID",
        into = String::from,
        parse = |channel_id: &str| channel_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "packet_dst_port" => DstPortIdAttribute(PortId) {
        friendly_name = "packet destination port ID",
        into = String::from,
        parse = |port_id: &str| port_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "packet_dst_channel" => DstChannelIdAttribute(ChannelId) {
        friendly_name = "packet destination channel ID",
        into = String::from,
        parse = |channel_id: &str| channel_id.parse().map_err(Into::into),
    }
);

define_attribute!(
    "packet_channel_ordering" => ChannelOrderingAttribute(Order) {
        friendly_name = "packet destination channel ID",
        into = |ordering: Order| ordering.to_string(),
        parse = |ordering: &str| ordering.parse().map_err(|e| DecodingError::invalid_raw_data(format_args!("packet channel ordering: {e}"))),
    }
);

define_attribute!(
    "packet_timeout_height" => TimeoutHeightAttribute(TimeoutHeight) {
        friendly_name = "packet timeout height",
        into = TimeoutHeight::to_event_attribute_value,
        parse = |height: &str| {
            let no_timeout = TimeoutHeight::no_timeout();

            if height == no_timeout.to_event_attribute_value() {
                Ok(no_timeout)
            } else {
                height.parse::<Height>().map(Into::into)
            }
        },
    }
);

define_attribute!(
    "packet_timeout_timestamp" => TimeoutTimestampAttribute(TimeoutTimestamp) {
        friendly_name = "packet timeout timestamp",
        into = |timeout_timestamp: TimeoutTimestamp| timeout_timestamp.nanoseconds().to_string(),
        parse = |height: &str| {
            height
                .parse::<u64>()
                .map(Into::into)
                .map_err(|e| DecodingError::invalid_raw_data(format_args!("invalid timeout timestamp: {e}")))
        },
    }
);

define_attribute!(
    "packet_ack_hex" => AcknowledgementAttribute(Acknowledgement) {
        friendly_name = "hex-encoded packet acknowledge",
        into = |ack: Acknowledgement| {
            str::from_utf8(&hex::encode(ack.as_bytes()))
                .expect("Never fails because hexadecimal is valid UTF8")
                .to_string()
        },
        parse = |ack: &str| {
            hex::decode(ack)
                .map_err(|e| {
                    DecodingError::invalid_raw_data(format!("packet acknowledge attribute value: {e}"))
                })?
                .try_into()
        },
    }
);

define_attribute!(
    "connection_id" => PacketConnectionIdAttribute(ConnectionId) {
        friendly_name = "connection ID",
        into = String::from,
        parse = |connection_id: &str| connection_id.parse().map_err(Into::into),
    }
);

define_event!(
    "channel_open_init" => OpenInit {
        port_id_on_a: PortIdAttribute,
        chan_id_on_a: ChannelIdAttribute,
        port_id_on_b: CounterpartyPortIdAttribute,
        conn_id_on_a: ConnectionIdAttribute,
        version_on_a: VersionAttribute,
    }
);

define_event!(
    "channel_open_try" => OpenTry {
        port_id_on_b: PortIdAttribute,
        chan_id_on_b: ChannelIdAttribute,
        port_id_on_a: CounterpartyPortIdAttribute,
        chan_id_on_a: CounterpartyChannelIdAttribute,
        conn_id_on_b: ConnectionIdAttribute,
        version_on_b: VersionAttribute,
    }
);

define_event!(
    "channel_open_ack" => OpenAck {
        port_id_on_a: PortIdAttribute,
        chan_id_on_a: ChannelIdAttribute,
        port_id_on_b: CounterpartyPortIdAttribute,
        chan_id_on_b: CounterpartyChannelIdAttribute,
        conn_id_on_a: ConnectionIdAttribute,
    }
);

define_event!(
    "channel_open_confirm" => OpenConfirm {
        port_id_on_b: PortIdAttribute,
        chan_id_on_b: ChannelIdAttribute,
        port_id_on_a: CounterpartyPortIdAttribute,
        chan_id_on_a: CounterpartyChannelIdAttribute,
        conn_id_on_b: ConnectionIdAttribute,
    }
);

define_event!(
    "channel_close_init" => CloseInit {
        port_id_on_a: PortIdAttribute,
        chan_id_on_a: ChannelIdAttribute,
        port_id_on_b: CounterpartyPortIdAttribute,
        chan_id_on_b: CounterpartyChannelIdAttribute,
        conn_id_on_a: ConnectionIdAttribute,
    }
);

define_event!(
    "channel_close_confirm" => CloseConfirm {
        port_id_on_b: PortIdAttribute,
        chan_id_on_b: ChannelIdAttribute,
        port_id_on_a: CounterpartyPortIdAttribute,
        chan_id_on_a: CounterpartyChannelIdAttribute,
        conn_id_on_b: ConnectionIdAttribute,
    }
);

define_event!(
    "channel_close" => ChannelClosed {
        port_id_on_a: PortIdAttribute,
        chan_id_on_a: ChannelIdAttribute,
        port_id_on_b: CounterpartyPortIdAttribute,
        maybe_chan_id_on_b: MaybeCounterpartyChannelIdAttribute,
        conn_id_on_a: ConnectionIdAttribute,
        ordering: ChannelOrderingAttribute,
    }
);

define_event!(
    "send_packet" => SendPacket {
        packet_data: PacketDataAttribute,
        timeout_height_on_b: TimeoutHeightAttribute,
        timeout_timestamp_on_b: TimeoutTimestampAttribute,
        seq_on_a: SequenceAttribute,
        port_id_on_a: SrcPortIdAttribute,
        chan_id_on_a: SrcChannelIdAttribute,
        port_id_on_b: DstPortIdAttribute,
        chan_id_on_b: DstChannelIdAttribute,
        ordering: ChannelOrderingAttribute,
        conn_id_on_a: PacketConnectionIdAttribute,
    }
);

define_event!(
    "recv_packet" => ReceivePacket {
        packet_data: PacketDataAttribute,
        timeout_height_on_b: TimeoutHeightAttribute,
        timeout_timestamp_on_b: TimeoutTimestampAttribute,
        seq_on_a: SequenceAttribute,
        port_id_on_a: SrcPortIdAttribute,
        chan_id_on_a: SrcChannelIdAttribute,
        port_id_on_b: DstPortIdAttribute,
        chan_id_on_b: DstChannelIdAttribute,
        ordering: ChannelOrderingAttribute,
        conn_id_on_b: PacketConnectionIdAttribute,
    }
);

define_event!(
    "write_acknowledgement" => WriteAcknowledgement {
        packet_data: PacketDataAttribute,
        timeout_height_on_b: TimeoutHeightAttribute,
        timeout_timestamp_on_b: TimeoutTimestampAttribute,
        seq_on_a: SequenceAttribute,
        port_id_on_a: SrcPortIdAttribute,
        chan_id_on_a: SrcChannelIdAttribute,
        port_id_on_b: DstPortIdAttribute,
        chan_id_on_b: DstChannelIdAttribute,
        acknowledgement: AcknowledgementAttribute,
        conn_id_on_b: PacketConnectionIdAttribute,
    }
);

define_event!(
    "acknowledge_packet" => AcknowledgePacket {
        timeout_height_on_b: TimeoutHeightAttribute,
        timeout_timestamp_on_b: TimeoutTimestampAttribute,
        seq_on_a: SequenceAttribute,
        port_id_on_a: SrcPortIdAttribute,
        chan_id_on_a: SrcChannelIdAttribute,
        port_id_on_b: DstPortIdAttribute,
        chan_id_on_b: DstChannelIdAttribute,
        ordering: ChannelOrderingAttribute,
        conn_id_on_a: PacketConnectionIdAttribute,
    }
);

define_event!(
    "timeout_packet" => TimeoutPacket {
        timeout_height_on_b: TimeoutHeightAttribute,
        timeout_timestamp_on_b: TimeoutTimestampAttribute,
        seq_on_a: SequenceAttribute,
        port_id_on_a: SrcPortIdAttribute,
        chan_id_on_a: SrcChannelIdAttribute,
        port_id_on_b: DstPortIdAttribute,
        chan_id_on_b: DstChannelIdAttribute,
        ordering: ChannelOrderingAttribute,
    }
);

#[cfg(test)]
mod tests {
    use tendermint::abci::Event as AbciEvent;

    use super::*;

    #[test]
    fn ibc_to_abci_channel_events() {
        struct Test {
            kind: &'static str,
            event: AbciEvent,
            expected_keys: Vec<&'static str>,
            expected_values: Vec<&'static str>,
        }

        let port_id = PortId::transfer();
        let channel_id = ChannelId::zero();
        let connection_id = ConnectionId::zero();
        let counterparty_port_id = PortId::transfer();
        let counterparty_channel_id = ChannelId::new(1);
        let version = Version::new("ics20-1".to_string());
        let expected_keys = vec![
            "port_id",
            "channel_id",
            "counterparty_port_id",
            "counterparty_channel_id",
            "connection_id",
            "version",
        ];
        let expected_values = vec![
            "transfer",
            "channel-0",
            "transfer",
            "channel-1",
            "connection-0",
            "ics20-1",
        ];

        let tests: Vec<Test> = vec![
            Test {
                kind: OpenInit::EVENT_KIND,
                event: OpenInit::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    connection_id.clone(),
                    version.clone(),
                )
                .into(),
                expected_keys: vec![
                    "port_id",
                    "channel_id",
                    "counterparty_port_id",
                    "connection_id",
                    "version",
                ],
                expected_values: vec![
                    "transfer",
                    "channel-0",
                    "transfer",
                    "connection-0",
                    "ics20-1",
                ],
            },
            Test {
                kind: OpenTry::EVENT_KIND,
                event: OpenTry::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    counterparty_channel_id.clone(),
                    connection_id.clone(),
                    version,
                )
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: expected_values.clone(),
            },
            Test {
                kind: OpenAck::EVENT_KIND,
                event: OpenAck::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    counterparty_channel_id.clone(),
                    connection_id.clone(),
                )
                .into(),
                expected_keys: expected_keys[0..5].to_vec(),
                expected_values: expected_values[0..5].to_vec(),
            },
            Test {
                kind: OpenConfirm::EVENT_KIND,
                event: OpenConfirm::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    counterparty_channel_id.clone(),
                    connection_id.clone(),
                )
                .into(),
                expected_keys: expected_keys[0..5].to_vec(),
                expected_values: expected_values[0..5].to_vec(),
            },
            Test {
                kind: CloseInit::EVENT_KIND,
                event: CloseInit::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    counterparty_channel_id.clone(),
                    connection_id.clone(),
                )
                .into(),
                expected_keys: expected_keys[0..5].to_vec(),
                expected_values: expected_values[0..5].to_vec(),
            },
            Test {
                kind: CloseConfirm::EVENT_KIND,
                event: CloseConfirm::new(
                    port_id,
                    channel_id,
                    counterparty_port_id,
                    counterparty_channel_id,
                    connection_id,
                )
                .into(),
                expected_keys: expected_keys[0..5].to_vec(),
                expected_values: expected_values[0..5].to_vec(),
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
            assert_eq!(t.expected_values.len(), t.event.attributes.len());
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
