# Phase 34 Review — CLI output correctness

**Data:** 2026-09-10
**Estado:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Corrigir riscos e inconsistências do binário `open-iem-admin` sem ampliar contrato do CLI geral `iem`.

## Implementado

- URLs HTTP aceitas somente para loopback; hosts remotos exigem HTTPS antes de requests autenticados.
- Renderização de arrays JSON usa união de chaves em ordem de primeira ocorrência, sem perder colunas de objetos posteriores.
- HTTP 404 retorna `Resource not found`, sem mensagem obsoleta de endpoint planejado.
- Testes unitários cobrem união de chaves e linhas não-objeto.

## Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml -p admin-cli`: PASS — 2 testes.
- `cargo clippy --manifest-path server/Cargo.toml -p admin-cli --all-targets -- -D warnings`: PASS.
- `scripts/validate-docs.sh`: PASS.
- `scripts/validate-skills.sh`: PASS.
- `git diff --check`: PASS.
- Review independente: primeira rodada BLOCKER por Bearer em HTTP remoto; correção aplicada. Segunda rodada obrigatória antes do commit.

## Limitações

- CI GitHub falha antes de steps, com `runner_id=0`; não representa falha de código executado.
- Workspace completo depende de `jack.pc`, ausente no VPS.
- Áudio real, PipeWire, mídia WebRTC e Raspberry Pi 5 não validados.

## Decisão

**PASS WITH CONDITIONS:** código e documentação locais passam; merge bloqueado até review final e CI remoto executado com sucesso.
