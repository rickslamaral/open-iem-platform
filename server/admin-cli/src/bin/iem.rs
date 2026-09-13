//! Open IEM Platform developer CLI.
//!
//! This binary is a typed dispatcher for the repository Makefile. It does not
//! replace Makefile semantics or provide an installation package.

use clap::{Parser, Subcommand};
use std::process::{self, Command};

#[derive(Debug, Parser)]
#[command(
    name = "iem",
    about = "Open IEM Platform developer CLI",
    version,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: CommandName,
}

#[derive(Debug, Clone, Copy, Subcommand, PartialEq, Eq)]
enum CommandName {
    /// Show available developer commands.
    Help,
    /// Report repository and build state.
    Status,
    /// Report environment validation state.
    Diagnostics,
    /// Validate project documentation and skills.
    Docs,
    /// Run Rust and frontend tests.
    Test,
    /// Build Rust and frontend artifacts.
    Build,
    /// Start project-owned Compose services.
    Up,
    /// Stop project-owned Compose services.
    Down,
}

impl CommandName {
    const fn make_target(self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Status => "status",
            Self::Diagnostics => "diagnostics",
            Self::Docs => "docs",
            Self::Test => "test",
            Self::Build => "build",
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let target = cli.command.make_target();
    let status = Command::new("make")
        .arg(target)
        .status()
        .unwrap_or_else(|error| {
            eprintln!("iem: failed to execute make {target}: {error}");
            process::exit(127);
        });

    process::exit(status.code().unwrap_or(1));
}

#[cfg(test)]
mod tests {
    use super::CommandName;

    #[test]
    fn every_public_command_maps_to_documented_make_target() {
        let mappings = [
            (CommandName::Help, "help"),
            (CommandName::Status, "status"),
            (CommandName::Diagnostics, "diagnostics"),
            (CommandName::Docs, "docs"),
            (CommandName::Test, "test"),
            (CommandName::Build, "build"),
            (CommandName::Up, "up"),
            (CommandName::Down, "down"),
        ];

        for (command, target) in mappings {
            assert_eq!(command.make_target(), target);
        }
    }
}
