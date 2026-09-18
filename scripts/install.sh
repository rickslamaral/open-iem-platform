#!/usr/bin/env bash
# Open IEM Platform installer — Linux source installer.
# Usage: curl -fsSL https://raw.githubusercontent.com/OWNER/REPO/main/scripts/install.sh | bash
set -Eeuo pipefail
IFS=$'\n\t'

REPO_URL="${OPENIEM_REPO_URL:-https://github.com/rickslamaral/open-iem-platform.git}"
# Installer must build immutable, explicitly selected source. Mutable branches are unsafe.
REF="${OPENIEM_REF:-}"
PREFIX="${OPENIEM_PREFIX:-/opt/openiem}"
BIN_DIR="${OPENIEM_BIN_DIR:-/usr/local/bin}"
STATE_DIR="${OPENIEM_STATE_DIR:-/var/lib/openiem}"
CONFIG_DIR="${OPENIEM_CONFIG_DIR:-/etc/openiem}"
SERVICE_NAME="openiem-server.service"
SOUNDTECH_ENV="$CONFIG_DIR/openiem-server.env"
SERVICE_DROPIN_DIR="/etc/systemd/system/$SERVICE_NAME.d"
DRY_RUN=0
VALIDATE_ONLY=0
VALIDATION_REPORT=""
SKIP_DEPS=0
MIN_RUSTC_MAJOR=1
MIN_RUSTC_MINOR=88
MIN_NODE_MAJOR=20
NO_SERVICE=0
KEEP_SOURCE=0
ROTATE_KEYS=0
ASSUME_YES=0
RUN_TESTS=0
TEST_REPORT=""

log() { printf '[open-iem] %s\n' "$*"; }
warn() { printf '[open-iem] WARNING: %s\n' "$*" >&2; }
fatal() { printf '[open-iem] ERROR: %s\n' "$*" >&2; exit 1; }
run() { if (( DRY_RUN )); then printf '+ %q' "$1"; shift; printf ' %q' "$@"; printf '\n'; else "$@"; fi; }
need_cmd() { command -v "$1" >/dev/null 2>&1 || fatal "missing command: $1"; }

usage() {
  cat <<'EOF'
Open IEM Platform Linux installer

Options:
  --repo URL       Git repository URL
  --ref SHA        Required full 40-character Git commit SHA (immutable source pin)
  --prefix PATH    Source/install prefix (default: /opt/openiem)
  --dry-run        Show actions without changing host
  --validate-only  Validate host and installed release; write report, no build/install
  --skip-deps      Do not install/check OS packages
  --no-service     Do not install systemd unit
  --keep-source    Keep temporary checkout after installation
  --rotate-keys    Replace existing JWT keys after explicit confirmation
  --yes            Confirm destructive actions (use with --rotate-keys)
  --run-tests      Run complete headless/audio test suite after install
  --test-report PATH  Write test summary report (default: /tmp/openiem-tests-<ref>.txt)
  -h, --help       Show help

Environment equivalents:
  OPENIEM_REPO_URL OPENIEM_REF OPENIEM_PREFIX OPENIEM_BIN_DIR
  OPENIEM_STATE_DIR OPENIEM_CONFIG_DIR
EOF
}

while (($#)); do
  case "$1" in
    --repo) REPO_URL="${2:?missing URL}"; shift 2 ;;
    --ref) REF="${2:?missing ref}"; shift 2 ;;
    --prefix) PREFIX="${2:?missing prefix}"; shift 2 ;;
    --bin-dir) BIN_DIR="${2:?missing bin dir}"; shift 2 ;;
    --state-dir) STATE_DIR="${2:?missing state dir}"; shift 2 ;;
    --config-dir) CONFIG_DIR="${2:?missing config dir}"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    --validate-only) VALIDATE_ONLY=1; shift ;;
    --validation-report) VALIDATION_REPORT="${2:?missing report path}"; shift 2 ;;
    --skip-deps) SKIP_DEPS=1; shift ;;
    --no-service) NO_SERVICE=1; shift ;;
    --keep-source) KEEP_SOURCE=1; shift ;;
    --rotate-keys) ROTATE_KEYS=1; shift ;;
    --yes) ASSUME_YES=1; shift ;;
    --run-tests) RUN_TESTS=1; shift ;;
    --test-report) TEST_REPORT="${2:?missing test report path}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) fatal "unknown option: $1 (use --help)" ;;
  esac
done

