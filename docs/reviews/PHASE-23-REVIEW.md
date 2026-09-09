# Phase 23 Review — WebSocket Master Gain/Mute Control

## Objective
Complete WebSocket role-based authorization (Phase 3 HIGH security gate) by adding `SetMasterGain`/`SetMasterMute` control messages with `MasterAck` response, broadcast fan-out, and explicit Musician RBAC block.

## Status: PASS

## Implemented
- `SetMasterGain` and `SetMasterMute` in `control-protocol::ClientMessage`.
- `MasterAck` in `control-protocol::ServerMessage`.
- `MasterDelta` broadcast struct and `master_event_tx` channel in `AppState`.
- Dispatch in `control-server`: validation, mix lookup, state mutation, `MasterAck` response.
- WebSocket handler: `check_permission` Musician block (explicit arm), `master_mix_index` helper, 3-branch `tokio::select!`, broadcast fan-out with role/assignment filter.
- 7 new integration tests; 215 total pass.

## Tests
- 215 tests, 0 failures (cargo test --workspace).
- WS musician-filtered master broadcast test added after reviewer suggestion.

## Metrics
- Tests: 215 (was 214 before musician filter test).
- Static scan: clean.
- Clippy -D warnings: clean.

## Security
- Musician role blocked from `SetMasterGain`/`SetMasterMute` by explicit `check_permission` arm.
- Broadcast filtered by role and mix assignment (same model as send deltas).
- No hardcoded secrets, no shell injection, no SQL injection.
- **Closes:** Phase 3 HIGH — "Authorize every WebSocket message by role and musician mix ownership".

## Architecture Impact
- `AppState` gains `master_event_tx` broadcast channel.
- WS select! grows from 2 to 3 branches.
- No breaking protocol changes; `MasterAck` is additive.

## Documentation
- CHANGELOG.md updated.
- docs/TODO.md updated.
- docs/DEVELOPMENT-LOG.md updated.
- This review document created.

## Known Issues / Low Priority
- DB error in master broadcast fan-out silenced via `unwrap_or(None)` — fail-closed, safe, not observable (TODO LOW added).
- `mix_assignment_lock` held during read-only DB lookup in broadcast fan-out — may contend under load (TODO LOW added).

## Follow-up
- Publicar `v0.3.1` somente após CI remoto executar e passar todos os gates; execução atual falha antes dos steps e logs não estão acessíveis pelo token atual.
- Code coverage reporting.
- Validar extração e renderização do Musician Guide PDF quando `pdftotext`/renderizador estiverem disponíveis.
- Validar ARM64 em Raspberry Pi 5 real.
