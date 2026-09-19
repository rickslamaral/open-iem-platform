//! Open IEM Platform developer CLI.
//!
//! This binary is a typed dispatcher for the repository Makefile and provides
//! local configuration snapshot file operations. It does not replace Makefile
//! semantics or provide an installation package.

use clap::{Parser, Subcommand};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{self, Command},
};

const MAX_CONFIG_SNAPSHOT_BYTES: u64 = 1024 * 1024;

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

#[derive(Debug, Subcommand, PartialEq, Eq)]
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
    /// Read or write a local control-plane configuration snapshot.
    Config {
        #[command(subcommand)]
        action: ConfigCommand,
    },
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
enum ConfigCommand {
    /// Write current local default configuration snapshot as JSON.
    Backup {
        /// Destination JSON file.
        #[arg(long, value_name = "PATH")]
        output: PathBuf,
    },
    /// Validate and read a local configuration snapshot JSON file.
    Restore {
        /// Source JSON file.
        #[arg(long, value_name = "PATH")]
        input: PathBuf,
    },
}

impl CommandName {
    fn make_target(&self) -> Option<&'static str> {
        match self {
            Self::Help => Some("help"),
            Self::Status => Some("status"),
            Self::Diagnostics => Some("diagnostics"),
            Self::Docs => Some("docs"),
            Self::Test => Some("test"),
            Self::Build => Some("build"),
            Self::Up => Some("up"),
            Self::Down => Some("down"),
            Self::Config { .. } => None,
        }
    }
}

fn reject_symlink_components(path: &std::path::Path) -> Result<(), String> {
    let mut current = if path.is_absolute() {
        PathBuf::from(std::path::MAIN_SEPARATOR.to_string())
    } else {
        std::env::current_dir()
            .map_err(|error| format!("iem: failed to resolve current directory: {error}"))?
    };
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => current.push(prefix.as_os_str()),
            std::path::Component::RootDir | std::path::Component::CurDir => {}
            std::path::Component::ParentDir => current.push(component.as_os_str()),
            std::path::Component::Normal(name) => {
                current.push(name);
                if current
                    .symlink_metadata()
                    .is_ok_and(|metadata| metadata.file_type().is_symlink())
                {
                    return Err(format!(
                        "iem: refusing path through symlink {}",
                        current.display()
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn sync_parent_directory(path: &std::path::Path) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("iem: failed to sync {}: {error}", parent.display()))
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &PathBuf) -> Result<(), String> {
    Ok(())
}

fn write_snapshot_atomically(output: &PathBuf, contents: &str) -> Result<(), String> {
    reject_symlink_components(output)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("iem: failed to read system clock: {error}"))?
        .as_nanos();
    let file_name = output
        .file_name()
        .ok_or_else(|| format!("iem: output has no file name: {}", output.display()))?
        .to_string_lossy();
    let mut file = None;
    let mut temporary = PathBuf::new();
    for attempt in 0..16 {
        temporary = output.with_file_name(format!(
            ".{file_name}.tmp-{}-{stamp}-{attempt}",
            process::id()
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(candidate) => {
                file = Some(candidate);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(format!(
                    "iem: failed to create {}: {error}",
                    temporary.display()
                ));
            }
        }
    }
    let mut file = file.ok_or_else(|| {
        format!(
            "iem: failed to create unique temporary file for {}",
            output.display()
        )
    })?;
    if let Err(error) = file
        .write_all(contents.as_bytes())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "iem: failed to write {}: {error}",
            output.display()
        ));
    }
    drop(file);
    if let Err(error) = fs::rename(&temporary, output) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "iem: failed to write {}: {error}",
            output.display()
        ));
    }
    sync_parent_directory(output)?;
    Ok(())
}

#[cfg(unix)]
fn read_snapshot(path: &PathBuf) -> Result<String, String> {
    use std::os::unix::fs::OpenOptionsExt;
    reject_symlink_components(path)?;
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| format!("iem: failed to read {}: {error}", path.display()))?;
    let size = file
        .metadata()
        .map_err(|error| format!("iem: failed to inspect {}: {error}", path.display()))?
        .len();
    if size > MAX_CONFIG_SNAPSHOT_BYTES {
        return Err(format!(
            "iem: configuration snapshot {} exceeds {} bytes",
            path.display(),
            MAX_CONFIG_SNAPSHOT_BYTES
        ));
    }
    let mut contents = String::new();
    file.take(MAX_CONFIG_SNAPSHOT_BYTES + 1)
        .read_to_string(&mut contents)
        .map_err(|error| format!("iem: failed to read {}: {error}", path.display()))?;
    if contents.len() as u64 > MAX_CONFIG_SNAPSHOT_BYTES {
        return Err(format!(
            "iem: configuration snapshot {} exceeds {} bytes",
            path.display(),
            MAX_CONFIG_SNAPSHOT_BYTES
        ));
    }
    Ok(contents)
}

