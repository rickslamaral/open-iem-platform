---
description: Validate Open IEM release readiness, artifacts, signatures, CI evidence, and hardware claims.
argument-hint: [version or tag]
disable-model-invocation: true
---

# Release Readiness

Use `.claude/references/release-checklist.md` as canonical checklist.

- Verify version/tag/manifests agree.
- Require real CI job execution, not immediate runner failures.
- Validate x86_64/ARM64 archives, checksums, signatures, fingerprint, and SBOM.
- Keep SIMULATED audio and unvalidated RPi5 status explicit.
- Never create tags, publish releases, push, or alter visibility without confirmation.
