# Phase 25 Review — Redação de erros e observabilidade WebSocket

> Follow-up Phase 26: erros após envelope válido agora preservam `request_id`; ver [PHASE-26-REVIEW.md](PHASE-26-REVIEW.md).

## Objetivo
Remover vazamento de detalhes de parser/transportes no protocolo WebSocket e separar encerramentos normais de expiração JWT.

## Status
**PASS WITH CONDITIONS — gates locais passam; CI remoto bloqueado antes dos steps.**

## Implementação
- `ProtocolError` é mapeado para códigos/mensagens públicas estáveis: `INVALID_JSON`, `UNSUPPORTED_VERSION`, `INVALID_REQUEST_ID` e `MESSAGE_TOO_LARGE`.
- Detalhes de `serde_json`, versão recebida e conteúdo do payload não são enviados ao cliente.
- Falhas de `socket.recv()` registram somente `session_id`; erro bruto de transporte não entra no log.
- Fechamento normal do peer termina silenciosamente; timeout de leitura continua emitindo `TOKEN_EXPIRED`.
- Teste unitário valida mensagens públicas sem conteúdo sensível.

## Verificação
- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 218 testes.
- `git diff --check`: PASS.
- Auditoria estática adicionada: sem secrets, `shell=True`, `eval/exec`, `pickle.loads` ou SQL interpolado.

## Segurança
- Nenhum secret adicionado.
- Logs não incluem JWT, payload, erro bruto de transporte ou detalhes de parser.
- Ainda pendente: política formal de retenção/serialização de logs e correlação de `request_id` em erros após envelope validado.

## Limitações
CI GitHub Actions falha imediatamente com todos os jobs sem steps executados; runner/permissão continua bloqueador externo. PipeWire, mídia WebRTC real e runtime ARM64 Raspberry Pi continuam não validados no VPS.

## Próximo
Corrigir correlação de `request_id` para erros depois de envelope validado. Depois implementar ressincronização explícita quando broadcast perder eventos por lag.
