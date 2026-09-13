## Task

Closes #<!-- issue number, if applicable -->

## Summary

<!-- What changed and why? -->

## Scope

- [ ] Rust backend/audio
- [ ] Musician UI
- [ ] Engineer UI
- [ ] Deployment/release
- [ ] Documentation only

## Validation

- [ ] `cargo fmt --manifest-path server/Cargo.toml --all -- --check`
- [ ] `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path server/Cargo.toml --workspace`
- [ ] Affected frontend typecheck/tests/build
- [ ] Python tests, if affected
- [ ] Independent security review
- [ ] Independent test review

Commands and real results:

```text

```

## Evidence boundaries

- Local:
- CI:
- Release:
- Installation:
- Hardware/Raspberry Pi 5:
- Audio/media (`SIMULATED` unless physically validated):

## Documentation

- [ ] TODO updated
- [ ] DEVELOPMENT-LOG updated
- [ ] CHANGELOG updated
- [ ] README updated if needed
- [ ] Phase review updated
- [ ] PT/EN/ES guides updated if user-facing

## Risk

<!-- Known limitations, migration, rollback. -->
