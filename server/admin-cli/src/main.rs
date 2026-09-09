//! Open IEM Platform — Admin CLI
//!
//! Admin utility for managing users and sessions via the API server.
//!
//! # Usage
//!
//! ```text
//! open-iem-admin --server http://localhost:8080 --token <JWT> user list
//! open-iem-admin --server http://localhost:8080 --token <JWT> user create --name alice --role engineer
//! open-iem-admin --server http://localhost:8080 --token <JWT> user delete --id 42
//! open-iem-admin --server http://localhost:8080 --token <JWT> session list
//! open-iem-admin --server http://localhost:8080 --token <JWT> session revoke --token <session_token>
//! open-iem-admin --server http://localhost:8080 health
//! ```
//!
//! Token can also be supplied via the `OPEN_IEM_ADMIN_TOKEN` env var.

use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde_json::Value;
use std::process;

// ---------------------------------------------------------------------------
// CLI structure
// ---------------------------------------------------------------------------

/// Open IEM Platform admin CLI.
#[derive(Debug, Parser)]
#[command(
    name = "open-iem-admin",
    about = "Open IEM Platform admin CLI",
    version
)]
struct Cli {
    /// API server base URL (e.g. `<http://localhost:8080>`).
    #[arg(long, env = "OPEN_IEM_SERVER", default_value = "http://localhost:8080")]
    server: String,

    /// JWT bearer token for authentication.
    /// Falls back to `OPEN_IEM_ADMIN_TOKEN` env var.
    #[arg(long, env = "OPEN_IEM_ADMIN_TOKEN")]
    token: Option<String>,

    /// Output raw JSON instead of formatted tables.
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// User management commands.
    User {
        #[command(subcommand)]
        action: UserCommands,
    },
    /// Session management commands.
    Session {
        #[command(subcommand)]
        action: SessionCommands,
    },
    /// Check server health.
    Health,
}

#[derive(Debug, Subcommand)]
enum UserCommands {
    /// List all users.
    List,
    /// Create a new user.
    Create {
        /// Display name for the new user.
        #[arg(long)]
        name: String,
        /// Role for the new user (e.g. engineer, musician, admin).
        #[arg(long)]
        role: String,
    },
    /// Delete a user by ID.
    Delete {
        /// User ID to delete.
        #[arg(long)]
        id: u64,
    },
}

#[derive(Debug, Subcommand)]
enum SessionCommands {
    /// List active sessions.
    List,
    /// Revoke an active session by token.
    Revoke {
        /// Session token to revoke.
        #[arg(long)]
        token: String,
    },
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

struct AdminClient {
    base: String,
    token: Option<String>,
    client: Client,
    json_output: bool,
}

impl AdminClient {
    fn new(base: String, token: Option<String>, json_output: bool) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self {
            base,
            token,
            client,
            json_output,
        }
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_deref().map(|t| format!("Bearer {t}"))
    }

    fn get(&self, path: &str) -> Result<Value, String> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().map_err(|e| format!("request failed: {e}"))?;
        Self::handle_response(resp)
    }

    fn post(&self, path: &str, body: &Value) -> Result<Value, String> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.client.post(&url).json(body);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().map_err(|e| format!("request failed: {e}"))?;
        Self::handle_response(resp)
    }

    fn delete(&self, path: &str) -> Result<Value, String> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.client.delete(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().map_err(|e| format!("request failed: {e}"))?;
        Self::handle_response(resp)
    }

    fn handle_response(resp: reqwest::blocking::Response) -> Result<Value, String> {
        let status = resp.status();
        match status.as_u16() {
            200..=299 => {
                let body: Value = resp.json().unwrap_or(Value::Object(serde_json::Map::new()));
                Ok(body)
            }
            401 | 403 => Err(format!(
                "Authentication error (HTTP {status}): check --token or OPEN_IEM_ADMIN_TOKEN"
            )),
            404 => Err(
                "Not yet implemented on server (HTTP 404): this endpoint is planned for Phase 9"
                    .to_string(),
            ),
            _ => {
                let body: Value = resp
                    .json()
                    .unwrap_or(Value::String(format!("HTTP {status}")));
                Err(format!("Server error (HTTP {status}): {body}"))
            }
        }
    }

    fn print(&self, value: &Value) {
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_default()
            );
        } else {
            print_table(value);
        }
    }
}

// ---------------------------------------------------------------------------
// Table rendering
// ---------------------------------------------------------------------------

fn print_table(value: &Value) {
    match value {
        Value::Array(rows) => {
            if rows.is_empty() {
                println!("(no results)");
                return;
            }
            // Collect all keys from first object
            if let Some(Value::Object(first)) = rows.first() {
                let keys: Vec<&str> = first.keys().map(String::as_str).collect();
                // Header
                let header: Vec<String> = keys.iter().map(|k| k.to_uppercase()).collect();
                println!("{}", header.join("  |  "));
                println!("{}", "-".repeat(header.join("  |  ").len()));
                // Rows
                for row in rows {
                    if let Value::Object(map) = row {
                        let vals: Vec<String> = keys
                            .iter()
                            .map(|k| {
                                map.get(*k).map_or(String::new(), |v| match v {
                                    Value::String(s) => s.clone(),
                                    other => other.to_string(),
                                })
                            })
                            .collect();
                        println!("{}", vals.join("  |  "));
                    }
                }
            } else {
                // Non-object array
                for item in rows {
                    println!("{item}");
                }
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                match v {
                    Value::String(s) => println!("{k}: {s}"),
                    other => println!("{k}: {other}"),
                }
            }
        }
        other => println!("{other}"),
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();
    let admin = AdminClient::new(cli.server, cli.token, cli.json);

    let result = match cli.command {
        Commands::Health => admin.get("/api/v1/health"),
        Commands::User { action } => match action {
            UserCommands::List => admin.get("/api/v1/admin/users"),
            UserCommands::Create { name, role } => admin.post(
                "/api/v1/admin/users",
                &serde_json::json!({ "name": name, "role": role }),
            ),
            UserCommands::Delete { id } => admin.delete(&format!("/api/v1/admin/users/{id}")),
        },
        Commands::Session { action } => match action {
            SessionCommands::List => admin.get("/api/v1/admin/sessions"),
            SessionCommands::Revoke { token } => {
                admin.delete(&format!("/api/v1/admin/sessions/{token}"))
            }
        },
    };

    match result {
        Ok(value) => {
            admin.print(&value);
        }
        Err(msg) => {
            eprintln!("Error: {msg}");
            process::exit(1);
        }
    }
}
