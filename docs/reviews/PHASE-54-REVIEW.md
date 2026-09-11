# Phase 54 Review — provenance de artefatos de release

**Data:** 2026-09-11
**Status:** PASS WITH CONDITIONS — implementação local; PR #40 aberto; CI remoto bloqueado; não mergeado.

## Objetivo

Adicionar evidência verificável de provenance aos archives de servidor antes da publicação, sem conceder permissões amplas aos jobs de build.

## Implementado

- `actions/attest-build-provenance@v2` fixada por SHA completo.
- Attestation aplicada aos archives x86_64 e ARM64 depois da validação estrutural e antes do upload.
- Jobs de build recebem somente `id-token: write` e `attestations: write`, além de `contents: read`.

## Verificação local

- `python3 -m pytest -q tests/test_validate_release_archive.py`: PASS — 7 testes.
- `make test`: PASS — Rust, frontends e harness `SIMULATED`.
- `make validate`: PASS — documentação, PDF e skills.
- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --workspace --all-targets -- -D warnings`: PASS.
- `git diff --check`: PASS.

## Segurança e limites

Não há secrets no diff nem permissões de escrita de conteúdo nos jobs de build. A attestation ainda não foi publicada/verificada porque GitHub Actions falha antes dos steps (`runner_id=0`). Checksum permanece útil para integridade, mas attestation depende do fluxo remoto executado. Hardware Raspberry Pi, PipeWire/ALSA e mídia WebRTC continuam não validados.

## Decisão

PASS WITH CONDITIONS. Validar attestation no GitHub após desbloquear runner. Não fazer merge ou release enquanto CI remoto não executar gates reais.
