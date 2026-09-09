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
    response::IntoResponse,
};
use control_protocol::{
    decode_client_message, ClientMessage, Envelope, Role, ServerMessage, MAX_MESSAGE_BYTES,
    PROTOCOL_VERSION,
};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

const MAX_MESSAGES_PER_MINUTE: u32 = 120;

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
use tracing::{debug, warn};

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
) -> impl IntoResponse {
    ws.protocols(["openiem.v1"]).on_upgrade(move |socket| {
        let session_id = u128::from(NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed));
        handle_socket(socket, state, claims, session_id)
    })
}

/// Long-running WebSocket I/O loop. The loop body is intentionally contained here
/// to keep the borrow checker happy with `socket` across await points.
#[allow(clippy::too_many_lines)]
async fn handle_socket(
    mut socket: WebSocket,
    state: AppState,
    claims: JwtClaims,
    session_id: u128,
) {
    debug!(user = %claims.sub, role = ?claims.role, "WebSocket connected");
    let mut window_started = unix_now();
    let mut message_count = 0_u32;

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
            // Inbound message from client.
            recv = tokio::time::timeout(Duration::from_secs(remaining), socket.recv()) => {
                let msg = match recv {
                    Ok(Some(Ok(m))) => m,
                    Ok(Some(Err(e))) => {
                        warn!("WebSocket recv error: {e}");
                        break;
                    }
                    Ok(None) | Err(_) => {
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
                            Err(e) => {
                                send_error(&mut socket, "PROTOCOL_ERROR", &e.to_string()).await;
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
                                state
                                    .db
                                    .get_user_assigned_mix(claims.user_id)
                                    .unwrap_or(None)
                                    == Some(usize::from(mix_index))
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

                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => {
                        debug!(user = %claims.sub, "WebSocket close frame");
                        break;
                    }
                    Message::Ping(payload) => {
                        let _ = socket.send(Message::Pong(payload)).await;
                    }
                    _ => {}
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
                            Role::Musician => {
                                let assigned = state
                                    .db
                                    .get_user_assigned_mix(claims.user_id)
                                    .unwrap_or(None);
                                assigned == Some(usize::from(delta.mix_index))
                            }
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
                            if socket.send(Message::Text(json.into())).await.is_err() {
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
                            Role::Musician => {
                                let assigned = state
                                    .db
                                    .get_user_assigned_mix(claims.user_id)
                                    .unwrap_or(None);
                                assigned == Some(usize::from(delta.mix_index))
                            }
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
                            if socket.send(Message::Text(json.into())).await.is_err() {
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
        let _ = socket.send(Message::Text(json.into())).await;
    }
}
