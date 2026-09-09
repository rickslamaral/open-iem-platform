# Phase 10 Implementation Guide

## 1. DB Layer additions (server/api-server/src/db.rs)

Add a mix_assignments table to the existing migrate() execute_batch string.
mix_index is 0-based integer (0 or 1 since MAX_MIXES=2). One musician per mix, one mix per musician.

Add DB methods:
- assign_mix(mix_index: usize, user_id: i64) -> Result<(), ApiError>
- unassign_mix(mix_index: usize) -> Result<(), ApiError>
- list_mix_assignments() -> Result<Vec<(usize, i64, String)>, ApiError>
- get_mix_assignment(mix_index: usize) -> Result<Option<i64>, ApiError>
- get_user_assigned_mix(user_id: i64) -> Result<Option<usize>, ApiError>

## 2. Auth: add user_id (i64) as field "uid" to JwtClaims, update issue_access_token and callers.

## 3. Mix helpers in server/mix-engine/src/mix.rs:
- get_send(channel_idx: usize) -> Option<&MixSend>
- set_send_gain_db(channel_idx: usize, gain_db: f32) -> Result<(), MixError>
- set_send_pan(channel_idx: usize, pan: f32) -> Result<(), MixError>
- set_send_muted(channel_idx: usize, muted: bool) -> Result<(), MixError>
Return MixError::InvalidIndex if channel_idx >= MAX_CHANNELS.

## 4. New routes/mixes.rs with list_mixes, assign_mix, unassign_mix, get_send_state, set_send_gain, set_send_pan, set_send_muted.
Ownership: Engineer/Admin any mix; Musician only own assigned mix.

## 5. Wire routes in main.rs and add pub mod mixes to routes/mod.rs.

## 6. Integration tests and DB unit tests as described.
