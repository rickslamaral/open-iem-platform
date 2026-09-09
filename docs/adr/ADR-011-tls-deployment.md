# ADR-011: TLS Deployment Strategy

## Status
Accepted — Phase 22 (2026-09-09)

## Context
Open IEM Platform requires TLS before any deployment beyond localhost loopback. ADR-008 mandates HTTPS for login, authenticated traffic and WebSocket upgrade. The server binary enforces this gate: non-loopback plaintext HTTP requires `OPENIEM_ALLOW_INSECURE_HTTP=true`, which is an explicit dev-only override.

Deployment targets:
- **Raspberry Pi 5** — primary production target, LAN-only, no internet exposure.
- **VPS / developer workstation** — SIMULATED, localhost only, HTTP allowed.

## Decision
Terminate TLS at a reverse proxy (Caddy or Nginx) placed in front of `api-server`. The `api-server` binary always binds to `127.0.0.1:8080` in production; the proxy receives public HTTPS traffic and forwards to the loopback. The server binary never sees raw external connections.

Rationale:
- Caddy provides automatic certificate management (mkcert for LAN, ACME for internet-facing).
- Proxy termination keeps TLS logic and renewal outside the application binary.
- `api-server` can remain HTTP internally without risk — it is not reachable externally.
- WebSocket upgrade (`/ws/v1`) is transparently proxied by Caddy.

## Consequences
- `OPENIEM_ALLOW_INSECURE_HTTP` MUST NOT be set in production; omit or set to `false`.
- `OPENIEM_BIND_ADDR` MUST be `127.0.0.1:8080` in production.
- The proxy MUST forward `X-Forwarded-For` / `X-Real-IP` if rate limiting uses client IP.
- LAN deployments MUST use a trusted local CA (mkcert) so browser clients accept the certificate without security warnings. Self-signed without a trusted CA causes `ERR_CERT_AUTHORITY_INVALID` in browsers and Fetch/WebSocket failures.
- Internet-facing deployments MUST use ACME / Let's Encrypt via Caddy.

## Alternatives Rejected
- **mTLS inside the app** — too complex for musician device onboarding.
- **HTTP only on LAN** — rejected; browsers increasingly block mixed content and HTTP WebSocket on HTTPS pages; ADR-008 mandates TLS.
- **Nginx** — supported but not documented here; Caddy preferred for simplicity and auto-renewal.

## Implementation
See `deployment/caddy/` and `deployment/raspberry-pi/README.md`.