#[cfg(not(unix))]
fn read_snapshot(path: &PathBuf) -> Result<String, String> {
    reject_symlink_components(path)?;
    let metadata = fs::metadata(path)
        .map_err(|error| format!("iem: failed to inspect {}: {error}", path.display()))?;
    if metadata.len() > MAX_CONFIG_SNAPSHOT_BYTES {
        return Err(format!(
            "iem: configuration snapshot {} exceeds {} bytes",
            path.display(),
            MAX_CONFIG_SNAPSHOT_BYTES
        ));
    }
    let mut file = fs::File::open(path)
        .map_err(|error| format!("iem: failed to read {}: {error}", path.display()))?;
    let mut contents = String::new();
    file.take(MAX_CONFIG_SNAPSHOT_BYTES + 1)
        .read_to_string(&mut contents)
        .map_err(|error| format!("iem: failed to read {}: {error}", path.display()))?;
    if contents.len() as u64 > MAX_CONFIG_SNAPSHOT_BYTES {
        return Err(format!(
            "iem: configuration snapshot {} exceeds {} bytes",
            path.display(),
            MAX_CONFIG_SNAPSHOT_BYTES
        ));
    }
    Ok(contents)
}

fn run_config(action: ConfigCommand) -> Result<(), String> {
    match action {
        ConfigCommand::Backup { output } => {
            if output
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                return Err(format!(
                    "iem: refusing to overwrite symlink {}",
                    output.display()
                ));
            }
            let snapshot = config_backup::backup(&control_server::ControlState::new());
            let json = config_backup::serialize(&snapshot).map_err(|error| error.to_string())?;
            write_snapshot_atomically(&output, &format!("{json}\n"))?;
            println!("wrote configuration snapshot to {}", output.display());
        }
        ConfigCommand::Restore { input } => {
            if input
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                return Err(format!("iem: refusing to read symlink {}", input.display()));
            }
            let json = read_snapshot(&input)?;
            let snapshot = config_backup::deserialize(&json).map_err(|error| error.to_string())?;
            let mut state = control_server::ControlState::new();
            config_backup::restore(&snapshot, &mut state).map_err(|error| error.to_string())?;
            println!(
                "validated local configuration snapshot from {}; no running state changed",
                input.display()
            );
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if let CommandName::Config { action } = cli.command {
        if let Err(error) = run_config(action) {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    let target = cli.command.make_target().expect("config handled above");
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
    use super::{run_config, Cli, CommandName, ConfigCommand, MAX_CONFIG_SNAPSHOT_BYTES};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn parses_config_backup_output() {
        let cli = Cli::try_parse_from(["iem", "config", "backup", "--output", "snapshot.json"])
            .expect("valid backup command");
        assert_eq!(
            cli.command,
            CommandName::Config {
                action: ConfigCommand::Backup {
                    output: PathBuf::from("snapshot.json")
                }
            }
        );
    }

    #[test]
    fn parses_config_restore_input() {
        let cli = Cli::try_parse_from(["iem", "config", "restore", "--input", "snapshot.json"])
            .expect("valid restore command");
        assert_eq!(
            cli.command,
            CommandName::Config {
                action: ConfigCommand::Restore {
                    input: PathBuf::from("snapshot.json")
                }
            }
        );
    }

    #[test]
    fn backup_and_restore_use_local_json_file() {
        let path = std::env::temp_dir().join(format!(
            "iem-config-test-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));

        run_config(ConfigCommand::Backup {
            output: path.clone(),
        })
        .expect("backup succeeds");
        let contents = std::fs::read_to_string(&path).expect("snapshot exists");
        assert!(contents.contains("\"version\": 1"));
        run_config(ConfigCommand::Restore {
            input: path.clone(),
        })
        .expect("restore succeeds");
        std::fs::remove_file(path).expect("remove snapshot");
    }

    #[test]
    fn config_commands_do_not_dispatch_make_targets() {
        assert!(CommandName::Config {
            action: ConfigCommand::Backup {
                output: PathBuf::from("out.json")
            }
        }
        .make_target()
        .is_none());
    }

    #[test]
    fn restore_rejects_oversized_snapshot() {
        let path = std::env::temp_dir().join(format!(
            "iem-config-large-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        std::fs::write(&path, vec![b' '; (MAX_CONFIG_SNAPSHOT_BYTES + 1) as usize])
            .expect("large snapshot exists");
        let error = run_config(ConfigCommand::Restore {
            input: path.clone(),
        })
        .expect_err("oversized snapshot rejected");
        assert!(error.contains("exceeds"));
        std::fs::remove_file(path).expect("large snapshot removed");
    }

    #[cfg(unix)]
    #[test]
    fn backup_refuses_symlink_output() {
        let directory = std::env::temp_dir().join(format!(
            "iem-config-symlink-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        std::fs::create_dir(&directory).expect("temporary directory");
        let target = directory.join("target.json");
        std::fs::write(&target, "untouched").expect("target exists");
        let link = directory.join("link.json");
        std::os::unix::fs::symlink(&target, &link).expect("symlink exists");

        let error =
            run_config(ConfigCommand::Backup { output: link }).expect_err("symlink rejected");
        assert!(error.contains("refusing to overwrite symlink"));
        assert_eq!(
            std::fs::read_to_string(target).expect("target readable"),
            "untouched"
        );
        std::fs::remove_dir_all(directory).expect("temporary directory removed");
    }
}
