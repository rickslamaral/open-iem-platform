# Phase 31 Review — Revogação pós-emissão de JWT

**Data:** 2026-09-10
**Estado:** PR #40 aberto; implementação commitada na branch; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Invalidar access tokens já emitidos quando sessão, usuário ou família de refresh for revogada, incluindo conexões WebSocket estabelecidas.

## Implementado

- Mapeamentos persistidos de access-session vinculam `jti`, usuário e sessão de refresh.
- Middleware valida mapeamento, usuário, expiração e estado de revogação.
- Rotação, logout, revogação administrativa, replay e remoção de usuário invalidam mapeamentos relacionados.
- WebSocket revalida sessão em mensagens recebidas e ticks de keepalive; revogação retorna `SESSION_REVOKED` e encerra o loop.
- Rollback de refresh não reativa sessão revogada concorrentemente e não consome token quando usuário não existe.
- Respostas HTTP internas não expõem detalhes de erro.
- Cliente Musician preserva snapshot REST atrasado como baseline quando revisão WebSocket já avançou.

## Verificação local

- `cargo fmt --all -- --check`: PASS.
- `cargo test -p api-server --test integration`: PASS — 57 testes.
- `cargo clippy -p api-server --all-targets -- -D warnings`: PASS.
- Workspace completo: BLOCKED pela dependência nativa ausente `jack.pc`/`jack-sys`.
- Frontend Musician: PASS — 34 testes, typecheck e build.
- `git diff --check`: PASS.

## Segurança

**Sem BLOCKER/HIGH identificado na revisão local atual.** Revogação é fail-closed. Tokens e detalhes internos não entram em URL nem resposta pública.

## CI e release

**BLOCKED.** Runs 34532261471 e 34532315872 falharam imediatamente, com `steps=[]` e `runner_id=0`. Token atual não permite consultar configurações/logs de Actions (`HTTP 403`). Não há evidência de CI verde, artefatos ou release pronta.

## Hardware

PipeWire, ALSA real, WebRTC media, latência e runtime Raspberry Pi 5 ARM64 continuam `SIMULATED`/`HARDWARE VALIDATION REQUIRED`.

## Decisão

**PASS WITH CONDITIONS:** implementação local aprovada; merge depende de CI remoto executável e revisão do PR #40. Não avançar release.

## Próximo

Desbloquear infraestrutura/permissão do GitHub Actions. Depois validar CI real e seguir com CLI `iem` somente após especificação documental.
