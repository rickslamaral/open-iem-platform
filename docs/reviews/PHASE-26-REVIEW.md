# Phase 26 Review — Resiliência WebSocket e limites de transporte

## Objetivo

Validar que mensagens WebSocket acima de 16 KiB sejam rejeitadas pelo transporte antes do parser da aplicação.

## Implementado

- Erros `FORBIDDEN` gerados após envelope válido preservam `request_id`; falhas de parsing usam `server`.
- `KeepaliveTracker` mantém desafio pendente, valida Pong correlacionado e evita falso timeout após Pong válido.
- `WebSocketUpgrade` aplica `max_message_size(16 KiB)` e `max_frame_size(16 KiB)`.
- Teste de integração envia mensagem Text não fragmentada com `16 * 1024 + 1` bytes.
- O teste confirma término do recebimento por erro de transporte; `axum-test` expõe reset sem handshake de fechamento para esse caso.

## Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --package api-server --test integration ws_oversized_text_message_is_rejected_by_upgrade_limit -- --nocapture`: PASS após alinhar asserção ao comportamento real do `axum-test`.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `git diff --check`: PASS.
- Scanner prescrito `/root/scan_patterns.py`: indisponível; nenhum resultado inventado.

## Revisão independente

- Test Agent: identificou que `WsMessage::receive_message()` entra em panic quando o peer reseta sem handshake; teste foi ajustado para capturar erro no task e validar encerramento do transporte.
- Security/Code Review: configuração de limite está correta; cobertura agora prova mensagem Text oversized. Não há concerns de segurança ou erros lógicos nesta alteração.

## Limitações

- Frame binário oversized e mensagem fragmentada com tamanho agregado acima de 16 KiB ainda não têm testes dedicados.
- CI GitHub continua falhando antes dos steps por runner/permissão.
- PipeWire, mídia WebRTC real e runtime ARM64 no Raspberry Pi permanecem `SIMULATED`/não validados.

## Decisão

**PASS WITH CONDITIONS.** Alteração é segura e testada localmente. Não declarar release até CI remoto executar gates reais.
