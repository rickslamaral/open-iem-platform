//! WebSocket handler: `/ws/v1`
//!
//! Authenticates via `Authorization: Bearer <token>` header during HTTP upgrade.
//! Each connection dispatches `ClientMessage` frames and receives `ServerMessage` responses.
//! Connection is closed on:
//!   - Auth failure
//!   - Malformed message
//!   - Message too large
//!   - Server shutdown

#![allow(clippy::unused_async)]

use crate::{auth::JwtClaims, state::AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use control_protocol::{
    decode_client_message, ClientMessage, Envelope, Role, ServerMessage, MAX_MESSAGE_BYTES,
    PROTOCOL_VERSION,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
/// Auth via `Authorization: Bearer <token>` header on the initial HTTP upgrade request.
/// Rejects with 401 if token is missing or invalid.
pub async fn ws_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims))
}

/// Long-running WebSocket I/O loop. The loop body is intentionally contained here
/// to keep the borrow checker happy with `socket` across await points.
#[allow(clippy::too_many_lines)]
async fn handle_socket(mut socket: WebSocket, state: AppState, claims: JwtClaims) {
    debug!(user = %claims.sub, role = ?claims.role, "WebSocket connected");
    let mut window_started = unix_now();
    let mut message_count = 0_u32;

    loop {
        let now = unix_now();
        if now >= claims.exp {
            send_error(&mut socket, "TOKEN_EXPIRED", "access token expired").await;
            break;
        }
        let remaining = claims.exp - now;
        let msg = match tokio::time::timeout(Duration::from_secs(remaining), socket.recv()).await {
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

                // Musician ownership check for send mutations.
                if claims.role == Role::Musician {
                    if let Some(mix_index) = send_mix_index(&envelope.payload) {
                        let assigned = state
                            .db
                            .get_user_assigned_mix(claims.user_id)
                            .unwrap_or(None);
                        if assigned != Some(usize::from(mix_index)) {
                            send_error(&mut socket, "FORBIDDEN", "musician does not own this mix")
                                .await;
                            continue;
                        }
                    }
                }

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
                    send_error(&mut socket, "INTERNAL_ERROR", "state lock poisoned").await;
                    break;
                };

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
        // Musician: cannot mutate channel gain/mute directly.
        (
            Role::Musician,
            ClientMessage::SetChannelGain { .. } | ClientMessage::SetChannelMute { .. },
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
