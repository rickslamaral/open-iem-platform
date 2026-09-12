# Raspberry Pi 5 — Open IEM Platform Deployment Guide

> **Status:** SIMULATED on VPS. Requires real Raspberry Pi 5 hardware for full validation. Audio (PipeWire/ALSA) remains unverified until hardware is available.

## Prerequisites

| Requirement | Notes |
|---|---|
| Raspberry Pi 5 (4 GB RAM minimum) | 8 GB recommended |
| Raspberry Pi OS (64-bit, Bookworm) | ARM64; 32-bit not supported |
| Network: static LAN IP or mDNS (`iem.local`) | |
| Caddy 2.x | TLS reverse proxy |
| mkcert | Local CA for LAN TLS |
| `openiem` system user | Dedicated service account |

## 1. Install binary

Download the ARM64 release artifact from GitHub Releases:

```bash
set -euo pipefail

# Replace VERSION with a published release tag.
VERSION=v0.3.1
[[ "${VERSION}" =~ ^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] \
  || { printf 'Invalid release tag: %s\n' "${VERSION}" >&2; exit 1; }
ARCHIVE="open-iem-server-${VERSION#v}-aarch64-linux.tar.gz"
RELEASE_URL="https://github.com/rickslamaral/open-iem-platform/releases/download/${VERSION}"
WORK_DIR=$(mktemp -d)
trap 'rm -rf -- "${WORK_DIR}"' EXIT
REPO_ROOT=$(git rev-parse --show-toplevel)
ARCHIVE_PATH="${WORK_DIR}/${ARCHIVE}"
CHECKSUM_PATH="${ARCHIVE_PATH}.sha256"
SIGNATURE_PATH="${ARCHIVE_PATH}.sig"
PUBLIC_KEY="/etc/openiem/release-signing-key.pem"

# Public key must be installed through an independently authenticated channel.
[[ -f "${PUBLIC_KEY}" && ! -L "${PUBLIC_KEY}" ]] \
  || { printf 'Missing trusted release signing public key: %s\n' "${PUBLIC_KEY}" >&2; exit 1; }

curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
  --output "${ARCHIVE_PATH}" "${RELEASE_URL}/${ARCHIVE}"
curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
  --output "${SIGNATURE_PATH}" "${RELEASE_URL}/${ARCHIVE}.sig"
python3 "${REPO_ROOT}/scripts/verify-release-signature.py" \
  "${ARCHIVE_PATH}" "${SIGNATURE_PATH}" "${PUBLIC_KEY}"
curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
  --output "${CHECKSUM_PATH}" "${RELEASE_URL}/${ARCHIVE}.sha256"
# Checksum file names archive by basename; verify exact downloaded file.
( cd "${WORK_DIR}" && sha256sum --check "${ARCHIVE}.sha256" )

# Validate every member with repository validator before extraction.
[[ -f "${REPO_ROOT}/scripts/validate-release-archive.py" && ! -L "${REPO_ROOT}/scripts/validate-release-archive.py" ]] \
  || { printf 'Missing trusted archive validator\n' >&2; exit 1; }
python3 "${REPO_ROOT}/scripts/validate-release-archive.py" \
  "${ARCHIVE_PATH}" api-server open-iem-admin

EXTRACT_DIR="${WORK_DIR}/open-iem-server-${VERSION#v}-aarch64-linux"
tar --extract --file "${ARCHIVE_PATH}" --directory "${WORK_DIR}" --no-same-owner --no-same-permissions
[[ -f "${EXTRACT_DIR}/api-server" && ! -L "${EXTRACT_DIR}/api-server" ]] \
  || { printf 'Archive missing regular api-server binary\n' >&2; exit 1; }
[[ -f "${EXTRACT_DIR}/open-iem-admin" && ! -L "${EXTRACT_DIR}/open-iem-admin" ]] \
  || { printf 'Archive missing regular open-iem-admin binary\n' >&2; exit 1; }
sudo install -o root -g root -m 755 "${EXTRACT_DIR}/api-server" /usr/local/bin/api-server
sudo install -o root -g root -m 755 "${EXTRACT_DIR}/open-iem-admin" /usr/local/bin/open-iem-admin
```

