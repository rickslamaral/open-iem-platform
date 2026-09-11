# Phase 48 Review — pinning imutável das actions

**Data:** 2026-09-11
**Estado:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Eliminar referências mutáveis de actions de terceiros nos workflows de CI e release.

## Implementado

- `actions/checkout`, `actions/cache`, `actions/setup-node`, `actions/upload-artifact`, `actions/download-artifact` e `softprops/action-gh-release` foram fixados em commits SHA completos.
- `dtolnay/rust-toolchain` foi fixado no commit atual da branch `stable`.
- Comentários mantêm versão humana sem reintroduzir tags mutáveis.

## Verificação

- SHAs confirmados com `git ls-remote` nos repositórios upstream.
- `make test`: PASS — Rust, frontend e harness de áudio determinístico.
- `bash scripts/validate-docs.sh`, `bash scripts/validate-pdf.sh`, `bash scripts/validate-skills.sh` e `git diff --check`: PASS.

## Limitações

CI remoto segue falhando antes dos steps com `runner_id=0` e `steps=[]`; após push, runs `34629014531` (PR) e `34629009541` (push) falharam. Hardware PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 continuam não validados.

## Decisão

**BLOCKED:** pinning concluído. Merge aguarda runner GitHub Actions funcional e checks remotos verdes.
