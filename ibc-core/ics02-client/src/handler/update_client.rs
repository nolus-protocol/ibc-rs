//! Protocol logic specific to processing ICS2 messages of type `MsgUpdateAnyClient`.

use ibc_core_client_context::prelude::*;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::events::{ClientMisbehaviour, UpdateClient};
use ibc_core_client_types::msgs::MsgUpdateOrMisbehaviour;
use ibc_core_client_types::{Height, UpdateKind};
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::error::HostError;
use ibc_core_host::types::identifiers::{ClientId, ClientType};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;
use ibc_primitives::ToVec;

pub fn validate<Ctx>(ctx: &Ctx, msg: MsgUpdateOrMisbehaviour) -> Result<(), ClientError>
where
    Ctx: ValidationContext,
{
    ctx.validate_message_signer(msg.signer())?;

    let client_id = msg.client_id().clone();

    let client_val_ctx = ctx.get_client_validation_context();

    // Read client state from the host chain store. The client should already exist.
    let client_state = client_val_ctx.client_state(&client_id)?;

    client_state
        .status(client_val_ctx, &client_id)?
        .verify_is_active()?;

    let client_message = msg.client_message();

    client_state.verify_client_message(client_val_ctx, &client_id, client_message)?;

    Ok(())
}

pub fn execute<Ctx>(ctx: &mut Ctx, msg: MsgUpdateOrMisbehaviour) -> Result<(), ClientError>
where
    Ctx: ExecutionContext,
{
    let client_id = msg.client_id().clone();
    let update_kind = match msg {
        MsgUpdateOrMisbehaviour::UpdateClient(_) => UpdateKind::UpdateClient,
        MsgUpdateOrMisbehaviour::Misbehaviour(_) => UpdateKind::SubmitMisbehaviour,
    };
    let client_message = msg.client_message();

    let client_exec_ctx = ctx.get_client_execution_context();

    let client_state = client_exec_ctx.client_state(&client_id)?;
    let client_type = client_state.client_type();

    let found_misbehaviour =
        client_state.check_for_misbehaviour(client_exec_ctx, &client_id, client_message.clone())?;

    let (consensus_heights, header) = if found_misbehaviour {
        client_state.update_state_on_misbehaviour(client_exec_ctx, &client_id, client_message)?;
        (Vec::new(), Vec::new())
    } else {
        if !matches!(update_kind, UpdateKind::UpdateClient) {
            return Err(ClientError::FailedToHandleMisbehaviour {
                description: "misbehaviour submitted, but none found".to_string(),
            });
        }

        let header = client_message;

        let consensus_heights =
            client_state.update_state(client_exec_ctx, &client_id, header.clone())?;

        (consensus_heights, header.to_vec())
    };

    emit_update_client_events(
        ctx,
        client_id,
        client_type,
        found_misbehaviour,
        consensus_heights,
        header,
    )
}

/// Emit the IBC events that conclude an `update_client` execution, for either a
/// successful header update or detected misbehaviour.
///
/// This holds the event-emission block shared by the generic `Any`
/// update-client handler above and any client-specific typed update path, so
/// the two emit identical events from a single source. It stays client agnostic:
/// it is parameterized only by the data carried in the events — `client_id`,
/// `client_type`, the resulting `consensus_heights`, and the raw `header` bytes
/// — plus a `found_misbehaviour` flag selecting which event to emit, and never
/// sees a concrete header type.
///
/// When `found_misbehaviour` is `false`, `consensus_heights` must be non-empty;
/// `consensus_heights` and `header` are ignored on the misbehaviour path.
pub fn emit_update_client_events<Ctx>(
    ctx: &mut Ctx,
    client_id: ClientId,
    client_type: ClientType,
    found_misbehaviour: bool,
    consensus_heights: Vec<Height>,
    header: Vec<u8>,
) -> Result<(), ClientError>
where
    Ctx: ExecutionContext,
{
    let event = if found_misbehaviour {
        IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
            client_id: client_id.into(),
            client_type: client_type.into(),
        })
    } else {
        let consensus_height = *consensus_heights.first().ok_or(HostError::missing_state(
            "updated height in client update state",
        ))?;

        IbcEvent::UpdateClient(UpdateClient {
            client_id: client_id.into(),
            client_type: client_type.into(),
            #[expect(deprecated)]
            consensus_height: consensus_height.into(),
            consensus_heights: consensus_heights.into(),
            header: header.into(),
        })
    };

    ctx.emit_ibc_event(IbcEvent::Message(MessageEvent::Client))?;
    ctx.emit_ibc_event(event)?;

    Ok(())
}
