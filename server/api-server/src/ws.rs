//! WebSocket handler: `/ws/v1`
//!
//! Authenticates via `Authorization: Bearer` header during HTTP upgrade.
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
use control_protocol::{decode_client_message, Envelope, ServerMessage, PROTOCOL_VERSION};
use tracing::{debug, warn};

/// Maximum WebSocket message size in bytes (16 KiB).
const MAX_WS_MESSAGE_BYTES: usize = 16 * 1024;

/// `/ws/v1` WebSocket upgrade handler.
///
/// Auth via `Authorization: Bearer` header on the initial HTTP upgrade request.
/// Rejects with 401 if token is missing or invalid.
pub async fn ws_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, claims: JwtClaims) {
    debug!(user = %claims.sub, role = ?claims.role, "WebSocket connected");

    loop {
        let msg = match socket.recv().await {
            Some(Ok(m)) => m,
            Some(Err(e)) => {
                warn!("WebSocket recv error: {e}");
                break;
            }
            None => break,
        };

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
