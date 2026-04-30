//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenTry`.;
use std::string::ToString;
use std::vec;

use ibc_core_client::context::prelude::{ClientStateCommon, ClientStateValidation, ConsensusState};
use ibc_core_client::context::ClientValidationContext;
use ibc_core_client::types::error::ClientError;
use ibc_core_connection_types::error::ConnectionError;
use ibc_core_connection_types::events::OpenTry;
use ibc_core_connection_types::msgs::MsgConnectionOpenTry;
use ibc_core_connection_types::{ConnectionEnd, Counterparty, State};
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::identifiers::{ClientId, ConnectionId};
use ibc_core_host::types::path::{
    ClientConnectionPath, ClientConsensusStatePath, ConnectionPath, Path,
};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::proto::{Any, Protobuf};

pub fn validate<Ctx>(ctx_b: &Ctx, msg: MsgConnectionOpenTry) -> Result<(), ConnectionError>
where
    Ctx: ValidationContext,
    <Ctx::HostClientState as TryFrom<Any>>::Error: Into<ClientError>,
{
    let vars = LocalVars::new(ctx_b, &msg)?;
    validate_impl(ctx_b, &msg, &vars)
}

fn validate_impl<Ctx>(
    ctx_b: &Ctx,
    msg: &MsgConnectionOpenTry,
    vars: &LocalVars,
) -> Result<(), ConnectionError>
where
    Ctx: ValidationContext,
    <Ctx::HostClientState as TryFrom<Any>>::Error: Into<ClientError>,
{
    ctx_b.validate_message_signer(&msg.signer)?;

    let client_val_ctx_b = ctx_b.get_client_validation_context();

    let client_id_on_a = msg.counterparty.client_id();

    // Verify proofs
    {
        let client_state_of_a_on_b =
            client_val_ctx_b.client_state(vars.conn_end_on_b.client_id())?;

        client_state_of_a_on_b
            .status(client_val_ctx_b, &msg.client_id_on_b)?
            .verify_is_active()?;

        client_state_of_a_on_b.validate_proof_height(msg.proofs_height_on_a)?;

        let client_cons_state_path_on_b = ClientConsensusStatePath::new(
            msg.client_id_on_b.clone(),
            msg.proofs_height_on_a.revision_number(),
            msg.proofs_height_on_a.revision_height(),
        );

        let consensus_state_of_a_on_b =
            client_val_ctx_b.consensus_state(&client_cons_state_path_on_b)?;

        let prefix_on_a = vars.conn_end_on_b.counterparty().prefix();
        let prefix_on_b = ctx_b.commitment_prefix();

        {
            //to stay
            let expected_conn_end_on_a = ConnectionEnd::new(
                State::Init,
                client_id_on_a.clone(),
                Counterparty::new(msg.client_id_on_b.clone(), None, prefix_on_b),
                msg.versions_on_a.clone(),
                msg.delay_period,
            )?;

            // For reference, the full call stack in IBC-go:
            // https://github.com/cosmos/ibc-go/blob/93baefd28cdceb2540135d2519edf6ecfd8715cc/modules/core/03-connection/keeper/handshake.go#L64
            // https://github.com/cosmos/ibc-go/blob/93baefd28cdceb2540135d2519edf6ecfd8715cc/modules/core/03-connection/keeper/verify.go#L19
            // https://github.com/cosmos/ibc-go/blob/93baefd28cdceb2540135d2519edf6ecfd8715cc/modules/core/02-client/keeper/keeper.go#L335
            // https://github.com/cosmos/ibc-go/blob/93baefd28cdceb2540135d2519edf6ecfd8715cc/modules/light-clients/07-tendermint/light_client_module.go#L103
            // https://github.com/cosmos/ibc-go/blob/93baefd28cdceb2540135d2519edf6ecfd8715cc/modules/light-clients/07-tendermint/client_state.go#L206
            client_state_of_a_on_b.verify_membership(
                prefix_on_a,                      // ~= connection.counterparty.Prefix()
                &msg.proof_conn_end_on_a,         // ~= initProof
                consensus_state_of_a_on_b.root(), // https://github.com/cosmos/ibc-go/blob/93baefd28cdceb2540135d2519edf6ecfd8715cc/modules/light-clients/07-tendermint/client_state.go#L238
                Path::Connection(ConnectionPath::new(&vars.conn_id_on_a)), // ~= counterparty.ConnectionId
                expected_conn_end_on_a.encode_vec(),                       // ~= expectedConnection
            )?;
        }
    }

    Ok(())
}

pub fn execute<Ctx>(ctx_b: &mut Ctx, msg: MsgConnectionOpenTry) -> Result<(), ConnectionError>
where
    Ctx: ExecutionContext,
{
    let vars = LocalVars::new(ctx_b, &msg)?;
    execute_impl(ctx_b, msg, vars)
}

fn execute_impl<Ctx>(
    ctx_b: &mut Ctx,
    msg: MsgConnectionOpenTry,
    vars: LocalVars,
) -> Result<(), ConnectionError>
where
    Ctx: ExecutionContext,
{
    let conn_id_on_a = vars
        .conn_end_on_b
        .counterparty()
        .connection_id()
        .ok_or(ConnectionError::InvalidCounterparty)?;
    let event = IbcEvent::OpenTryConnection(OpenTry::new(
        vars.conn_id_on_b.clone(),
        msg.client_id_on_b.clone(),
        conn_id_on_a.clone(),
        vars.client_id_on_a.clone(),
    ));
    ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Connection))?;
    ctx_b.emit_ibc_event(event)?;
    ctx_b.log_message("success: conn_open_try verification passed".to_string())?;

    ctx_b.increase_connection_counter()?;
    ctx_b.store_connection_to_client(
        &ClientConnectionPath::new(msg.client_id_on_b),
        vars.conn_id_on_b.clone(),
    )?;
    ctx_b.store_connection(&ConnectionPath::new(&vars.conn_id_on_b), vars.conn_end_on_b)?;

    Ok(())
}

struct LocalVars {
    conn_id_on_b: ConnectionId,
    conn_end_on_b: ConnectionEnd,
    client_id_on_a: ClientId,
    conn_id_on_a: ConnectionId,
}

impl LocalVars {
    fn new<Ctx>(ctx_b: &Ctx, msg: &MsgConnectionOpenTry) -> Result<Self, ConnectionError>
    where
        Ctx: ValidationContext,
    {
        let version_on_b = ctx_b.pick_version(&msg.versions_on_a)?;

        Ok(Self {
            conn_id_on_b: ConnectionId::new(ctx_b.connection_counter()?),
            conn_end_on_b: ConnectionEnd::new(
                State::TryOpen,
                msg.client_id_on_b.clone(),
                msg.counterparty.clone(),
                vec![version_on_b],
                msg.delay_period,
            )?,
            client_id_on_a: msg.counterparty.client_id().clone(),
            conn_id_on_a: msg
                .counterparty
                .connection_id()
                .ok_or(ConnectionError::InvalidCounterparty)?
                .clone(),
        })
    }
}