[[ "$(uname -s)" == Linux ]] || fatal "Linux required"
[[ "$PREFIX" = /* && "$BIN_DIR" = /* && "$STATE_DIR" = /* && "$CONFIG_DIR" = /* ]] || fatal "paths must be absolute"
validate_path() {
  local path="$1" part
  [[ "$path" =~ ^/[A-Za-z0-9._/-]*$ ]] || return 1
  [[ "$path" != */../* && "$path" != */./* && "$path" != */.. && "$path" != */. && "$path" != //* ]] || return 1
  [[ "$path" == / ]] && return 0
  while [[ "$path" == */* ]]; do
    part="${path##*/}"
    [[ "$part" != "" && "$part" != . && "$part" != .. ]] || return 1
    path="${path%/*}"
    [[ -n "$path" ]] || path=/
    [[ "$path" == / ]] && break
  done
}
validate_path "$PREFIX" && validate_path "$BIN_DIR" && validate_path "$STATE_DIR" && validate_path "$CONFIG_DIR" || fatal "paths contain unsafe characters"
[[ "$REPO_URL" == https://* || "$REPO_URL" == git@* || "$REPO_URL" == ssh://* ]] || fatal "repository URL must use HTTPS or SSH"
[[ "$REF" =~ ^[0-9a-fA-F]{40}$ ]] || fatal "--ref must be a full 40-character commit SHA; mutable branches and tags are refused"

SUDO=()
if (( EUID != 0 )); then
  command -v sudo >/dev/null 2>&1 || fatal "root or sudo required"
  SUDO=(sudo)
fi

install_deps() {
  (( SKIP_DEPS )) && { log 'skipping OS dependency installation'; return; }
  local pm=""
  if command -v apt-get >/dev/null 2>&1; then pm=apt
  elif command -v dnf >/dev/null 2>&1; then pm=dnf
  elif command -v yum >/dev/null 2>&1; then pm=yum
  elif command -v pacman >/dev/null 2>&1; then pm=pacman
  elif command -v zypper >/dev/null 2>&1; then pm=zypper
  elif command -v apk >/dev/null 2>&1; then pm=apk
  else fatal 'unsupported package manager; install git, curl, Rust, Node.js/npm, OpenSSL and a C compiler, then use --skip-deps'; fi

  case "$pm" in
    apt)
      run "${SUDO[@]}" apt-get update
      run "${SUDO[@]}" apt-get install -y --no-install-recommends git ca-certificates curl build-essential pkg-config openssl nodejs npm alsa-utils kmod python3
      ;;
    dnf|yum)
      run "${SUDO[@]}" "$pm" install -y git ca-certificates curl gcc gcc-c++ make pkgconf-pkg-config openssl openssl-devel cargo rust nodejs npm python3 alsa-utils kmod
      ;;
    pacman)
      run "${SUDO[@]}" pacman -Sy --needed --noconfirm git ca-certificates curl base-devel openssl rust nodejs npm python alsa-utils kmod
      ;;
    zypper)
      run "${SUDO[@]}" zypper --non-interactive install git ca-certificates curl gcc gcc-c++ make pkg-config libopenssl-devel rust nodejs npm python3 alsa-utils kmod
      ;;
    apk)
      run "${SUDO[@]}" apk add git ca-certificates curl build-base pkgconf openssl-dev rust cargo nodejs npm python3 alsa-utils kmod
      ;;
  esac
}

preflight_node_check() {
  # Validate an existing Node.js installation before changing the host.
  if command -v node >/dev/null 2>&1; then
    local node_major
    node_major="$(node -p 'process.versions.node.split(".")[0]')"
    (( node_major >= MIN_NODE_MAJOR )) || fatal "Node.js >= $MIN_NODE_MAJOR required; found $(node --version). Upgrade Node.js before running the installer."
  fi
}

rustc_is_compatible() {
  command -v rustc >/dev/null 2>&1 || return 1
  local version major minor
  version="$(rustc --version | awk '{print $2}')"
  major="${version%%.*}"
  minor="${version#*.}"
  minor="${minor%%.*}"
  (( major > MIN_RUSTC_MAJOR || (major == MIN_RUSTC_MAJOR && minor >= MIN_RUSTC_MINOR) ))
}

ensure_rust_toolchain() {
  rustc_is_compatible && {
    log "using existing Rust $(rustc --version)"
    return
  }

  if command -v rustc >/dev/null 2>&1; then
    warn "existing $(rustc --version) is older than rustc ${MIN_RUSTC_MAJOR}.${MIN_RUSTC_MINOR}; upgrading via rustup"
  else
    log "Rust not found; installing stable toolchain via rustup"
  fi

  if ! command -v rustup >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal
  fi
  # rustup installs per-user and does not require replacing distro packages.
  [[ -f "$HOME/.cargo/env" ]] && . "$HOME/.cargo/env"
  rustup toolchain install stable --profile minimal
  rustup default stable
  rustc_is_compatible || fatal "rustc >= ${MIN_RUSTC_MAJOR}.${MIN_RUSTC_MINOR} required after rustup installation"
  log "using upgraded Rust $(rustc --version)"
}

check_node_version() {
  need_cmd node
  local node_major
  node_major="$(node -p 'process.versions.node.split(".")[0]')"
  (( node_major >= MIN_NODE_MAJOR )) || fatal "Node.js >= $MIN_NODE_MAJOR required; found $(node --version). Install Node.js ${MIN_NODE_MAJOR}+ and retry."
}

check_tools() {
  local c
  for c in git curl openssl cargo rustc node npm timeout python3; do need_cmd "$c"; done
  rustc_is_compatible || fatal "rustc >= ${MIN_RUSTC_MAJOR}.${MIN_RUSTC_MINOR} required; found $(rustc --version)"
  check_node_version
}

