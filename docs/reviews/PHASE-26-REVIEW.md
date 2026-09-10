# Phase 26 Review — Correlação de erros WebSocket

## Objetivo
Preservar `request_id` em erros gerados depois de envelope WebSocket válido, mantendo `server` para falhas de parsing sem ID confiável.

## Implementação
- `ws.rs` clona `request_id` somente após `decode_client_message` validar envelope.
- Erros `FORBIDDEN` por RBAC e ownership usam ID recebido.
- Erros de JSON, versão, ID inválido, tamanho e frames binários continuam sem correlação de requisição e usam `server`.

## Verificação
- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 219 testes.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `git diff --check`: PASS.
- Reviews independentes encontraram bypass de liveness por Pong não solicitado e replay de desafio; corrigidos com payload novo por Ping e teste unitário de correspondência.
- Frontends Musician/Engineer: typecheck, testes e build PASS nesta rodada.

## Limitações
CI GitHub Actions continua falhando antes dos steps por runner/permissão. PipeWire, mídia WebRTC real e runtime ARM64 Raspberry Pi continuam não validados no VPS.
