# Phase 28 Review — Quotas de conexões WebSocket

## Objetivo

Impedir que um usuário ou endereço IP consuma sozinho o teto de conexões WebSocket do processo.

## Status

**PASS WITH CONDITIONS** — gates locais passam; merge continua bloqueado enquanto GitHub Actions falhar antes de alocar runner.

## Implementado

- `WebSocketQuota` reserva globalmente até 64 conexões, 4 por `user_id` e 16 por `IpAddr`.
- Reserva é atômica sob mutex; falha em qualquer limite não altera contadores.
- `WebSocketQuotaGuard` libera contadores via RAII quando `handle_socket` termina.
- `ws_handler` usa `ConnectInfo<SocketAddr>` e não confia em `X-Forwarded-For`.
- Limite global retorna HTTP 503; limites por usuário/IP retornam HTTP 429, com `Retry-After: 5`.
- Servidor real usa `into_make_service_with_connect_info::<SocketAddr>()`.

## Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 31 testes unitários do `api-server`, 52 testes de integração e demais crates PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `scripts/validate-docs.sh`: PASS.
- `scripts/validate-skills.sh`: PASS.
- `git diff --check`: PASS.
- Scan estático: nenhum secret ou padrão perigoso encontrado.
- Reviews independentes: `passed=true`, sem blockers de segurança ou lógica.

## Limitações

Quota é local ao processo. Múltiplas instâncias exigem coordenador compartilhado. NAT pode agrupar clientes no mesmo IP. Tentativas com JWT inválido continuam sem rate limit por IP. Revogação pós-emissão de JWT permanece pendente. PipeWire, mídia WebRTC real e runtime ARM64 em Raspberry Pi não foram validados.

## CI/Merge

Runs recentes da PR #39 falham antes de qualquer step, com `runner_id=0`, `runner_name` vazio e logs vazios. Causa é configuração/permissão/billing do GitHub Actions, não código ou workflow local. Não declarar merge ou release até runner executar todos os gates.
