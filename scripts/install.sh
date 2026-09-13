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
DRY_RUN=0
SKIP_DEPS=0
NO_SERVICE=0
KEEP_SOURCE=0
ROTATE_KEYS=0
ASSUME_YES=0

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
  --skip-deps      Do not install/check OS packages
  --no-service     Do not install systemd unit
  --keep-source    Keep temporary checkout after installation
  --rotate-keys    Replace existing JWT keys after explicit confirmation
  --yes            Confirm destructive actions (use with --rotate-keys)
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
    --skip-deps) SKIP_DEPS=1; shift ;;
    --no-service) NO_SERVICE=1; shift ;;
    --keep-source) KEEP_SOURCE=1; shift ;;
    --rotate-keys) ROTATE_KEYS=1; shift ;;
    --yes) ASSUME_YES=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) fatal "unknown option: $1 (use --help)" ;;
  esac
done

[[ "$(uname -s)" == Linux ]] || fatal "Linux required"
[[ "$PREFIX" = /* && "$BIN_DIR" = /* && "$STATE_DIR" = /* && "$CONFIG_DIR" = /* ]] || fatal "paths must be absolute"
[[ "$PREFIX$BIN_DIR$STATE_DIR$CONFIG_DIR" != *$'\n'* && "$PREFIX$BIN_DIR$STATE_DIR$CONFIG_DIR" != *'&'* && "$PREFIX$BIN_DIR$STATE_DIR$CONFIG_DIR" != *'\\'* && "$PREFIX$BIN_DIR$STATE_DIR$CONFIG_DIR" != *'#'* ]] || fatal "paths cannot contain newline, &, \\, or #"
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
      run "${SUDO[@]}" apt-get install -y --no-install-recommends git ca-certificates curl build-essential pkg-config openssl cargo rustc nodejs npm
      ;;
    dnf|yum)
      run "${SUDO[@]}" "$pm" install -y git ca-certificates curl gcc gcc-c++ make pkgconf-pkg-config openssl openssl-devel cargo rust nodejs npm
      ;;
    pacman)
      run "${SUDO[@]}" pacman -Sy --needed --noconfirm git ca-certificates curl base-devel openssl rust nodejs npm
      ;;
    zypper)
      run "${SUDO[@]}" zypper --non-interactive install git ca-certificates curl gcc gcc-c++ make pkg-config libopenssl-devel rust nodejs npm
      ;;
    apk)
      run "${SUDO[@]}" apk add git ca-certificates curl build-base pkgconf openssl-dev rust cargo nodejs npm
      ;;
  esac
}

check_tools() {
  local c
  for c in git curl openssl cargo rustc node npm; do need_cmd "$c"; done
  local node_major
  node_major="$(node -p 'process.versions.node.split(".")[0]')"
  (( node_major >= 20 )) || fatal "Node.js >= 20 required; found $(node --version)"
}

TMP_DIR=""
cleanup() {
  if [[ -n "$TMP_DIR" && -d "$TMP_DIR" && $KEEP_SOURCE -eq 0 ]]; then rm -rf -- "$TMP_DIR"; fi
}
trap cleanup EXIT

install_deps
if (( DRY_RUN )); then
  log "would verify tools: git curl openssl cargo rustc node npm (Node.js >= 20)"
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
      run "${SUDO[@]}" ln -sfn "$PREVIOUS_RELEASE" "$CURRENT_LINK"
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
run "${SUDO[@]}" cp -a "$STAGE/." "$RELEASE_DIR/"
run "${SUDO[@]}" ln -sfn "$RELEASE_DIR" "$PREFIX/current"
run "${SUDO[@]}" ln -sfn "$PREFIX/current/server/api-server" "$BIN_DIR/api-server"
run "${SUDO[@]}" ln -sfn "$PREFIX/current/server/open-iem-admin" "$BIN_DIR/open-iem-admin"

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

if (( NO_SERVICE == 0 )) && command -v systemctl >/dev/null 2>&1; then
  sed -e "s#^WorkingDirectory=.*#WorkingDirectory=$PREFIX#" \
      -e "s#^ExecStart=.*#ExecStart=$PREFIX/current/server/api-server#" \
      -e "s#^Environment=OPENIEM_DB_PATH=.*#Environment=OPENIEM_DB_PATH=$STATE_DIR/openiem.db#" \
      -e "s#^Environment=OPENIEM_JWT_PRIVATE_PEM=.*#Environment=OPENIEM_JWT_PRIVATE_PEM=$CONFIG_DIR/keys/ed25519_private.pem#" \
      -e "s#^Environment=OPENIEM_JWT_PUBLIC_PEM=.*#Environment=OPENIEM_JWT_PUBLIC_PEM=$CONFIG_DIR/keys/ed25519_public.pem#" \
      "$TMP_DIR/src/deployment/systemd/openiem-server.service" > "$STAGE/$SERVICE_NAME"
  run "${SUDO[@]}" install -o root -g root -m 0644 "$STAGE/$SERVICE_NAME" "/etc/systemd/system/$SERVICE_NAME"
  run "${SUDO[@]}" systemctl daemon-reload
  run "${SUDO[@]}" systemctl enable --now "$SERVICE_NAME"
else
  warn 'systemd unavailable or disabled; start api-server manually'
fi

INSTALL_COMMITTED=1
log 'installation complete'
printf '%s\n' "API binary: $BIN_DIR/api-server" "Admin CLI: $BIN_DIR/open-iem-admin" "State: $STATE_DIR" "Config: $CONFIG_DIR"
