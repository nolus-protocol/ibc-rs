- [ibc-client-tendermint] Add `update_state_from_header` and
  `check_for_misbehaviour_from_header`, which take an already-decoded `Header`,
  and make the existing `Any`-taking `update_state` / `check_for_misbehaviour`
  thin decode-then-delegate wrappers over them, so an `update_client` header can
  be decoded once rather than re-decoded by each entry point.
  ([\#12](https://github.com/nolus-protocol/ibc-rs/issues/12))
- [ibc-core-client] Add `emit_update_client_events`, a shared client-agnostic
  helper that single-sources `UpdateClient` / `ClientMisbehaviour` event
  emission for both the generic handler and a typed update path.
  ([\#12](https://github.com/nolus-protocol/ibc-rs/issues/12))
