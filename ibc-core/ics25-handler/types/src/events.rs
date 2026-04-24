//! Defines events emitted during handling of IBC messages

use ibc_core_channel_types::events as ChannelEvents;
use ibc_core_client_types::events::{self as ClientEvents};
use ibc_core_connection_types::events as ConnectionEvents;
use ibc_core_host_types::error::DecodingError;
use ibc_core_router_types::event::ModuleEvent;
use ibc_primitives::prelude::*;
use tendermint::abci;

const MESSAGE_EVENT: &str = "message";

/// Events created by the IBC component of a chain, destined for a relayer.
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IbcEvent {
    CreateClient(ClientEvents::CreateClient),
    UpdateClient(ClientEvents::UpdateClient),
    UpgradeClient(ClientEvents::UpgradeClient),
    ClientMisbehaviour(ClientEvents::ClientMisbehaviour),

    OpenInitConnection(ConnectionEvents::OpenInit),
    OpenTryConnection(ConnectionEvents::OpenTry),
    OpenAckConnection(ConnectionEvents::OpenAck),
    OpenConfirmConnection(ConnectionEvents::OpenConfirm),

    OpenInitChannel(ChannelEvents::OpenInit),
    OpenTryChannel(ChannelEvents::OpenTry),
    OpenAckChannel(ChannelEvents::OpenAck),
    OpenConfirmChannel(ChannelEvents::OpenConfirm),
    CloseInitChannel(ChannelEvents::CloseInit),
    CloseConfirmChannel(ChannelEvents::CloseConfirm),

    SendPacket(ChannelEvents::SendPacket),
    ReceivePacket(ChannelEvents::ReceivePacket),
    WriteAcknowledgement(ChannelEvents::WriteAcknowledgement),
    AcknowledgePacket(ChannelEvents::AcknowledgePacket),
    TimeoutPacket(ChannelEvents::TimeoutPacket),
    ChannelClosed(ChannelEvents::ChannelClosed),

    Module(ModuleEvent),
    Message(MessageEvent),
}

impl TryFrom<IbcEvent> for abci::Event {
    type Error = DecodingError;

    fn try_from(event: IbcEvent) -> Result<Self, Self::Error> {
        Ok(match event {
            IbcEvent::CreateClient(event) => event.into(),
            IbcEvent::UpdateClient(event) => event.into(),
            IbcEvent::UpgradeClient(event) => event.into(),
            IbcEvent::ClientMisbehaviour(event) => event.into(),
            IbcEvent::OpenInitConnection(event) => event.into(),
            IbcEvent::OpenTryConnection(event) => event.into(),
            IbcEvent::OpenAckConnection(event) => event.into(),
            IbcEvent::OpenConfirmConnection(event) => event.into(),
            IbcEvent::OpenInitChannel(event) => event.into(),
            IbcEvent::OpenTryChannel(event) => event.into(),
            IbcEvent::OpenAckChannel(event) => event.into(),
            IbcEvent::OpenConfirmChannel(event) => event.into(),
            IbcEvent::CloseInitChannel(event) => event.into(),
            IbcEvent::CloseConfirmChannel(event) => event.into(),
            IbcEvent::SendPacket(event) => event.into(),
            IbcEvent::ReceivePacket(event) => event.into(),
            IbcEvent::WriteAcknowledgement(event) => event.into(),
            IbcEvent::AcknowledgePacket(event) => event.into(),
            IbcEvent::TimeoutPacket(event) => event.into(),
            IbcEvent::ChannelClosed(event) => event.into(),
            IbcEvent::Module(event) => event.into(),
            IbcEvent::Message(event) => abci::Event {
                kind: MESSAGE_EVENT.to_string(),
                attributes: vec![("module", event.module_attribute(), true).into()],
            },
        })
    }
}

impl IbcEvent {
    pub fn event_type(&self) -> &str {
        match self {
            IbcEvent::CreateClient(_) => ClientEvents::CreateClient::EVENT_KIND,
            IbcEvent::UpdateClient(_) => ClientEvents::UpdateClient::EVENT_KIND,
            IbcEvent::ClientMisbehaviour(_) => ClientEvents::ClientMisbehaviour::EVENT_KIND,
            IbcEvent::UpgradeClient(_) => ClientEvents::UpgradeClient::EVENT_KIND,
            IbcEvent::OpenInitConnection(_) => ConnectionEvents::OpenInit::EVENT_KIND,
            IbcEvent::OpenTryConnection(_) => ConnectionEvents::OpenTry::EVENT_KIND,
            IbcEvent::OpenAckConnection(_) => ConnectionEvents::OpenAck::EVENT_KIND,
            IbcEvent::OpenConfirmConnection(_) => ConnectionEvents::OpenConfirm::EVENT_KIND,
            IbcEvent::OpenInitChannel(_) => ChannelEvents::OpenInit::EVENT_KIND,
            IbcEvent::OpenTryChannel(_) => ChannelEvents::OpenTry::EVENT_KIND,
            IbcEvent::OpenAckChannel(_) => ChannelEvents::OpenAck::EVENT_KIND,
            IbcEvent::OpenConfirmChannel(_) => ChannelEvents::OpenConfirm::EVENT_KIND,
            IbcEvent::CloseInitChannel(_) => ChannelEvents::CloseInit::EVENT_KIND,
            IbcEvent::CloseConfirmChannel(_) => ChannelEvents::CloseConfirm::EVENT_KIND,
            IbcEvent::SendPacket(_) => ChannelEvents::SendPacket::EVENT_KIND,
            IbcEvent::ReceivePacket(_) => ChannelEvents::ReceivePacket::EVENT_KIND,
            IbcEvent::WriteAcknowledgement(_) => ChannelEvents::WriteAcknowledgement::EVENT_KIND,
            IbcEvent::AcknowledgePacket(_) => ChannelEvents::AcknowledgePacket::EVENT_KIND,
            IbcEvent::TimeoutPacket(_) => ChannelEvents::TimeoutPacket::EVENT_KIND,
            IbcEvent::ChannelClosed(_) => ChannelEvents::ChannelClosed::EVENT_KIND,
            IbcEvent::Module(module_event) => module_event.kind.as_str(),
            IbcEvent::Message(_) => MESSAGE_EVENT,
        }
    }
}

/// An event type that is emitted by the Cosmos SDK.
///
/// We need to emit it as well, as currently [hermes] relies on it.
///
/// [hermes]: https://github.com/informalsystems/hermes
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageEvent {
    Client,
    Connection,
    Channel,
    // stores the module name
    Module(String),
}

impl MessageEvent {
    /// The ABCI event attribute has only one attribute, with key `module`.
    /// This method gets the associated value.
    pub fn module_attribute(&self) -> String {
        match self {
            MessageEvent::Client => "ibc_client".to_string(),
            MessageEvent::Connection => "ibc_connection".to_string(),
            MessageEvent::Channel => "ibc_channel".to_string(),
            MessageEvent::Module(module_name) => module_name.clone(),
        }
    }
}

impl From<MessageEvent> for IbcEvent {
    fn from(e: MessageEvent) -> Self {
        IbcEvent::Message(e)
    }
}

impl From<ModuleEvent> for IbcEvent {
    fn from(e: ModuleEvent) -> Self {
        IbcEvent::Module(e)
    }
}
