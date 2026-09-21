# Phase 102 Review — PLC frame metrics na ReceiverMetrics observability

## Escopo

Expor contadores PLC na camada de observabilidade:
- `plc_frames_total: AtomicU64` — frames PLC cumulativos.
- `plc_consecutive_max: AtomicU64` — pico de frames PLC consecutivos observado.
- Método `record_plc_frame(consecutive: u32)` — incrementa total e atualiza max via CAS.

## Implementação

**Arquivo modificado:** `server/observability/src/receiver.rs`

Campos em `ReceiverMetrics`:
```rust
plc_frames_total: AtomicU64,
plc_consecutive_max: AtomicU64,
```

`record_plc_frame(consecutive: u32)`:
```rust
pub fn record_plc_frame(&self, consecutive: u32) {
    saturating_inc(&self.plc_frames_total);
    let c = u64::from(consecutive);
    // CAS loop sem lock — nunca bloqueia RT thread
    let mut cur = self.plc_consecutive_max.load(Ordering::Relaxed);
    loop {
        if c <= cur { break; }
        match self.plc_consecutive_max.compare_exchange_weak(
            cur, c, Ordering::Relaxed, Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(v) => cur = v,
        }
    }
}
```

`reset()` zera ambos. `snapshot()` expõe `plc_frames_total` e `plc_consecutive_max`.

`ReceiverSnapshot` recebe os dois campos (serializável via `serde::Serialize`).

## Testes adicionados

| Teste | Cobertura |
|---|---|
| `plc_frame_increments_total` | 2 chamadas → total=2 |
| `plc_consecutive_max_tracks_peak` | Sequência 1,3,2 → max=3 |
| `reset_clears_plc_counters` | reset após record → ambos zero |

## Integração pendente (Phase 103+)

O call site — conectar `OpusReceiver::plc_frames_total()` e `plc_consecutive()` ao
`ReceiverMetrics::record_plc_frame()` no loop de playout — está pendente.
Esta fase apenas expõe a API de observabilidade; o wire-up ocorrerá no
loop de receiver runtime.

## Evidência

```
cargo fmt --manifest-path server/Cargo.toml --all -- --check
cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path server/Cargo.toml
Todos os gates: PASS

Commit [verified]: feat(Phase102): add PLC frame metrics to ReceiverMetrics observability
Branch: feat/phase102-plc-observability-metrics (local, pronto para push)
```

## Pendências

- Wire-up do call site: `record_plc_frame` ainda não é chamado por nenhum produtor em produção.
- Runtime, PipeWire, ALSA, WebRTC/Opus e Raspberry Pi 5 continuam não validados.
- GAP-023 (`RESOLVED CODE+CI/SIMULATED`) permanece: métricas reais dependem de runtime loop.
