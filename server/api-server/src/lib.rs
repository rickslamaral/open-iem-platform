//! Open IEM Platform — API Server
//!
//! Axum HTTP/WebSocket server for the control plane.
//! Phase 3: REST `/api/v1`, WebSocket `/ws/v1`, JWT auth, SQLite user store.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod auth;
pub mod db;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod security;
pub mod state;
pub mod ws;
