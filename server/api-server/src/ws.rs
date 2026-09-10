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
//! excess upgrades receive HTTP 503 with `Retry-After: 5`.
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
    state::{AppState, MasterDelta, SendDelta},
};
use axum::{
    extract::{
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
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::Instant;

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

const MAX_MESSAGES_PER_MINUTE: u32 = 120;
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_SOCKET_SEND_WAIT: Duration = Duration::from_secs(10);

fn keepalive_expired(last_pong: Instant, now: Instant) -> bool {
    now.saturating_duration_since(last_pong) >= KEEPALIVE_TIMEOUT
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
    ws: WebSocketUpgrade,
    Extension(claims): Extension<JwtClaims>,
) -> Response {
    let Ok(permit) = state.websocket_connections.clone().try_acquire_owned() else {
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            [(axum::http::header::RETRY_AFTER, "5")],
            "WebSocket connection limit reached",
        )
            .into_response();
    };
    ws.protocols(["openiem.v1"])
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
    _connection_permit: OwnedSemaphorePermit,
) {
    debug!(user = %claims.sub, role = ?claims.role, "WebSocket connected");
    let mut window_started = unix_now();
    let mut message_count = 0_u32;
    let mut keepalive = tokio::time::interval_at(
        tokio::time::Instant::now() + KEEPALIVE_INTERVAL,
        KEEPALIVE_INTERVAL,
    );
    keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last_pong = Instant::now();

    // Subscribe to broadcast events before entering the loop.
    let mut event_rx = state.event_tx.subscribe();
    let mut master_event_rx = state.master_event_tx.subscribe();

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
                if keepalive_expired(last_pong, Instant::now()) {
                    send_error(&mut socket, "CONNECTION_TIMEOUT", "WebSocket pong timeout").await;
                    break;
                }
                if !send_with_timeout(&mut socket, Message::Ping(Vec::new().into())).await {
                    break;
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
                                send_error(&mut socket, code, message).await;
                                break;
                            }
                            Ok(env) => env,
                        };

                        // Role-based permission check — also covers send mutations.
                        let permitted = check_permission(&claims, &envelope.payload);
                        if !permitted {
                            send_error(&mut socket, "FORBIDDEN", "role cannot perform this action").await;
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
                            send_error(&mut socket, "FORBIDDEN", "musician does not own this mix")
                                .await;
                            continue;
                        }

                        // Remember whether this is a send mutation before consuming the envelope.
                        let is_send_mutation = send_mix_index(&envelope.payload).is_some();
                        let is_master_mutation = master_mix_index(&envelope.payload).is_some();

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
                    Message::Pong(_) => {
                        last_pong = Instant::now();
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

                        // Coordinate assignment read with assignment mutations. Publication
                        // order is serialized by the sender's lock; release before socket I/O.
                        let assignment_guard = state.mix_assignment_lock.lock().await;
                        // Filter by role:
                        //  - Engineer/Admin see deltas for all mixes.
                        //  - Musician sees deltas only for their assigned mix.
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
                        drop(assignment_guard);
                        if let Some(json) = ack_json {
                            if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(user = %claims.sub, "broadcast lagged by {n} events; some deltas skipped");
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

                        let assignment_guard = state.mix_assignment_lock.lock().await;
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
                        drop(assignment_guard);
                        if let Some(json) = ack_json {
                            if !send_with_timeout(&mut socket, Message::Text(json.into())).await {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(user = %claims.sub, "master broadcast lagged by {n} events; some deltas skipped");
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
        // Musician: cannot mutate channel gain/mute directly or master gain/mute.
        (
            Role::Musician,
            ClientMessage::SetChannelGain { .. }
            | ClientMessage::SetChannelMute { .. }
            | ClientMessage::SetMasterGain { .. }
            | ClientMessage::SetMasterMute { .. },
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
    let envelope = Envelope {
        version: PROTOCOL_VERSION,
        request_id: "server".to_owned(),
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
    use super::{keepalive_expired, public_protocol_error, KEEPALIVE_TIMEOUT};
    use control_protocol::ProtocolError;
    use tokio::time::{Duration, Instant};

    #[test]
    fn keepalive_is_not_expired_before_timeout() {
        let last_pong = Instant::now();
        assert!(!keepalive_expired(
            last_pong,
            last_pong + KEEPALIVE_TIMEOUT - Duration::from_millis(1),
        ));
    }

    #[test]
    fn keepalive_expires_at_timeout() {
        let last_pong = Instant::now();
        assert!(keepalive_expired(last_pong, last_pong + KEEPALIVE_TIMEOUT));
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
