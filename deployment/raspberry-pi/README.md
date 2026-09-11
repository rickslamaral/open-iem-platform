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
# Replace VERSION with a published release tag.
VERSION=v0.3.1
ARCHIVE="open-iem-server-${VERSION#v}-aarch64-linux.tar.gz"
RELEASE_URL="https://github.com/rickslamaral/open-iem-platform/releases/download/${VERSION}"
curl --fail --show-error --location --remote-name "${RELEASE_URL}/${ARCHIVE}"
curl --fail --show-error --location --remote-name "${RELEASE_URL}/${ARCHIVE}.sha256"
# Abort unless downloaded archive matches published SHA-256 checksum.
sha256sum --check "${ARCHIVE}.sha256"
tar -xzf "${ARCHIVE}"
sudo install -o root -g root -m 755 "open-iem-server-${VERSION#v}-aarch64-linux/api-server" /usr/local/bin/api-server
```

## 2. Create service user and directories

```bash
sudo useradd --system --no-create-home --shell /usr/sbin/nologin openiem
sudo mkdir -p /etc/openiem/keys /var/lib/openiem /opt/openiem
sudo chown openiem:openiem /var/lib/openiem /opt/openiem
```

## 3. Generate JWT keys

```bash
sudo openssl genpkey -algorithm ed25519 -out /etc/openiem/keys/ed25519_private.pem
sudo openssl pkey -in /etc/openiem/keys/ed25519_private.pem -pubout -out /etc/openiem/keys/ed25519_public.pem
sudo chmod 600 /etc/openiem/keys/ed25519_private.pem
sudo chmod 644 /etc/openiem/keys/ed25519_public.pem
sudo chown openiem:openiem /etc/openiem/keys/ed25519_private.pem /etc/openiem/keys/ed25519_public.pem
```

## 4. Install systemd service

```bash
sudo cp deployment/systemd/openiem-server.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable openiem-server
sudo systemctl start openiem-server
sudo systemctl status openiem-server
```

## 5. Install Caddy and configure TLS

```bash
# Install Caddy (Debian/Ubuntu/Raspbian)
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
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
sudo mkdir -p /etc/caddy/certs
mkcert -cert-file /etc/caddy/certs/iem.local.pem \
       -key-file /etc/caddy/certs/iem.local-key.pem \
       iem.local $LAN_IP
sudo chmod 644 /etc/caddy/certs/iem.local.pem
sudo chmod 600 /etc/caddy/certs/iem.local-key.pem

# Install Caddyfile
sudo cp deployment/caddy/Caddyfile /etc/caddy/Caddyfile
# Edit /etc/caddy/Caddyfile: replace 'iem.local' with your Pi's LAN IP if mDNS is unavailable.
sudo systemctl restart caddy
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
