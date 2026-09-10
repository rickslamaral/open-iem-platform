# Phase 27 Review — WebSocket state recovery

## Objetivo

Recuperar estado autoritativo quando um cliente perde eventos de broadcast e corrigir suporte do Musician para atualizações de master.

## Status

**PASS WITH CONDITIONS** — implementação e gates locais passam; CI remoto segue bloqueado antes dos steps por indisponibilidade/permissão de runner.

## Implementado

- `RecvError::Lagged` nos canais de send e master envia envelope `State` com revisão autoritativa.
- Musician refaz `GET /api/v1/state` autenticado ao receber `State`.
- Requests de snapshot concorrentes são enfileirados em uma única nova tentativa, evitando perder sinal de ressincronização enquanto request anterior está em voo.
- `MasterAck` é validado e aplicado ao mix correspondente no snapshot local.
- Lock de estado poisoned encerra ressincronização com log, sem panic no handler.

## Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 218 testes Rust e 3 doc-tests.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- Musician typecheck, 33 testes e build: PASS.
- `scripts/validate-docs.sh`: PASS.
- `scripts/validate-skills.sh`: PASS.
- `git diff --check`: PASS.
- Static scan do diff: nenhum secret ou padrão perigoso encontrado.

## Revisão independente

- Primeiro review encontrou risco real: refresh solicitado durante snapshot em voo era descartado. Corrigido com `snapshotRefreshQueued`.
- Segundo review encontrou risco de panic em `state.control.lock().expect(...)` no caminho de recuperação. Corrigido com tratamento fail-closed.
- Re-review posterior: sem novos blockers de segurança ou lógica no diff corrigido.

## Limitações

- CI GitHub remoto falha antes dos steps; artefatos e merge não podem ser declarados validados.
- PipeWire, mídia WebRTC real e runtime ARM64 em Raspberry Pi continuam `SIMULATED`/não validados.
- Revogação pós-emissão de JWT, quotas por usuário/IP e teste temporal do loop WebSocket seguem pendentes.

## Decisão

Código local pronto para commit e PR. Merge bloqueado até CI remoto executar todos os gates e passar.
