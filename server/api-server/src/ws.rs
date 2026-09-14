//! WebSocket handler: `/ws/v1`
//!
//! Authenticates via `Sec-WebSocket-Protocol: openiem.bearer.<JWT>, openiem.v1` during HTTP upgrade.
//! Only `openiem.v1` is echoed in the upgrade response; bearer token is never returned.
//! Each connection dispatches `ClientMessage` frames and receives `ServerMessage` responses.
//! Connection is closed on:
//!   - Auth failure
//!   - Malformed message
//!   - Message too large
//!   - Server shutdown
//!
//! The process accepts at most `MAX_WEBSOCKET_CONNECTIONS` upgraded connections;
//! each authenticated user is limited to four connections and each peer IP to
//! sixteen connections; excess upgrades receive HTTP 503 or HTTP 429 with
//! `Retry-After: 5`.
//!
//! ## Broadcast model
//!
//! After a successful send mutation (`SetSendGain`, `SetSendPan`, `SetSendMuted`)
//! the handler publishes a `SendDelta` event on `AppState::event_tx`.
//!
//! Every connected session subscribes to that channel via `event_rx` and
//! forwards matching events to its client:
//!  - **Engineer/Admin** — receive deltas for all mixes.
//!  - **Musician** — receive deltas only for their assigned mix.
//!
//! Deltas are sent as unsolicited `SendAck` envelopes with `request_id = "server"`.

#![allow(clippy::unused_async)]

use crate::{
    auth::JwtClaims,
    quota::QuotaRejection,
    state::{AppState, EqBandDelta, MasterDelta, SendDelta},
};
use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket},
        Extension, State, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
};
use control_protocol::{
    decode_client_message, ClientMessage, Envelope, ProtocolError, Role, ServerMessage,
    MAX_MESSAGE_BYTES, PROTOCOL_VERSION,
};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::Instant;

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

const MAX_MESSAGES_PER_MINUTE: u32 = 120;
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_SOCKET_SEND_WAIT: Duration = Duration::from_secs(10);

#[derive(Debug, Default)]
struct KeepaliveTracker {
    ping_sent_at: Option<Instant>,
    expected_pong: Option<Vec<u8>>,
}

impl KeepaliveTracker {
    fn expired(&self, now: Instant) -> bool {
        self.expected_pong.is_some()
            && self
                .ping_sent_at
                .is_some_and(|sent| now.saturating_duration_since(sent) >= KEEPALIVE_TIMEOUT)
    }

    fn should_send_ping(&self) -> bool {
        self.expected_pong.is_none()
    }

    fn record_ping(&mut self, payload: Vec<u8>, now: Instant) {
        self.ping_sent_at = Some(now);
        self.expected_pong = Some(payload);
    }

    fn receive_pong(&mut self, payload: &[u8], now: Instant) -> bool {
        if self.expected_pong.as_deref() != Some(payload) {
            return false;
        }
        self.ping_sent_at = Some(now);
        self.expected_pong = None;
        true
    }
}

async fn send_with_timeout(socket: &mut WebSocket, message: Message) -> bool {
    tokio::time::timeout(MAX_SOCKET_SEND_WAIT, socket.send(message))
        .await
        .is_ok_and(|result| result.is_ok())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

use tracing::{debug, warn};

fn musician_assigned_to_mix(state: &AppState, user_id: i64, mix_index: u8) -> bool {
    match state.db.get_user_assigned_mix(user_id) {
        Ok(assigned) => assigned == Some(usize::from(mix_index)),
        Err(error) => {
            warn!(%error, "failed to read musician mix assignment; denying access");
            false
        }
    }
}

/// Maximum WebSocket message size in bytes (16 KiB).
const MAX_WS_MESSAGE_BYTES: usize = MAX_MESSAGE_BYTES;

/// `/ws/v1` WebSocket upgrade handler.
///
/// Auth via `Sec-WebSocket-Protocol: openiem.bearer.<JWT>` on initial HTTP upgrade request.
/// Rejects with 401 if token is missing or invalid.
pub async fn ws_handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    ws: WebSocketUpgrade,
    Extension(claims): Extension<JwtClaims>,
) -> Response {
    let permit = match state
        .websocket_connections
        .try_reserve(claims.user_id, peer.ip())
    {
        Ok(permit) => permit,
        Err(QuotaRejection::Global) => {
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                [(axum::http::header::RETRY_AFTER, "5")],
                "WebSocket connection limit reached",
            )
                .into_response();
        }
        Err(QuotaRejection::User) => {
            return (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                [(axum::http::header::RETRY_AFTER, "5")],
                "WebSocket user connection limit reached",
            )
                .into_response();
        }
        Err(QuotaRejection::Ip) => {
            return (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                [(axum::http::header::RETRY_AFTER, "5")],
                "WebSocket IP connection limit reached",
            )
                .into_response();
        }
    };
    ws.max_message_size(MAX_WS_MESSAGE_BYTES)
        .max_frame_size(MAX_WS_MESSAGE_BYTES)
        .protocols(["openiem.v1"])
        .on_upgrade(move |socket| {
            let session_id = u128::from(NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed));
            handle_socket(socket, state, claims, session_id, permit)
        })
        .into_response()
}

