# Security Policy

## Supported Versions

| Phase | Version | Security Support |
|-------|---------|-----------------|
| Phase 0 (Bootstrap) | 0.0.x | Development only |
| Phase 3+ (Backend) | 0.x.x | Active |

## Reporting a Vulnerability

**Do not report security vulnerabilities via public GitHub issues.**

To report a security vulnerability:

1. Email the maintainer directly (see GitHub profile)
2. Include: description, reproduction steps, impact assessment
3. Allow reasonable time for a fix before public disclosure

## Security Model

Open IEM Platform is designed for **local LAN deployment**. The security model assumes:

- The server is on a trusted local network (live sound venue)
- Internet exposure is not supported and not recommended
- Physical access to the server is controlled by the venue/engineer

### Roles

```
ADMIN     — System configuration, user management
ENGINEER  — All mixes, devices, scenes, channel locking
MUSICIAN  — Own assigned mix only
```

### Authorization Guarantees

- Every API endpoint enforces role-based authorization
- Every WebSocket message enforces authorization
- UI hiding is NOT security — all rules enforced server-side
- Musician cannot access another musician's mix (server-enforced)
- Locked channels cannot be modified by musician role

## Security Requirements for Contributors

- No credentials or secrets in source code
- No credentials in committed configuration files
- All SQL queries must be parameterized (no string interpolation)
- All API inputs must be validated and bounded
- Audio gain values must be bounded (no unbounded amplification)
- Dependencies must pass `cargo audit` and `npm audit`

## Audio Safety

This is an audio product used in live performance:
- Limiter mandatory on master outputs
- Gain bounded in all paths
- Safe startup: mute until state verified
- Any gain/routing change requires review for loud-output risk

Unsafe audio behavior (unexpected loud output) is treated as a **security issue**.
