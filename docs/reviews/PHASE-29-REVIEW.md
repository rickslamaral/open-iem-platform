# Phase 29 Review — Limitação de falhas de autenticação WebSocket

**Data:** 2026-09-10
**Estado:** implementação local validada; merge bloqueado por CI remoto sem runner
**Escopo:** limitar tentativas inválidas de autenticação em `/ws/v1`.

## Implementação real

- Limiter em memória por IP do peer TCP obtido de `ConnectInfo<SocketAddr>`.
- Limite de 5 falhas em janela de 60 segundos.
- Estado limitado a 4.096 IPs, com remoção do IP menos recentemente observado quando possível.
- Apenas falhas de protocolo ou token inválido consomem orçamento.
- Autenticação bem-sucedida não consome orçamento.
- Bloqueio retorna HTTP 429 e `Retry-After: 5`.
- Cabeçalhos encaminhados não definem identidade do peer.

## Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all --no-fail-fast`: PASS — 231 testes Rust e 3 doc-tests.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- Testes determinísticos cobrem threshold, reset da janela, eviction e concorrência.
- `git diff --check`: PASS.
- `scripts/validate-docs.sh`: PASS na rodada anterior documentada.
- `scripts/validate-skills.sh`: PASS na rodada anterior documentada.

## Segurança e limitações

- Limiter é local ao processo. Não fornece coordenação entre réplicas.
- IP real atrás de proxy exige configuração confiável no proxy; `X-Forwarded-For` não é aceito como prova pelo serviço.
- Revogação pós-emissão de JWT permanece limitada ao TTL do token.
- PipeWire, mídia WebRTC real, latência, XRUN e runtime ARM64 em Raspberry Pi 5 continuam não validados no VPS.
- CI remoto continua bloqueado antes dos steps por ausência/permissão de runner (`runner_id=0`); portanto não há evidência de CI verde nem release pronta.

## Decisão

**PASS local com bloqueio operacional:** código e testes locais passam; merge e release aguardam CI remoto executável.