/// Long-running WebSocket I/O loop. The loop body is intentionally contained here
/// to keep the borrow checker happy with `socket` across await points.
#[allow(clippy::too_many_lines)]
async fn handle_socket(
    mut socket: WebSocket,
    state: AppState,
    claims: JwtClaims,
    session_id: u128,
    _connection_permit: crate::quota::WebSocketQuotaGuard,
) {
    debug!(user = %claims.sub, role = ?claims.role, "WebSocket connected");
    let mut window_started = unix_now();
    let mut message_count = 0_u32;
    let mut keepalive = tokio::time::interval_at(
        tokio::time::Instant::now() + KEEPALIVE_INTERVAL,
        KEEPALIVE_INTERVAL,
    );
    keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut keepalive_state = KeepaliveTracker::default();
    let mut ping_sequence = 0_u64;

    // Subscribe to broadcast events before entering the loop.
    let mut event_rx = state.event_tx.subscribe();
    let mut master_event_rx = state.master_event_tx.subscribe();
    let mut eq_band_event_rx = state.eq_band_event_tx.subscribe();

    loop {
        let now = unix_now();
        if now >= claims.exp {
            send_error(&mut socket, "TOKEN_EXPIRED", "access token expired").await;
            break;
        }
        let remaining = claims.exp - now;

        tokio::select! {
            // Server keepalive. A Pong within the last 60 seconds keeps session alive.
            _ = keepalive.tick() => {
                let Some(session_id_db) = claims.session_id else {
                    break;
                };
                if let Ok(true) = state.db.is_access_session_active(
                    &claims.jti,
                    claims.user_id,
                    session_id_db,
                    unix_now(),
                ) {
                } else {
                    send_error(&mut socket, "SESSION_REVOKED", "session revoked").await;
                    break;
                }
                if keepalive_state.expired(Instant::now()) {
                    send_error(&mut socket, "CONNECTION_TIMEOUT", "WebSocket pong timeout").await;
                    break;
                }
                if keepalive_state.should_send_ping() {
                    ping_sequence = ping_sequence.wrapping_add(1);
                    let payload = format!("openiem-keepalive-{session_id}-{ping_sequence}").into_bytes();
                    if !send_with_timeout(&mut socket, Message::Ping(payload.clone().into())).await {
                        break;
                    }
                    keepalive_state.record_ping(payload, Instant::now());
                }
            }

            // Inbound message from client.
            recv = tokio::time::timeout(Duration::from_secs(remaining), socket.recv()) => {
                let msg = match recv {
                    Ok(Some(Ok(m))) => m,
                    Ok(Some(Err(_))) => {
                        warn!(session_id, "WebSocket receive failed");
                        break;
                    }
                    Ok(None) => {
                        debug!(session_id, "WebSocket peer closed connection");
                        break;
                    }
                    Err(_) => {
                        send_error(&mut socket, "TOKEN_EXPIRED", "access token expired").await;
                        break;
                    }
                };

                let now = unix_now();
                if now >= claims.exp {
                    send_error(&mut socket, "TOKEN_EXPIRED", "access token expired").await;
                    break;
                }
                let Some(session_id_db) = claims.session_id else {
                    break;
                };
                if let Ok(true) = state.db.is_access_session_active(
                    &claims.jti,
                    claims.user_id,
                    session_id_db,
                    now,
                ) {
                } else {
                    send_error(&mut socket, "SESSION_REVOKED", "session revoked").await;
                    break;
                }
                if now.saturating_sub(window_started) >= 60 {
                    window_started = now;
                    message_count = 0;
                }
                message_count = message_count.saturating_add(1);
                if message_count > MAX_MESSAGES_PER_MINUTE {
                    send_error(&mut socket, "RATE_LIMITED", "too many messages").await;
                    break;
                }

                match msg {
                    Message::Text(text) => {
                        if text.len() > MAX_WS_MESSAGE_BYTES {
                            warn!("WebSocket message too large: {} bytes", text.len());
                            send_error(
                                &mut socket,
                                "MESSAGE_TOO_LARGE",
                                "message exceeds 16 KiB limit",
                            )
                            .await;
                            break;
                        }

                        let envelope = match decode_client_message(&text) {
                            Err(error) => {
                                let (code, message) = public_protocol_error(&error);
                                send_error_with_request(&mut socket, "server", code, message).await;
                                break;
                            }
                            Ok(env) => env,
                        };
                        let request_id = envelope.request_id.clone();

                        // Role-based permission check — also covers send mutations.
                        let permitted = check_permission(&claims, &envelope.payload);
                        if !permitted {
                            send_error_with_request(
                                &mut socket,
                                &request_id,
                                "FORBIDDEN",
                                "role cannot perform this action",
                            )
                            .await;
                            continue;
                        }

                        // Serialize assignment checks with assignment mutations and the
                        // control-state update. This closes the WS ownership TOCTOU window.
                        let assignment_guard = state.mix_assignment_lock.lock().await;
                        let musician_owns_send = if claims.role == Role::Musician {
                            send_mix_index(&envelope.payload).is_none_or(|mix_index| {
                                musician_assigned_to_mix(&state, claims.user_id, mix_index)
                            })
                        } else {
                            true
                        };
                        if !musician_owns_send {
                            drop(assignment_guard);
                            send_error_with_request(
                                &mut socket,
                                &request_id,
                                "FORBIDDEN",
                                "musician does not own this mix",
                            )
                            .await;
                            continue;
                        }

                        // Remember whether this is a send mutation before consuming the envelope.
                        let is_send_mutation = send_mix_index(&envelope.payload).is_some();
                        let is_master_mutation = master_mix_index(&envelope.payload).is_some();
                        let is_eq_mutation = is_eq_band_mutation(&envelope.payload);

                        // Dispatch with lock held only for the duration of the call.
                        // Lock is released before any await point.
                        let dispatch_result = {
                            let Ok(mut ctrl) = state.control.lock() else {
                                // Lock poisoned — close connection.
                                break;
                            };
                            Ok::<_, ()>(ctrl.dispatch(envelope))
                        };

                        let Ok(response) = dispatch_result else {
                            drop(assignment_guard);
                            send_error(&mut socket, "INTERNAL_ERROR", "state lock poisoned").await;
                            break;
                        };
                        // Broadcast while assignment lock remains held. This preserves
                        // mutation order relative to assignment changes.
                        if is_send_mutation {
                            if let ServerMessage::SendAck {
                                mix_index,
                                channel_index,
                                gain_db,
                                pan,
                                muted,
                                revision,
                            } = &response.payload
                            {
                                let delta = SendDelta {
                                    mix_index: *mix_index,
                                    channel_index: *channel_index,
                                    gain_db: *gain_db,
                                    pan: *pan,
                                    muted: *muted,
                                    revision: *revision,
                                    originator_session_id: session_id,
                                };
                                if let Err(e) = state.event_tx.send(delta) {
                                    // Only happens when channel is full (all 256 slots consumed
                                    // by lagging receivers).  Log at warn level so operators know
                                    // a delta was dropped rather than silently swallowed.
                                    warn!("broadcast channel full; send delta dropped: {e}");
                                }
                            }
                        }
                        if is_master_mutation {
                            if let ServerMessage::MasterAck {
                                mix_index,
                                master_gain_db,
                                master_muted,
                                revision,
                            } = &response.payload
                            {
                                let delta = MasterDelta {
                                    mix_index: *mix_index,
                                    master_gain_db: *master_gain_db,
                                    master_muted: *master_muted,
                                    revision: *revision,
                                    originator_session_id: session_id,
                                };
                                if let Err(e) = state.master_event_tx.send(delta) {
                                    warn!("broadcast channel full; master delta dropped: {e}");
                                }
                            }
                        }
                        if is_eq_mutation {
                            if let ServerMessage::EqBandAck {
                                mix_index,
                                band_index,
                                frequency_hz,
                                gain_db,
                                q,
                                enabled,
                                revision,
                            } = &response.payload
                            {
                                let delta = EqBandDelta {
                                    mix_index: *mix_index,
                                    band_index: *band_index,
                                    frequency_hz: *frequency_hz,
                                    gain_db: *gain_db,
                                    q: *q,
                                    enabled: *enabled,
                                    revision: *revision,
                                    originator_session_id: session_id,
                                };
                                if let Err(e) = state.eq_band_event_tx.send(delta) {
                                    warn!("broadcast channel full; EQ band delta dropped: {e}");
                                }
                            }
                        }
                        // Release coordination lock before any socket I/O.
                        drop(assignment_guard);

                        let json = match serde_json::to_string(&response) {
                            Ok(j) => j,
                            Err(e) => {
                                warn!("serialize error: {e}");
                                break;
                            }
                        };

                        if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                            break;
                        }
                    }
                    Message::Close(_) => {
                        debug!(user = %claims.sub, "WebSocket close frame");
                        break;
                    }
                    Message::Ping(payload) => {
                        if !send_with_timeout(&mut socket, Message::Pong(payload)).await {
                            break;
                        }
                    }
                    Message::Pong(payload) => {
                        keepalive_state.receive_pong(payload.as_ref(), Instant::now());
                    }
                    Message::Binary(_) => {
                        send_error(&mut socket, "INVALID_MESSAGE", "binary WebSocket frames are not supported").await;
                        break;
                    }
                }
            }

            // Outbound broadcast event from another session.
            event = event_rx.recv() => {
                match event {
                    Ok(delta) => {
                        // Skip self: the originating session already received a
                        // direct SendAck response and must not receive a second one.
                        if delta.originator_session_id == session_id {
                            continue;
                        }

                        // Lock-free ownership check — fan-out is read-only: no assignment
                        // mutation happens here. The mutation sender holds the lock for the
                        // atomic mutation+publish, which serialises order. Receivers do a
                        // read-only assignment lookup without contending on the lock.
                        // A momentary stale read on assignment transition is acceptable —
                        // the client re-syncs via REST snapshot.
                        // *receiver* side.  A narrow race exists if the musician's assignment
                        // changes concurrently: they may receive one extra delta for a mix they
                        // just left, or miss one delta for a mix they just joined.  Both cases
                        // are harmless — the client reconciles via a REST snapshot whenever it
                        // receives a `State` revision notice.  Holding the lock here would block
                        // all inbound assignment mutations while every connected session
                        // processes each broadcast event, creating O(sessions) contention.
                        let should_forward = match claims.role {
                            Role::Admin | Role::Engineer => true,
                            Role::Musician => musician_assigned_to_mix(
                                &state,
                                claims.user_id,
                                delta.mix_index,
                            )
                        };

                        let ack_json = if should_forward {
                            let ack = Envelope {
                                version: PROTOCOL_VERSION,
                                request_id: "server".to_owned(),
                                payload: ServerMessage::SendAck {
                                    mix_index: delta.mix_index,
                                    channel_index: delta.channel_index,
                                    gain_db: delta.gain_db,
                                    pan: delta.pan,
                                    muted: delta.muted,
                                    revision: delta.revision,
                                },
                            };
                            serde_json::to_string(&ack).ok()
                        } else {
                            None
                        };
                        if let Some(json) = ack_json {
                            if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(user = %claims.sub, "broadcast lagged by {n} events; some deltas skipped");
                        let revision = if let Ok(control) = state.control.lock() {
                            control.revision()
                        } else {
                            warn!(session_id, "control state lock poisoned during resync");
                            break;
                        };
                        let state_notice = Envelope {
                            version: PROTOCOL_VERSION,
                            request_id: "server".to_owned(),
                            payload: ServerMessage::State { revision },
                        };
                        if let Ok(json) = serde_json::to_string(&state_notice) {
                            if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        // Server shutting down.
                        break;
                    }
                }
            }

            // Outbound master broadcast event from another session.
            master_event = master_event_rx.recv() => {
                match master_event {
                    Ok(delta) => {
                        // Skip originator.
                        if delta.originator_session_id == session_id {
                            continue;
                        }

                        // Lock-free ownership check — same rationale as the send-delta fan-out
                        // above. The mutation sender holds the lock for the atomic
                        // mutation+publish; receivers do a read-only assignment lookup without
                        // contending on the lock.
                        let should_forward = match claims.role {
                            Role::Admin | Role::Engineer => true,
                            Role::Musician => musician_assigned_to_mix(
                                &state,
                                claims.user_id,
                                delta.mix_index,
                            )
                        };

                        let ack_json = if should_forward {
                            let ack = Envelope {
                                version: PROTOCOL_VERSION,
                                request_id: "server".to_owned(),
                                payload: ServerMessage::MasterAck {
                                    mix_index: delta.mix_index,
                                    master_gain_db: delta.master_gain_db,
                                    master_muted: delta.master_muted,
                                    revision: delta.revision,
                                },
                            };
                            serde_json::to_string(&ack).ok()
                        } else {
                            None
                        };
                        if let Some(json) = ack_json {
                            if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(user = %claims.sub, "master broadcast lagged by {n} events; some deltas skipped");
                        let revision = if let Ok(control) = state.control.lock() {
                            control.revision()
                        } else {
                            warn!(session_id, "control state lock poisoned during resync");
                            break;
                        };
                        let state_notice = Envelope {
                            version: PROTOCOL_VERSION,
                            request_id: "server".to_owned(),
                            payload: ServerMessage::State { revision },
                        };
                        if let Ok(json) = serde_json::to_string(&state_notice) {
                            if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            // Outbound EQ band broadcast — Engineer/Admin only (Musician cannot configure EQ).
            eq_band_event = eq_band_event_rx.recv() => {
                match eq_band_event {
                    Ok(delta) => {
                        // Skip originator.
                        if delta.originator_session_id == session_id {
                            continue;
                        }
                        // Musician role does not receive EQ deltas.
                        if claims.role == Role::Musician {
                            continue;
                        }
                        let ack = Envelope {
                            version: PROTOCOL_VERSION,
                            request_id: "server".to_owned(),
                            payload: ServerMessage::EqBandAck {
                                mix_index: delta.mix_index,
                                band_index: delta.band_index,
                                frequency_hz: delta.frequency_hz,
                                gain_db: delta.gain_db,
                                q: delta.q,
                                enabled: delta.enabled,
                                revision: delta.revision,
                            },
                        };
                        if let Ok(json) = serde_json::to_string(&ack) {
                            if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(user = %claims.sub, "EQ band broadcast lagged by {n} events; some deltas skipped");
                        // Send a State notice so the client knows to re-fetch.
                        let revision = if let Ok(control) = state.control.lock() {
                            control.revision()
                        } else {
                            warn!(session_id, "control state lock poisoned during resync");
                            break;
                        };
                        let state_notice = Envelope {
                            version: PROTOCOL_VERSION,
                            request_id: "server".to_owned(),
                            payload: ServerMessage::State { revision },
                        };
                        if let Ok(json) = serde_json::to_string(&state_notice) {
                            if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }

    debug!(user = %claims.sub, "WebSocket disconnected");
}

/// Returns true when the role is allowed to send this message.
fn check_permission(claims: &JwtClaims, msg: &ClientMessage) -> bool {
    match (&claims.role, msg) {
        // Admin/Engineer may send any message; Musician may read state and mutate own sends.
        (Role::Admin | Role::Engineer, _)
        | (
            Role::Musician,
            ClientMessage::GetState
            | ClientMessage::SetSendGain { .. }
            | ClientMessage::SetSendPan { .. }
            | ClientMessage::SetSendMuted { .. },
        ) => true,
        // Musician: cannot mutate channel gain/mute directly, master gain/mute, or EQ.
        (
            Role::Musician,
            ClientMessage::SetChannelGain { .. }
            | ClientMessage::SetChannelMute { .. }
            | ClientMessage::SetMasterGain { .. }
            | ClientMessage::SetMasterMute { .. }
            | ClientMessage::SetEqBand { .. },
        ) => false,
    }
}

/// Returns the `mix_index` for send-mutation messages so the caller can check ownership.
fn send_mix_index(msg: &ClientMessage) -> Option<u8> {
    match msg {
        ClientMessage::SetSendGain { mix_index, .. }
        | ClientMessage::SetSendPan { mix_index, .. }
        | ClientMessage::SetSendMuted { mix_index, .. } => Some(*mix_index),
        _ => None,
    }
}

/// Returns the `mix_index` for master-mutation messages.
fn master_mix_index(msg: &ClientMessage) -> Option<u8> {
    match msg {
        ClientMessage::SetMasterGain { mix_index, .. }
        | ClientMessage::SetMasterMute { mix_index, .. } => Some(*mix_index),
        _ => None,
    }
}

/// Returns `true` when the message is an EQ band mutation (Engineer/Admin only).
fn is_eq_band_mutation(msg: &ClientMessage) -> bool {
    matches!(msg, ClientMessage::SetEqBand { .. })
}

fn public_protocol_error(error: &ProtocolError) -> (&'static str, &'static str) {
    match error {
        ProtocolError::InvalidJson(_) => ("INVALID_JSON", "invalid WebSocket message"),
        ProtocolError::UnsupportedVersion(_) => {
            ("UNSUPPORTED_VERSION", "unsupported protocol version")
        }
        ProtocolError::InvalidRequestId => ("INVALID_REQUEST_ID", "invalid request id"),
        ProtocolError::MessageTooLarge => ("MESSAGE_TOO_LARGE", "message exceeds protocol limit"),
    }
}

async fn send_error(socket: &mut WebSocket, code: &str, message: &str) {
    send_error_with_request(socket, "server", code, message).await;
}

async fn send_error_with_request(
    socket: &mut WebSocket,
    request_id: &str,
    code: &str,
    message: &str,
) {
    let envelope = Envelope {
        version: PROTOCOL_VERSION,
        request_id: request_id.to_owned(),
        payload: ServerMessage::Error {
            code: code.to_owned(),
            message: message.to_owned(),
        },
    };
    if let Ok(json) = serde_json::to_string(&envelope) {
        let _ = send_with_timeout(socket, Message::Text(json.into())).await;
    }
}

#[cfg(test)]
mod tests {
    use super::{public_protocol_error, KeepaliveTracker, KEEPALIVE_TIMEOUT};
    use control_protocol::ProtocolError;
    use tokio::time::{advance, interval_at, Duration, Instant, MissedTickBehavior};

    #[tokio::test(start_paused = true)]
    async fn keepalive_loop_policy_is_deterministic_at_interval_boundaries() {
        let mut interval = interval_at(
            Instant::now() + Duration::from_secs(30),
            Duration::from_secs(30),
        );
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut tracker = KeepaliveTracker::default();
        let sent_at = Instant::now();

        advance(Duration::from_secs(30)).await;
        interval.tick().await;
        assert!(tracker.should_send_ping());
        tracker.record_ping(b"challenge".to_vec(), sent_at + Duration::from_secs(30));

        advance(Duration::from_secs(29)).await;
        assert!(!tracker.expired(sent_at + Duration::from_secs(59)));
        advance(Duration::from_secs(1)).await;
        assert!(!tracker.expired(sent_at + KEEPALIVE_TIMEOUT));
        assert!(tracker.expired(sent_at + Duration::from_secs(90)));

        advance(Duration::from_secs(30)).await;
        interval.tick().await;
        assert!(!tracker.should_send_ping());
    }

    #[test]
    fn keepalive_tracker_covers_timeout_and_correlated_pong() {
        let sent_at = Instant::now();
        let mut tracker = KeepaliveTracker::default();
        assert!(tracker.should_send_ping());
        tracker.record_ping(b"expected".to_vec(), sent_at);
        assert!(!tracker.should_send_ping());
        assert!(!tracker.expired(sent_at + KEEPALIVE_TIMEOUT - Duration::from_millis(1)));
        assert!(tracker.expired(sent_at + KEEPALIVE_TIMEOUT));
        assert!(!tracker.receive_pong(b"wrong", sent_at));
        assert!(tracker.expired(sent_at + KEEPALIVE_TIMEOUT));
        assert!(tracker.receive_pong(b"expected", sent_at));
        assert!(!tracker.receive_pong(b"expected", sent_at));
        assert!(tracker.should_send_ping());
        assert!(!tracker.expired(sent_at + KEEPALIVE_TIMEOUT));
    }

    #[test]
    fn protocol_errors_expose_stable_public_messages_only() {
        let (code, message) = public_protocol_error(&ProtocolError::InvalidJson(
            serde_json::from_str::<serde_json::Value>("{secret-token").unwrap_err(),
        ));
        assert_eq!(
            (code, message),
            ("INVALID_JSON", "invalid WebSocket message")
        );

        let (code, message) = public_protocol_error(&ProtocolError::UnsupportedVersion(99));
        assert_eq!(
            (code, message),
            ("UNSUPPORTED_VERSION", "unsupported protocol version")
        );
    }
}