## 2. Create service user and directories

```bash
set -euo pipefail
sudo useradd --system --no-create-home --shell /usr/sbin/nologin openiem
sudo install -d -o root -g openiem -m 0750 /etc/openiem
sudo install -d -o root -g openiem -m 0750 /etc/openiem/keys
sudo install -d -o openiem -g openiem -m 0750 /var/lib/openiem /opt/openiem
```

## 3. Generate JWT keys

```bash
set -euo pipefail
sudo install -d -o root -g openiem -m 0750 /etc/openiem/keys
KEY_WORK_DIR=$(mktemp -d)
trap 'rm -rf -- "${KEY_WORK_DIR}"' EXIT
sudo openssl genpkey -algorithm ed25519 -out "${KEY_WORK_DIR}/ed25519_private.pem"
sudo openssl pkey -in "${KEY_WORK_DIR}/ed25519_private.pem" -pubout -out "${KEY_WORK_DIR}/ed25519_public.pem"
sudo install -o openiem -g openiem -m 0600 "${KEY_WORK_DIR}/ed25519_private.pem" /etc/openiem/keys/ed25519_private.pem
sudo install -o openiem -g openiem -m 0644 "${KEY_WORK_DIR}/ed25519_public.pem" /etc/openiem/keys/ed25519_public.pem
```

## 4. Install systemd service

Execute from a trusted clone of this repository. Resolve repository root before using deployment files:

```bash
set -euo pipefail
REPO_ROOT=$(git rev-parse --show-toplevel)
SERVICE_FILE="${REPO_ROOT}/deployment/systemd/openiem-server.service"
[[ -f "${SERVICE_FILE}" && ! -L "${SERVICE_FILE}" ]] \
  || { printf 'Missing regular systemd unit: %s\n' "${SERVICE_FILE}" >&2; exit 1; }
systemd-analyze verify "${SERVICE_FILE}"
sudo install -o root -g root -m 0644 "${SERVICE_FILE}" /etc/systemd/system/openiem-server.service
if ! [[ -f /etc/systemd/system/openiem-server.service && ! -L /etc/systemd/system/openiem-server.service ]]; then
  printf 'Installed systemd unit is not a regular file\n' >&2
  sudo rm -f /etc/systemd/system/openiem-server.service
  exit 1
fi
if ! systemd-analyze verify /etc/systemd/system/openiem-server.service; then
  sudo rm -f /etc/systemd/system/openiem-server.service
  exit 1
fi
sudo systemctl daemon-reload
sudo systemctl enable openiem-server
sudo systemctl start openiem-server
sudo systemctl status openiem-server
```

## 5. Install Caddy and configure TLS

