# Phase 106 Review — Receiver metrics full wire-up: ingress, reconnect, PLC and REST snapshot

## Escopo
Todos os contadores de métricas do receiver foram conectados ao `OpusReceiver`: `packets_received`, `packets_dropped` (QueueFull, InvalidPacket, drain de reconexão), `reconnect_count`, `plc_frames_total` e `plc_consecutive_max`. Os contadores são expostos no snapshot REST com `schema_version`. Teste de integração REST confirma os valores dos contadores.

## Implementação
- `OpusReceiver::with_metrics(Arc<ReceiverMetrics>) -> Self` builder adicionado
- `enqueue`: registra `dropped` para `InvalidPacket` (vazio/oversized) e `QueueFull`
- `playout`: registra `received` quando `jitter.push` tem sucesso; `dropped` para pacotes stale (`sequence < expected`) e overflow do jitter; drain de reconexão registra `dropped` para cada pacote descartado
- `reconnect`: registra `reconnect_count` uma vez; drena ingress registrando cada pacote descartado
- Caminho PLC: registra `plc_frame(consecutive)` e `output_failures` no latch de mute
- Teste de integração `metrics_exposes_receiver_counters` confirma todos os contadores serializados na resposta REST

## Evidência
- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `cargo test --manifest-path server/Cargo.toml`: PASS.
- Streaming: 71 unitários, 1 integração e 1 round-trip PASS.
- Frontends Musician: typecheck, 0 testes e build PASS.
- Frontend Engineer: typecheck, 0 testes e build PASS.
- Revisão independente: PASS.

Evidência permanece CODE. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e
Raspberry Pi 5 não foram validados.

## Pendências
- Validar métricas em runtime WebRTC/DTLS-SRTP real.
- Validar em hardware PipeWire/ALSA e Raspberry Pi 5.
- Integrar ao binário headless receiver quando output OS for implementado.
