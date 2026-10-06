#!/usr/bin/env bash
# Start Open IEM public Web UI test stack with host supplied at runtime.
set -Eeuo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$REPO_ROOT/docker-compose.public.yml"
ENV_FILE="$REPO_ROOT/.env.local"
PUBLIC_HOST="${1:-${OPENIEM_PUBLIC_HOST:-}}"

if [[ -z "$PUBLIC_HOST" ]]; then
  printf 'usage: %s PUBLIC_HOST\n' "$0" >&2
  printf 'example: %s 203.0.113.10\n' "$0" >&2
  exit 2
fi

if [[ ! "$PUBLIC_HOST" =~ ^[A-Za-z0-9.-]+$ || "$PUBLIC_HOST" == .* || "$PUBLIC_HOST" == *. ]]; then
  printf 'invalid public host\n' >&2
  exit 2
fi
[[ "${#PUBLIC_HOST}" -le 253 ]] || { printf 'public host too long\n' >&2; exit 2; }
if [[ "$PUBLIC_HOST" =~ ^[0-9.]+$ ]]; then
  IFS=. read -r -a octets <<< "$PUBLIC_HOST"
  [[ "${#octets[@]}" == 4 ]] || { printf 'invalid IPv4 host\n' >&2; exit 2; }
  for octet in "${octets[@]}"; do
    [[ "$octet" =~ ^[0-9]{1,3}$ && "$octet" -le 255 ]] || { printf 'invalid IPv4 host\n' >&2; exit 2; }
  done
else
  IFS=. read -r -a labels <<< "$PUBLIC_HOST"
  for label in "${labels[@]}"; do
    [[ "${#label}" -le 63 ]] || { printf 'invalid DNS host\n' >&2; exit 2; }
    [[ "$label" =~ ^[A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?$ ]] || { printf 'invalid DNS host\n' >&2; exit 2; }
  done
fi

[[ -f "$COMPOSE_FILE" && ! -L "$COMPOSE_FILE" ]] || { printf 'missing compose file: %s\n' "$COMPOSE_FILE" >&2; exit 1; }
[[ -f "$ENV_FILE" && ! -L "$ENV_FILE" ]] || { printf 'missing env file: %s\n' "$ENV_FILE" >&2; exit 1; }
[[ -d "$REPO_ROOT/keys" && ! -L "$REPO_ROOT/keys" ]] || { printf 'invalid keys directory\n' >&2; exit 1; }
[[ -f "$REPO_ROOT/keys/ed25519_private.pem" && ! -L "$REPO_ROOT/keys/ed25519_private.pem" ]] || { printf 'missing JWT private key\n' >&2; exit 1; }
[[ -f "$REPO_ROOT/keys/ed25519_public.pem" && ! -L "$REPO_ROOT/keys/ed25519_public.pem" ]] || { printf 'missing JWT public key\n' >&2; exit 1; }
[[ "$(stat -c '%a' "$REPO_ROOT/keys/ed25519_private.pem")" == "600" ]] || { printf 'JWT private key must mode 600\n' >&2; exit 1; }
openssl pkey -in "$REPO_ROOT/keys/ed25519_private.pem" -pubout 2>/dev/null | cmp -s - "$REPO_ROOT/keys/ed25519_public.pem" || {
  printf 'JWT key pair is invalid or mismatched\n' >&2
  exit 1
}
[[ -s "$ENV_FILE" ]] || { printf 'empty env file: %s\n' "$ENV_FILE" >&2; exit 1; }
chmod 600 "$ENV_FILE"
password_count="$(grep -c '^OPENIEM_SOUNDTECH_PASSWORD=' "$ENV_FILE" || true)"
password_line="$(grep '^OPENIEM_SOUNDTECH_PASSWORD=' "$ENV_FILE" || true)"
password_value="${password_line#OPENIEM_SOUNDTECH_PASSWORD=}"
[[ "$password_count" == 1 && -n "$password_line" && -n "$password_value" ]] || { printf 'OPENIEM_SOUNDTECH_PASSWORD is empty\n' >&2; exit 1; }

export OPENIEM_PUBLIC_HOST="$PUBLIC_HOST"
cd "$REPO_ROOT"

docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" config --quiet
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d --build --force-recreate

docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" ps
printf 'Engineer: http://%s:5174/\n' "$PUBLIC_HOST"
printf 'Musician: http://%s:5173/\n' "$PUBLIC_HOST"
printf 'API: http://%s:3000/\n' "$PUBLIC_HOST"
