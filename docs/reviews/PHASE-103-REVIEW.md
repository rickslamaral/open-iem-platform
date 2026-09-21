# Phase 103 Review — OpusReceiver metrics builder and PLC frame wiring

## Escopo

Adicionar suporte opcional de métricas de observabilidade ao `OpusReceiver`
via padrão builder, e registrar frames PLC no path de playout.

## Implementação

- `server/streaming/src/opus_receiver.rs` atualizado com:
  - Novo campo `metrics: Option<Arc<ReceiverMetrics>>` em `OpusReceiver`.
  - Builder `with_metrics(Arc<ReceiverMetrics>) -> Self` para anexar métricas de forma opcional.
  - `record_plc_frame(consecutive)` chamado no path de playout PLC; incrementa `plc_frames_total` e atualiza `plc_consecutive_max` via CAS lock-free.
  - Todas as chamadas guardadas por `if let Some(ref m) = self.metrics`; sem métricas anexadas, comportamento existente é preservado integralmente.

## Evidência

```text
cargo test --manifest-path server/Cargo.toml -p streaming
69/69 testes de streaming passam
plc_frame counter exercitado via testes unitários de ReceiverMetrics no crate observability
commit 7f25428 (local)
```

Evidência: CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e
Raspberry Pi 5 não foram validados.

## Pendências

- Validar métricas PLC em runtime WebRTC/DTLS-SRTP real.
- Validar em hardware PipeWire/ALSA e Raspberry Pi 5.
- Conectar `AppState.metrics.receiver` à construção do `OpusReceiver` no binário headless receiver.
