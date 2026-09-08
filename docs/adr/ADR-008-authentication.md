# ADR-008: Authentication Mechanism

## Status
Proposed — PENDING DECISION

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
**DEFERRED** — requires security review before Phase 3.

## Recommendation for Evaluation
Given local LAN context, JWT with username/password login is likely appropriate:
- Standard, well-understood
- Stateless verification (no DB lookup per request)
- Expiry support
- Works with WebSocket (token in header or URL param — prefer header)
- No complex PKI infrastructure on Raspberry Pi

## Constraints to Resolve
- [ ] Token lifetime (session duration for a live show: 8-12 hours?)
- [ ] Token refresh mechanism
- [ ] Secure token storage on mobile PWA (localStorage vs Cookie)
- [ ] HTTPS required? (LAN self-signed cert vs HTTP)
- [ ] WebSocket auth: token in initial HTTP upgrade header (preferred) or per-message?

## Security Requirements (Non-Negotiable)
- Backend authorization enforced on every endpoint and every WS message
- UI hiding is NOT security
- Musician CANNOT access other musician's mix (server-enforced)
- Token expiry must be enforced

## Next Step
Security-review skill + architect-designer to resolve by Phase 3 start.