```bash
set -euo pipefail
REPO_ROOT=$(git rev-parse --show-toplevel)
# Install Caddy (Debian/Ubuntu/Raspbian)
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
  'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
  'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list >/dev/null
sudo apt update && sudo apt install caddy

# Install mkcert
sudo apt install -y mkcert
mkcert -install  # installs local CA into system trust store

# Protect the mkcert CA private key — CRITICAL.
# If rootCA.key is compromised, an attacker can issue valid certs for all devices
# that trust your CA, enabling silent MITM for ALL HTTPS traffic on those devices,
# not just Open IEM. Restrict access immediately after CA creation.
CA_ROOT=$(mkcert -CAROOT)
sudo chmod 700 "$CA_ROOT"
sudo chmod 600 "$CA_ROOT/rootCA.key"
# Do NOT include rootCA.key in unencrypted backups.
# To revoke: regenerate CA with 'mkcert -uninstall && mkcert -install',
# re-issue all certificates, and remove the old rootCA.pem from every device.

# Certificates issued by mkcert have a default validity of ~2 years 3 months.
# Add a reminder or systemd timer to renew before expiry. Check with:
#   openssl x509 -enddate -noout -in /etc/caddy/certs/iem.local.pem

# Generate LAN certificate (adjust IPs to your Pi's LAN IP)
LAN_IP=$(hostname -I | awk '{print $1}')
CERT_WORK_DIR=$(mktemp -d)
trap 'rm -rf -- "${CERT_WORK_DIR}"' EXIT
mkcert -cert-file "${CERT_WORK_DIR}/iem.local.pem" \
       -key-file "${CERT_WORK_DIR}/iem.local-key.pem" \
       iem.local "$LAN_IP"
sudo install -d -o root -g caddy -m 0750 /etc/caddy/certs
sudo install -o root -g caddy -m 0644 "${CERT_WORK_DIR}/iem.local.pem" /etc/caddy/certs/iem.local.pem
sudo install -o root -g caddy -m 0640 "${CERT_WORK_DIR}/iem.local-key.pem" /etc/caddy/certs/iem.local-key.pem

# Copy Caddyfile through an unprivileged descriptor, then install fixed temp file
CADDYFILE="${REPO_ROOT}/deployment/caddy/Caddyfile"
CADDYFILE_WORK="${CERT_WORK_DIR}/Caddyfile"
python3 - "${CADDYFILE}" "${CADDYFILE_WORK}" <<'PY'
import os
import shutil
import stat
import sys

source_fd = os.open(sys.argv[1], os.O_RDONLY | os.O_NOFOLLOW)
if not stat.S_ISREG(os.fstat(source_fd).st_mode):
    os.close(source_fd)
    raise SystemExit(f"Caddyfile is not a regular file: {sys.argv[1]}")
with os.fdopen(source_fd, "rb") as source, open(sys.argv[2], "xb") as target:
    shutil.copyfileobj(source, target)
PY
sudo install -o root -g root -m 0644 "${CADDYFILE_WORK}" /etc/caddy/Caddyfile
rm -f -- "${CADDYFILE_WORK}"
# If mDNS is unavailable, replace 'iem.local' with the LAN IP in:
#   /etc/caddy/Caddyfile
#   /etc/systemd/system/openiem-server.service (OPENIEM_ALLOWED_ORIGINS=https://<LAN-IP>)
sudo systemctl daemon-reload
sudo systemctl restart caddy
sudo systemctl restart openiem-server
sudo systemctl status caddy
```

## 6. Trust local CA on musician/engineer devices

```bash
# On the Pi: find the CA root
mkcert -CAROOT
# → /root/.local/share/mkcert/rootCA.pem (or similar)

# Copy rootCA.pem to each device and install:
# Android: Settings > Security > Install certificate
# iOS: AirDrop the .pem, then Settings > General > VPN & Device Management > trust
# macOS: Keychain Access > System > import, set to Always Trust
# Windows: certmgr.msc > Trusted Root Certification Authorities > import
```

## 7. Verify deployment

```bash
# Health check (from another device on the LAN)
curl https://iem.local/api/v1/health

# Expected response: {"status":"ok"}

# WebSocket test (from the Pi itself)
curl -i -N -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Version: 13" \
     -H "Sec-WebSocket-Key: test" \
     https://iem.local/ws/v1
# Expects 400 (missing auth) — confirms WebSocket endpoint is reachable via TLS
```

## 8. PipeWire audio (SIMULATED until hardware validated)

PipeWire/ALSA integration is SIMULATED on VPS. On Raspberry Pi 5:

```bash
sudo apt install -y pipewire pipewire-audio-client-libraries wireplumber
systemctl --user enable --now pipewire wireplumber
```

Validate with `pw-top`. Audio processing details are in `docs/research/pipewire-integration.md`.

## Known limitations

| Limitation | Status |
|---|---|
| PipeWire/ALSA audio | SIMULATED — requires Pi hardware |
| ARM64 binary runtime | Not hardware-validated |
| mDNS `iem.local` resolution | Requires `avahi-daemon` on Pi; may need manual IP on some Android versions |
| PREEMPT_RT kernel | Research phase — see TODO |
| Windows/macOS server | UNSUPPORTED |
