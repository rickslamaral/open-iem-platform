#!/usr/bin/env bash
# Static contract check for kiosk binding, URL, origin, and safety defaults.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
server="$repo_root/deployment/systemd/openiem-server.service"
installer="$repo_root/scripts/install.sh"
launcher="$repo_root/deployment/systemd/openiem-kiosk.service"
main="$repo_root/server/api-server/src/main.rs"
security="$repo_root/server/api-server/src/security.rs"

for file in "$server" "$installer" "$launcher" "$main" "$security"; do [[ -f "$file" ]] || { echo "missing: $file" >&2; exit 1; }; done
bash -n "$installer" "$launcher"
grep -q "kiosk_bind='0.0.0.0:8080'" "$installer"
grep -q "kiosk_bind='127.0.0.1:8080'" "$installer"
grep -q "kiosk_env='true'" "$installer"
grep -q 'OPENIEM_KIOSK_MODE=false' "$server"
grep -q 'OPENIEM_BIND_ADDR=127.0.0.1:8080' "$server"
grep -q 'secure local QR broker contract is not available' "$installer"
grep -q 'ConditionPathExists=/run/openiem-kiosk-secure-qr-broker' "$launcher"
grep -q 'OPENIEM_KIOSK_MODE.*requires non-loopback' "$main"
grep -q 'same_host_origin' "$security"
grep -q 'CACHE_CONTROL' "$main"
grep -q 'CONTENT_SECURITY_POLICY' "$main"
grep -q 'ServeFile' "$main"
grep -q 'OPENIEM_QR_SESSION_TTL_SECONDS' "$repo_root/server/api-server/src/routes/qr.rs"
if grep -qE 'ufw|firewall-cmd|iptables|nft ' "$installer"; then
  echo 'installer must not silently open firewall' >&2
  exit 1
fi
printf 'kiosk contract check passed\n'
