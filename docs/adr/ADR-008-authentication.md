# ADR-008: Authentication Mechanism

## Status
Accepted — Phase 3 security baseline (2026-09-08)

## Context
Open IEM Platform operates on a **local LAN** (not internet-facing). Musicians and engineers connect from phones and laptops on the same Wi-Fi network as the server. The system requires RBAC (ADMIN, ENGINEER, MUSICIAN roles).

Threat model context:
- Primary threat: unauthorized mix modification by another musician
- Secondary threat: accidental access to wrong mix
- Not primary threat: external internet attackers (system is LAN-only)

Authentication options:
1. **Pre-shared tokens** — simple, static tokens per role/user
2. **JWT (JSON Web Tokens)** — stateless, expiry support, standard
3. **Session cookies + HTTPS** — browser-native, requires TLS
4. **mTLS** — mutual TLS, highest security, complex setup on RPi
5. **Username/password → JWT** — standard web auth with token issuance

## Decision
Use username/password login with short-lived Ed25519-signed JWT access tokens and opaque, rotated refresh tokens persisted server-side.

- Access token: 15-minute lifetime; claims limited to `sub`, `role`, `iat`, `exp`, `jti`, issuer and audience. Algorithm and issuer/audience are fixed during verification.
- Refresh token: 12-hour lifetime, cryptographically random, stored hashed in SQLite, rotated on every use. Reuse revokes its token family. Logout and user revocation invalidate active refresh tokens.
- Passwords: Argon2id with unique salt and versioned parameters. Never store or log plaintext passwords, tokens, or hashes.
- Browser storage: `HttpOnly`, `Secure`, `SameSite=Strict` cookies. Never `localStorage`, `sessionStorage`, or URL query parameters.
- Transport: HTTPS required for login and authenticated traffic, including WebSocket upgrade. HTTP is rejected, not silently trusted because traffic is on LAN.
- WebSocket: authenticate during HTTP upgrade, bind connection to identity, authorize every message, enforce message size/rate limits, and close on session expiry or revocation.
- Mutating browser requests: validate `Origin` and use CSRF protection where cookie authentication alone is insufficient.

## Constraints Resolved
- [x] Token lifetime: 15-minute access; 12-hour refresh, covering an 8–12 hour show.
- [x] Token refresh: rotating opaque refresh token with reuse detection.
- [x] Secure token storage: `HttpOnly` + `Secure` + `SameSite=Strict` cookies.
- [x] HTTPS: mandatory; provision a trusted local certificate/CA for target hardware.
- [x] WebSocket auth: cookie during initial HTTPS upgrade; never token in URL.

## Implementation dependencies
The backend must use a maintained JWT crate configured for EdDSA, Argon2id password hashing, a CSPRNG for refresh tokens, and secret-safe handling. Dependency versions require `cargo audit` before production release.

## Security Requirements (Non-Negotiable)
- Backend authorization enforced on every endpoint and every WS message
- UI hiding is NOT security
- Musician CANNOT access other musician's mix (server-enforced)
- Token expiry must be enforced

## Next Step
Security-review skill + architect-designer to resolve by Phase 3 start.