atomic_symlink() {
  local target="$1" link="$2" tmp
  tmp="${link}.new.$$"
  run "${SUDO[@]}" ln -s "$target" "$tmp"
  run "${SUDO[@]}" mv -Tf "$tmp" "$link"
}

ensure_node_after_deps() {
  check_node_version
  log "using existing Node.js $(node --version)"
}

ensure_config_dir_secure() {
  local path="$CONFIG_DIR" info mode type
  local IFS=' '
  while :; do
    [[ -d "$path" && ! -L "$path" ]] || fatal "config path is not a directory or is a symlink: $path"
    info="$("${SUDO[@]}" stat -c '%u %a %F' -- "$path")" || fatal "cannot inspect config path: $path"
    read -r owner mode type <<< "$info"
    [[ "$owner" == 0 && "$type" == directory ]] || fatal "config path must be root-owned directory: $path"
    (( (8#$mode & 0022) == 0 )) || fatal "config path is writable by non-root users: $path"
    [[ "$path" == / ]] && break
    path="${path%/*}"
    [[ -n "$path" ]] || path=/
  done
}

runtime_config_is_secure() {
  local path="$CONFIG_DIR" info owner mode type env_content env_bytes env_last_byte env_stat env_owner env_mode env_type
  local IFS=' '
  while :; do
    [[ -d "$path" && ! -L "$path" ]] || return 1
    info="$("${SUDO[@]}" stat -c '%u %a %F' -- "$path")" || return 1
    read -r owner mode type <<< "$info"
    [[ "$owner" == 0 && "$type" == directory ]] || return 1
    (( (8#$mode & 0022) == 0 )) || return 1
    [[ "$path" == / ]] && break
    path="${path%/*}"
    [[ -n "$path" ]] || path=/
  done
  [[ -f "$SOUNDTECH_ENV" && ! -L "$SOUNDTECH_ENV" ]] || return 1
  env_content="$("${SUDO[@]}" cat -- "$SOUNDTECH_ENV")" || return 1
  env_bytes="$("${SUDO[@]}" stat -c '%s' -- "$SOUNDTECH_ENV")" || return 1
  (( env_bytes > 0 )) || return 1
  env_last_byte="$("${SUDO[@]}" od -An -t x1 -N 1 --skip=$((env_bytes - 1)) "$SOUNDTECH_ENV" | tr -d '[:space:]')" || return 1
  [[ "$env_last_byte" == 0a && "$env_bytes" -eq $((${#env_content} + 1)) && "$env_content" =~ ^OPENIEM_SOUNDTECH_PASSWORD=[A-Za-z0-9._~+/=-]+$ ]] || return 1
  env_stat="$("${SUDO[@]}" stat -c '%u %a %F' -- "$SOUNDTECH_ENV")" || return 1
  read -r env_owner env_mode env_type <<< "$env_stat"
  [[ "$env_owner" == 0 && "$env_mode" == 600 && "$env_type" == regular\ file ]]
}

ensure_soundtech_env() {
  local tmp env_content env_bytes env_last_byte env_stat env_owner env_mode env_type password
  local IFS=' '
  ensure_config_dir_secure
  [[ ! -L "$SOUNDTECH_ENV" ]] || fatal "SoundTech env file cannot be a symlink: $SOUNDTECH_ENV"
  if [[ -e "$SOUNDTECH_ENV" ]]; then
    [[ -f "$SOUNDTECH_ENV" && ! -L "$SOUNDTECH_ENV" ]] || fatal "SoundTech env file must be a regular file: $SOUNDTECH_ENV"
    env_content="$("${SUDO[@]}" cat -- "$SOUNDTECH_ENV")" || fatal "cannot read SoundTech env file: $SOUNDTECH_ENV"
    env_bytes="$("${SUDO[@]}" stat -c '%s' -- "$SOUNDTECH_ENV")" || fatal "cannot inspect SoundTech env file: $SOUNDTECH_ENV"
    (( env_bytes > 0 )) || fatal "unsafe SoundTech env file; expected one OPENIEM_SOUNDTECH_PASSWORD line"
    env_last_byte="$("${SUDO[@]}" od -An -t x1 -N 1 --skip=$((env_bytes - 1)) "$SOUNDTECH_ENV" | tr -d '[:space:]')" || fatal "cannot inspect SoundTech env file: $SOUNDTECH_ENV"
    [[ "$env_last_byte" == 0a && "$env_bytes" -eq $((${#env_content} + 1)) && "$env_content" =~ ^OPENIEM_SOUNDTECH_PASSWORD=[A-Za-z0-9._~+/=-]+$ ]] || fatal "unsafe SoundTech env file; expected one OPENIEM_SOUNDTECH_PASSWORD line"
    env_stat="$("${SUDO[@]}" stat -c '%u %a %F' -- "$SOUNDTECH_ENV")" || fatal "cannot inspect SoundTech env file: $SOUNDTECH_ENV"
    read -r env_owner env_mode env_type <<< "$env_stat"
    [[ "$env_owner" == 0 && "$env_mode" == 600 && "$env_type" == regular\ file ]] || fatal "SoundTech env file must be root-owned mode 0600: $SOUNDTECH_ENV"
    log 'preserving existing SoundTech password'
    return
  fi
  (( DRY_RUN )) && { log "would generate random SoundTech password in $SOUNDTECH_ENV"; return; }
  tmp="$(mktemp -t openiem-soundtech.XXXXXX)"
  umask 077
  command -v openssl >/dev/null 2>&1 || fatal "openssl required to generate SoundTech password"
  password="$(openssl rand -hex 32)" || fatal "cannot generate SoundTech password"
  [[ "$password" =~ ^[A-Za-z0-9._~+/=-]+$ ]] || fatal "generated SoundTech password is invalid"
  printf 'OPENIEM_SOUNDTECH_PASSWORD=%s\n' "$password" > "$tmp"
  run "${SUDO[@]}" install -o root -g root -m 0600 "$tmp" "$SOUNDTECH_ENV"
  rm -f -- "$tmp"
  log 'generated SoundTech password and stored it in protected env file'
}

ensure_service_dropin_dir_secure() {
  local path="$SERVICE_DROPIN_DIR" info owner mode type
  local IFS=' '
  while :; do
    [[ -d "$path" && ! -L "$path" ]] || fatal "systemd drop-in path is not a directory or is a symlink: $path"
    info="$("${SUDO[@]}" stat -c '%u %a %F' -- "$path")" || fatal "cannot inspect systemd drop-in path: $path"
    read -r owner mode type <<< "$info"
    [[ "$owner" == 0 && "$type" == directory ]] || fatal "systemd drop-in path must be root-owned directory: $path"
    (( (8#$mode & 0022) == 0 )) || fatal "systemd drop-in path is writable by non-root users: $path"
    [[ "$path" == / ]] && break
    path="${path%/*}"
    [[ -n "$path" ]] || path=/
  done
}

install_service_env_dropin() {
  local tmp="$1"
  (( DRY_RUN )) && { log "would install $SERVICE_DROPIN_DIR/override.conf"; return; }
  "${SUDO[@]}" install -d -o root -g root -m 0755 "$SERVICE_DROPIN_DIR"
  ensure_service_dropin_dir_secure
  mkdir -p "$tmp/dropin"
  printf '[Service]\nEnvironmentFile=%s\n' "$SOUNDTECH_ENV" > "$tmp/dropin/override.conf"
  local target="$SERVICE_DROPIN_DIR/override.conf"
  local staged="$SERVICE_DROPIN_DIR/.override.conf.tmp.$$"
  [[ ! -L "$target" ]] || fatal "systemd override cannot be a symlink: $target"
  run "${SUDO[@]}" install -o root -g root -m 0644 "$tmp/dropin/override.conf" "$staged"
  run "${SUDO[@]}" mv -fT -- "$staged" "$target"
  [[ ! -L "$target" ]] || fatal "systemd override became a symlink: $target"
  ensure_service_dropin_dir_secure
}

validate_host() {
  local report="${VALIDATION_REPORT:-}"
  local status=PASS
  if [[ -z "$report" ]]; then
    report="$(mktemp -t openiem-validation.XXXXXX)"
  else
    create_report_file "$report" || fatal "cannot create validation report: $report"
  fi
  [[ "$report" = /* ]] || fatal "validation report path must be absolute: $report"
  local rust="missing" node_version="missing" release="missing" service="unknown" soundtech_password="missing" service_env="missing" dropin_stat dropin_content dropin_bytes
  command -v rustc >/dev/null 2>&1 && rust="$(rustc --version)"
  command -v node >/dev/null 2>&1 && node_version="$(node --version)"
  runtime_config_is_secure && soundtech_password="configured"
  if [[ -d "$SERVICE_DROPIN_DIR" && ! -L "$SERVICE_DROPIN_DIR" && -f "$SERVICE_DROPIN_DIR/override.conf" && ! -L "$SERVICE_DROPIN_DIR/override.conf" ]]; then
    dropin_stat="$("${SUDO[@]}" stat -c '%u %a %F' -- "$SERVICE_DROPIN_DIR/override.conf" 2>/dev/null || true)"
    if [[ "$dropin_stat" == "0 644 regular file" ]]; then
      dropin_content="$(cat -- "$SERVICE_DROPIN_DIR/override.conf")"
      dropin_bytes="$(stat -c '%s' -- "$SERVICE_DROPIN_DIR/override.conf")"
      [[ "$dropin_content" == $'[Service]\nEnvironmentFile='"$SOUNDTECH_ENV" &&
        "$dropin_bytes" -eq $((${#dropin_content} + 1)) ]] && service_env="configured"
    fi
  fi
  RELEASE_DIR="$PREFIX/releases/$REF"
  if ! rustc_is_compatible; then status=FAIL; fi
  if ! command -v node >/dev/null 2>&1; then
    status=FAIL
  else
    local node_major
    node_major="$(node -p 'process.versions.node.split(".")[0]' 2>/dev/null || printf 0)"
    (( node_major >= MIN_NODE_MAJOR )) || status=FAIL
  fi
  if release_is_valid && [[ "$(readlink "$PREFIX/current" 2>/dev/null || true)" == "$RELEASE_DIR" ]]; then
    release="valid"
  else
    release="missing_or_inactive"; status=FAIL
  fi
  [[ "$soundtech_password" == "configured" ]] || status=FAIL
  if command -v systemctl >/dev/null 2>&1; then
    [[ "$service_env" == "configured" ]] || status=FAIL
    systemctl is-active --quiet "$SERVICE_NAME" && service="active" || service="inactive"
  fi
  [[ ! -e "$report" || ! -L "$report" ]] || fatal "validation report path cannot be a symlink: $report"
  {
    printf 'open-iem validation\n'
    printf 'ref=%s\n' "$REF"
    printf 'host=%s\n' "$(hostname 2>/dev/null || printf unknown)"
    printf 'kernel=%s\n' "$(uname -srmo)"
    printf 'architecture=%s\n' "$(uname -m)"
    printf 'rustc=%s\n' "$rust"
    printf 'node=%s\n' "$node_version"
    printf 'release=%s\n' "$release"
    printf 'service=%s\n' "$service"
    printf 'soundtech_password=%s\n' "$soundtech_password"
    printf 'env_file=%s\n' "$SOUNDTECH_ENV"
    printf 'status=%s\n' "$status"
  } | tee "$report" || fatal "cannot write validation report: $report"
  log "validation report: $report"
  [[ "$status" == PASS ]]
}

release_is_valid() {
  [[ -x "$PREFIX/releases/$REF/server/api-server" &&
     -x "$PREFIX/releases/$REF/server/open-iem-admin" &&
     -f "$PREFIX/releases/$REF/web/musician/index.html" &&
     -f "$PREFIX/releases/$REF/web/engineer/index.html" ]]
}

alsa_utils_version() {
  if command -v aplay >/dev/null 2>&1; then
    aplay --version 2>/dev/null | awk 'NR==1 {print $NF; exit}'
  elif command -v dpkg-query >/dev/null 2>&1; then
    dpkg-query -W -f='${Version}' alsa-utils 2>/dev/null || printf unavailable
  else
    printf unavailable
  fi
}

os_release_value() {
  local key="$1"
  [[ -r /etc/os-release ]] || { printf unknown; return; }
  awk -F= -v wanted="$key" '$1 == wanted { value=substr($0, index($0, "=") + 1); gsub(/^"|"$/, "", value); print value; exit }' /etc/os-release
}

create_report_file() {
  local path="$1" parent
  [[ "$path" = /* ]] || fatal "report path must be absolute: $path"
  parent="$(dirname -- "$path")"
  [[ -d "$parent" && ! -L "$parent" ]] || fatal "report directory must be a real directory: $parent"
  python3 - "$path" <<'PY'
import os
import stat
import sys
path = sys.argv[1]
flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
if hasattr(os, "O_NOFOLLOW"):
    flags |= os.O_NOFOLLOW
try:
    fd = os.open(path, flags, 0o600)
except FileExistsError:
    raise SystemExit("report path already exists")
except OSError as exc:
    raise SystemExit(f"cannot create report securely: {exc}")
os.close(fd)
PY
}

run_test_suite() {
  local report="${TEST_REPORT:-}"
  local suite_status=0 test_count=0 pass_count=0 fail_count=0
  if [[ -z "$report" ]]; then
    report="$(mktemp -t openiem-tests.XXXXXX)"
  else
    create_report_file "$report" || fatal "cannot create test report: $report"
  fi
  local source_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
  cd -- "$source_root" || fatal "cannot enter trusted source root"
  [[ ! -e "$report" || ! -L "$report" ]] || fatal "test report path cannot be a symlink: $report"
  [[ "$report" = /* ]] || fatal "test report path must be absolute: $report"
  local report_dir="$(dirname -- "$report")"
  [[ -d "$report_dir" && ! -L "$report_dir" ]] || fatal "test report directory must be a real directory: $report_dir"
  : > "$report" || fatal "cannot write test report: $report"
  {
    printf 'open-iem headless test report\n'
    printf 'ref=%s\n' "$REF"
    printf 'host=%s\n' "$(hostname 2>/dev/null || printf unknown)"
    printf 'kernel=%s\n' "$(uname -srmo)"
    printf 'architecture=%s\n' "$(uname -m)"
    printf 'os=%s\n' "$(os_release_value PRETTY_NAME)"
    printf 'os_id=%s\n' "$(os_release_value ID)"
    printf 'os_version=%s\n' "$(os_release_value VERSION_ID)"
    printf 'rustc=%s\n' "$(rustc --version 2>/dev/null || printf unavailable)"
    printf 'cargo=%s\n' "$(cargo --version 2>/dev/null || printf unavailable)"
    printf 'node=%s\n' "$(node --version 2>/dev/null || printf unavailable)"
    printf 'npm=%s\n' "$(npm --version 2>/dev/null || printf unavailable)"
    printf 'python=%s\n' "$(python3 --version 2>&1 || printf unavailable)"
    printf 'alsa_utils=%s\n' "$(alsa_utils_version)"
    printf 'started=%s\n\n' "$(date -Is)"
  } | tee "$report"

  run_test() {
    local name="$1"; shift
    test_count=$((test_count + 1))
    printf '\n== %s ==\n' "$name" | tee -a "$report"
    if timeout --signal=TERM 600s "$@" 2>&1 | tee -a "$report"; then
      pass_count=$((pass_count + 1))
      printf 'result=PASS\n' | tee -a "$report"
    else
      test_status=${PIPESTATUS[0]}
      fail_count=$((fail_count + 1))
      printf 'result=FAIL exit=%s\n' "$test_status" | tee -a "$report"
      suite_status=1
    fi
  }

  run_test 'shell syntax' bash -n scripts/install.sh scripts/ci/run-headless-audio.sh
  run_test 'git diff check' git diff --check
  run_test 'Rust formatter' cargo fmt --manifest-path server/Cargo.toml --all -- --check
  run_test 'Rust clippy' cargo clippy --manifest-path server/Cargo.toml --workspace --all-targets -- -D warnings
  run_test 'Rust unit and integration tests' cargo test --manifest-path server/Cargo.toml --workspace
  run_test 'Rust release build' cargo build --manifest-path server/Cargo.toml --workspace --release
  run_test 'API integration tests' cargo test --manifest-path server/Cargo.toml --package api-server --test integration
  run_test 'headless audio and deterministic DSP' scripts/ci/run-headless-audio.sh
  run_test 'Musician typecheck' npm run typecheck --prefix web/musician
  run_test 'Musician tests' npm test --prefix web/musician -- --run
  run_test 'Musician release build' npm run build --prefix web/musician
  run_test 'Engineer typecheck' npm run typecheck --prefix web/engineer
  run_test 'Engineer tests' npm test --prefix web/engineer -- --run
  run_test 'Engineer release build' npm run build --prefix web/engineer
  run_test 'documentation validation' bash scripts/validate-docs.sh
  run_test 'working tree diff check' git diff --check

  # Report optional physical ALSA status without weakening deterministic gates.
  if command -v aplay >/dev/null 2>&1 && command -v arecord >/dev/null 2>&1; then
    printf '\n== ALSA capability ==\n' | tee -a "$report"
    aplay -l 2>&1 | tee -a "$report" || true
    arecord -l 2>&1 | tee -a "$report" || true
  else
    printf '\nalsa=unavailable (alsa-utils not installed)\n' | tee -a "$report"
  fi
  run_test 'no audio process leaks' bash -c '! pgrep -f "(^|/)(arecord|aplay|pw-record|pw-play)( |$)" >/dev/null'

  local final_status=FAIL
  [[ $suite_status -eq 0 ]] && final_status=PASS
  printf '\nsummary=%s\n tests_total=%s\n tests_passed=%s\n tests_failed=%s\nstatus=%s\nfinished=%s\n' \
    "$([[ $suite_status -eq 0 ]] && printf 'ALL_TESTS_PASSED' || printf 'TESTS_FAILED')" \
    "$test_count" "$pass_count" "$fail_count" "$final_status" "$(date -Is)" | tee -a "$report"
  if [[ "$final_status" == PASS ]]; then
    log "ALL TESTS PASSED ($pass_count/$test_count)"
  else
    warn "TESTS FAILED ($fail_count failed, $pass_count passed, $test_count total)"
  fi
  log "test report: $report"
  return "$suite_status"
}

if (( VALIDATE_ONLY )); then
  validate_host
  exit $?
fi

# Fast path: valid immutable release needs no network, dependencies, or rebuild,
# but still repairs required runtime configuration and systemd integration.
if (( DRY_RUN == 0 )) && release_is_valid; then
  if [[ "$(readlink "$PREFIX/current" 2>/dev/null || true)" == "$PREFIX/releases/$REF" ]]; then
    log "release $REF already installed and active"
  else
    atomic_symlink "$PREFIX/releases/$REF" "$PREFIX/current"
    log "release $REF already installed; activated it"
  fi
  "${SUDO[@]}" install -d -m 0755 "$CONFIG_DIR" "$STATE_DIR"
  ensure_soundtech_env
  if (( NO_SERVICE == 0 )) && command -v systemctl >/dev/null 2>&1; then
    DROPIN_TMP="$(mktemp -d -t openiem-dropin.XXXXXX)"
    install_service_env_dropin "$DROPIN_TMP"
    rm -rf -- "$DROPIN_TMP"
    "${SUDO[@]}" systemctl daemon-reload
    "${SUDO[@]}" systemctl enable --now "$SERVICE_NAME"
  fi
  log 'runtime configuration repaired'
  printf '%s\n' "API binary: $BIN_DIR/api-server" "Admin CLI: $BIN_DIR/open-iem-admin" "State: $STATE_DIR" "Config: $CONFIG_DIR" "SoundTech password: generated or preserved (not printed)"
  if (( NO_SERVICE )); then
    printf '%s\n' "Service: $SERVICE_NAME (not managed)"
  elif command -v systemctl >/dev/null 2>&1; then
    if systemctl is-active --quiet "$SERVICE_NAME"; then
      printf '%s\n' "Service: $SERVICE_NAME active"
    else
      printf '%s\n' "Service: $SERVICE_NAME (managed/requested)"
    fi
  else
    printf '%s\n' "Service: $SERVICE_NAME (systemd unavailable)"
  fi
  if (( RUN_TESTS )); then
    run_test_suite || exit $?
  fi
  exit 0
fi

preflight_node_check
install_deps
if (( DRY_RUN )); then
  log "would verify tools: git curl openssl cargo rustc node npm (Rust >= ${MIN_RUSTC_MAJOR}.${MIN_RUSTC_MINOR}, Node.js >= ${MIN_NODE_MAJOR})"
  log "would install/upgrade Rust with rustup only when existing rustc is too old"
  log "would clone $REPO_URL at immutable commit $REF"
  log 'would verify checkout resolves exactly to requested commit'
  log 'would build Rust workspace in release mode'
  log 'would install dependencies and build web/musician and web/engineer'
  log "would install binaries and web assets below $PREFIX"
  (( NO_SERVICE )) || log "would install and enable $SERVICE_NAME"
  (( ROTATE_KEYS )) && log 'would rotate JWT keys only after confirmation'
  log 'dry-run complete; host unchanged'
  exit 0
fi
ensure_rust_toolchain

TMP_DIR=""
cleanup() {
  if [[ -n "$TMP_DIR" && -d "$TMP_DIR" && $KEEP_SOURCE -eq 0 ]]; then rm -rf -- "$TMP_DIR"; fi
}
trap cleanup EXIT

check_tools

TMP_DIR="$(mktemp -d -t openiem-install.XXXXXX)"
trap cleanup EXIT
log "cloning $REPO_URL at immutable commit $REF"
# Fetch only requested commit, then verify exact object identity before build.
git clone --filter=blob:none --no-checkout "$REPO_URL" "$TMP_DIR/src"
git -C "$TMP_DIR/src" fetch --depth 1 origin "$REF"
git -C "$TMP_DIR/src" checkout --detach "$REF"
resolved_ref="$(git -C "$TMP_DIR/src" rev-parse HEAD)"
[[ "$resolved_ref" == "$REF" ]] || fatal "checkout did not resolve requested commit SHA"
cd "$TMP_DIR/src"

RELEASE_DIR="$PREFIX/releases/$REF"
if release_is_valid; then
  log "release $REF already installed and valid; nothing to do"
  exit 0
fi

log 'building Rust workspace'
cargo build --manifest-path server/Cargo.toml --workspace --release
log 'building musician UI'
npm ci --prefix web/musician --ignore-scripts
npm run build --prefix web/musician
log 'building engineer UI'
npm ci --prefix web/engineer --ignore-scripts
npm run build --prefix web/engineer

# Build complete release in isolated staging. Existing current release stays untouched on failure.
STAGE="$(mktemp -d -t openiem-stage.XXXXXX)"
RELEASE_DIR="$PREFIX/releases/$REF"
PREVIOUS_RELEASE=""
CURRENT_LINK="$PREFIX/current"
INSTALL_COMMITTED=0
KEY_TMP=""
if [[ -L "$CURRENT_LINK" ]]; then PREVIOUS_RELEASE="$(readlink "$CURRENT_LINK")"; fi
cleanup_release() {
  if (( INSTALL_COMMITTED == 0 )); then
    if [[ -n "$PREVIOUS_RELEASE" ]]; then
      atomic_symlink "$PREVIOUS_RELEASE" "$CURRENT_LINK"
    else
      run "${SUDO[@]}" rm -f -- "$CURRENT_LINK"
    fi
    if [[ -n "$RELEASE_DIR" && -d "$RELEASE_DIR" && "$RELEASE_DIR" != "$PREVIOUS_RELEASE" ]]; then
      run "${SUDO[@]}" rm -rf -- "$RELEASE_DIR"
    fi
  fi
  [[ -z "$KEY_TMP" ]] || rm -rf -- "$KEY_TMP"
  rm -rf -- "$STAGE"
  cleanup
}
trap cleanup_release EXIT
mkdir -p "$STAGE/server" "$STAGE/web/musician" "$STAGE/web/engineer"
install -m 0755 server/target/release/api-server "$STAGE/server/api-server"
install -m 0755 server/target/release/open-iem-admin "$STAGE/server/open-iem-admin"
cp -a web/musician/dist/. "$STAGE/web/musician/"
cp -a web/engineer/dist/. "$STAGE/web/engineer/"
[[ ! -e "$RELEASE_DIR" ]] || fatal "release already installed at $RELEASE_DIR; choose a different immutable commit"
run "${SUDO[@]}" install -d -m 0755 "$PREFIX/releases" "$RELEASE_DIR" "$BIN_DIR" "$CONFIG_DIR" "$STATE_DIR"
ensure_config_dir_secure
run "${SUDO[@]}" cp -a "$STAGE/." "$RELEASE_DIR/"
atomic_symlink "$RELEASE_DIR" "$PREFIX/current"
atomic_symlink "$PREFIX/current/server/api-server" "$BIN_DIR/api-server"
atomic_symlink "$PREFIX/current/server/open-iem-admin" "$BIN_DIR/open-iem-admin"

if ! id openiem >/dev/null 2>&1; then run "${SUDO[@]}" useradd --system --home-dir /nonexistent --no-create-home --shell /usr/sbin/nologin openiem; fi
run "${SUDO[@]}" chown -R openiem:openiem "$STATE_DIR"
run "${SUDO[@]}" install -d -o root -g openiem -m 0750 "$CONFIG_DIR/keys"
PRIVATE_KEY="$CONFIG_DIR/keys/ed25519_private.pem"
PUBLIC_KEY="$CONFIG_DIR/keys/ed25519_public.pem"
if [[ -e "$PRIVATE_KEY" || -e "$PUBLIC_KEY" ]]; then
  if [[ ! -e "$PRIVATE_KEY" || ! -e "$PUBLIC_KEY" ]] && (( ROTATE_KEYS == 0 )); then
    fatal 'incomplete JWT key pair; use --rotate-keys after reviewing host state'
  fi
  (( ROTATE_KEYS )) || { log 'existing JWT keys preserved'; }
  if (( ROTATE_KEYS )); then
    if (( ASSUME_YES == 0 )); then
      [[ -t 0 ]] || fatal 'key rotation requires interactive confirmation or --yes'
      printf '[open-iem] WARNING: this invalidates all existing JWT sessions. Replace keys? Type ROTATE: '
      read -r confirmation
      [[ "$confirmation" == ROTATE ]] || fatal 'key rotation cancelled'
    fi
    log 'rotating JWT keys after confirmation'
  fi
fi
if [[ ! -e "$PRIVATE_KEY" && ! -e "$PUBLIC_KEY" ]] || (( ROTATE_KEYS )); then
  KEY_TMP="$(mktemp -d -t openiem-keys.XXXXXX)"
  umask 077
  openssl genpkey -algorithm ed25519 -out "$KEY_TMP/private.pem"
  openssl pkey -in "$KEY_TMP/private.pem" -pubout -out "$KEY_TMP/public.pem"
  run "${SUDO[@]}" install -o openiem -g openiem -m 0600 "$KEY_TMP/private.pem" "$PRIVATE_KEY.new"
  run "${SUDO[@]}" install -o root -g openiem -m 0640 "$KEY_TMP/public.pem" "$PUBLIC_KEY.new"
  run "${SUDO[@]}" mv -f "$PRIVATE_KEY.new" "$PRIVATE_KEY"
  run "${SUDO[@]}" mv -f "$PUBLIC_KEY.new" "$PUBLIC_KEY"
  rm -rf -- "$KEY_TMP"
  log 'JWT keys generated'
fi

ensure_soundtech_env

if (( NO_SERVICE == 0 )) && command -v systemctl >/dev/null 2>&1; then
  sed -e "s#^WorkingDirectory=.*#WorkingDirectory=$PREFIX#" \
      -e "s#^ExecStart=.*#ExecStart=$PREFIX/current/server/api-server#" \
      -e "s#^Environment=OPENIEM_DB_PATH=.*#Environment=OPENIEM_DB_PATH=$STATE_DIR/openiem.db#" \
      -e "s#^Environment=OPENIEM_JWT_PRIVATE_PEM=.*#Environment=OPENIEM_JWT_PRIVATE_PEM=$CONFIG_DIR/keys/ed25519_private.pem#" \
      -e "s#^Environment=OPENIEM_JWT_PUBLIC_PEM=.*#Environment=OPENIEM_JWT_PUBLIC_PEM=$CONFIG_DIR/keys/ed25519_public.pem#" \
      "$TMP_DIR/src/deployment/systemd/openiem-server.service" > "$STAGE/$SERVICE_NAME"
  run "${SUDO[@]}" install -o root -g root -m 0644 "$STAGE/$SERVICE_NAME" "/etc/systemd/system/$SERVICE_NAME"
  install_service_env_dropin "$STAGE"
  run "${SUDO[@]}" systemctl daemon-reload
  run "${SUDO[@]}" systemctl enable --now "$SERVICE_NAME"
else
  warn 'systemd unavailable or disabled; start api-server manually'
fi

INSTALL_COMMITTED=1
log 'installation complete'
service_status="disabled_or_unavailable"
if (( NO_SERVICE == 0 )) && command -v systemctl >/dev/null 2>&1; then
  service_status="$(systemctl is-active "$SERVICE_NAME" 2>/dev/null || true)"
  [[ -n "$service_status" ]] || service_status="inactive"
fi
printf '%s\n' \
  "API binary: $BIN_DIR/api-server" \
  "Admin CLI: $BIN_DIR/open-iem-admin" \
  "State: $STATE_DIR" \
  "Config: $CONFIG_DIR" \
  "SoundTech password: generated or preserved (not printed)" \
  "SoundTech env file: $SOUNDTECH_ENV" \
  "Service: $SERVICE_NAME ($service_status)"
if (( RUN_TESTS )); then
  run_test_suite
fi
