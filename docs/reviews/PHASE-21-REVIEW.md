# Phase 21 Review — Autenticação WebSocket sem token em URL

**Status:** PASS WITH CONDITIONS
**Data:** 2026-09-09
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

## Objetivo

Remover access token de query string durante upgrade WebSocket.

## Implementado

- `/ws/v1` extrai JWT de `Sec-WebSocket-Protocol: openiem.bearer.<JWT>, openiem.v1`.
- `WebSocketUpgrade::protocols` seleciona e ecoa somente `openiem.v1`; bearer nunca retorna no handshake.
- Query string não é consultada para autenticação.
- Parser exige exatamente dois protocolos: um `openiem.bearer.<JWT>` e `openiem.v1`; rejeita extras, duplicatas, headers repetidos e entrada acima de 4096 bytes.
- Cliente Musician usa `new WebSocket('/ws/v1', ['openiem.bearer.<JWT>', 'openiem.v1'])`.
- REST mantém `Authorization: Bearer`.

## Verificação

- Rust fmt: PASS.
- Rust unitários: PASS — 19 testes.
- Rust integração: PASS — 41 testes.
- Rust clippy `-D warnings`: PASS.
- Musician: 32 testes, typecheck e build: PASS.
- Engineer: 2 testes, typecheck e build: PASS.
- `git diff --check`: PASS.

## Segurança

Token deixa de aparecer em URL, reduzindo exposição em logs de proxy, histórico e métricas de URL. Origin validation, JWT expiry, RBAC, ownership, rate limit e limite de mensagem permanecem ativos.

## Condições e limitações

Subprotocolo carrega credencial durante handshake e exige TLS em produção. TLS fail-closed ainda depende de deployment reverse proxy no Raspberry Pi 5. PipeWire, Opus, mídia WebRTC real, telemetria e runtime ARM64 continuam SIMULATED/não validados no VPS.

## Próximo

Validar reverse proxy TLS e execução ARM64 em Raspberry Pi 5 real.
