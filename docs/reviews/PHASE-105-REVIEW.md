# Phase 105 Review — Receiver ingress and stale-packet drop metrics

## Escopo

Completar contagem de descartes em `OpusReceiver`, cobrindo overflow da fila
de ingress, overflow do jitter buffer e pacotes atrasados removidos durante
`playout`.

## Implementação

- `enqueue` chama `ReceiverMetrics::record_dropped()` somente para `QueueFull`;
  canal desconectado não é contado como descarte.
- `playout` contabiliza overflow do jitter e pacotes stale (`sequence < expected`).
- Testes cobrem overflow ingress, overflow jitter e pacote stale.

## Evidência

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `cargo test --manifest-path server/Cargo.toml`: PASS.
- Streaming: 71 unitários, 6 integração e 2 round-trip PASS.
- Frontends Musician: typecheck, 61 testes e build PASS.
- Frontend Engineer: typecheck, 46 testes e build PASS.
- Revisão independente: PASS após correção de subcontagem stale.

Evidência permanece CODE. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e
Raspberry Pi 5 não foram validados.
