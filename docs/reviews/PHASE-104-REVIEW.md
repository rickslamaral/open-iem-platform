# Phase 104 Review — Complete receiver metrics wireup (received/dropped/reconnect)

## Escopo

Completar wireup de métricas no `OpusReceiver` registrando pacotes recebidos,
pacotes descartados por overflow do jitter buffer e eventos de reconnect.

## Implementação

- `server/streaming/src/opus_receiver.rs` atualizado com:
  - `record_received()` chamado quando jitter buffer aceita pacote com sucesso.
  - `record_dropped()` chamado quando jitter buffer rejeita pacote por overflow.
  - `record_reconnect()` chamado ao início de `reconnect()`, após reset de estado.
  - Todas as chamadas guardadas por `if let Some(ref m) = self.metrics`; comportamento existente preservado quando métricas não estão anexadas.
- Três novos testes unitários adicionados:
  - `metrics_record_received_on_good_packet` — verifica incremento de `packets_received_total` em push bem-sucedido.
  - `metrics_record_dropped_on_overflow` — verifica incremento de `packets_dropped_total` em overflow do jitter buffer.
  - `metrics_record_reconnect_on_reconnect_call` — verifica incremento de `reconnects_total` ao chamar `reconnect()`.

## Evidência

```text
cargo test --manifest-path server/Cargo.toml -p streaming
69/69 testes de streaming passam (inclui 3 novos testes de métricas)
Revisão independente: PASS
Static scan (clippy): clean
commit 2ac4a77 (local)
```

Evidência: CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e
Raspberry Pi 5 não foram validados.

## Pendências

- Validar métricas recebidas/descartadas/reconnect em runtime WebRTC/DTLS-SRTP real.
- Validar em hardware PipeWire/ALSA e Raspberry Pi 5.
- Conectar `AppState.metrics.receiver` à construção do `OpusReceiver` no binário headless receiver (fora do escopo do api-server).
