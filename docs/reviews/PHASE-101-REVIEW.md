# Phase 101 Review — PLC Concealment no OpusReceiver com fail-safe limitado

## Escopo

Implementar Packet Loss Concealment (PLC) bounded no `OpusReceiver`:
- Gerar até `PLC_MAX_CONSECUTIVE=4` frames de 20 ms via decodificador Opus em modo FEC/PLC.
- Após exaurir orçamento, travar estado `Muted` fail-safe preservando o pacote recebido para ressincronização explícita.
- Enforce duração fixa de 20 ms para frames MVP; frames variáveis falham fechado.
- Corrigir `reconnect()` para não apagar jitter buffer, preservando pacote pós-gap.

## Implementação

**Arquivo modificado:** `server/streaming/src/opus_receiver.rs`

Constantes adicionadas:
```rust
const PLC_FRAME_SAMPLES: usize = 960;       // 20 ms @ 48 kHz
const PLC_MAX_CONSECUTIVE: u32 = 4;         // orçamento: 80 ms total
```

Campos novos em `OpusReceiver`:
```rust
plc_consecutive: u32,       // frames PLC desde último decode válido
plc_frames_total: u64,      // acumulador (saturating, nunca reseta)
```

Lógica PLC em `playout()`:
1. Gap detectado (`sequence > expected`): se `plc_consecutive < PLC_MAX_CONSECUTIVE`, incrementa budget antes do decode e chama `decoder.decode(&[], PLC_FRAME_SAMPLES, ...)`.
2. Falha de decoder PLC → `output_failed = true` → mute permanente.
3. Amostras ≠ `PLC_FRAME_SAMPLES` → mesma proteção fail-closed.
4. Budget exaurido → `output_failed = true`, preserva `next_sequence = Some(sequence)` para o pacote recebido aguardar no jitter buffer.
5. `reconnect()`: remove `jitter.packets.clear()`, reseta apenas `plc_consecutive = 0`.
6. Decode válido: `plc_consecutive = 0`.

Accessors públicos: `plc_consecutive() -> u32`, `plc_frames_total() -> u64`.

## Testes adicionados

| Teste | Cobertura |
|---|---|
| `plc_triggers_on_missing_packet` | PLC dispara em gap de 1, estado Playing, contador=1 |
| `plc_reset_on_good_decode` | Contador zera após decode válido pós-PLC |
| `plc_budget_exhaustion_mutes` | 4 PLCs → Playing; 5ª chamada → OutputFailed latched |
| `plc_counter_reset_on_reconnect` | `reconnect()` zera `plc_consecutive` |
| `reconnect_preserves_queued_packet_for_resynchronization` | Exaustão PLC → mute → reconnect → decode do pacote preservado → Playing |

## Evidência

```
cargo test --manifest-path server/Cargo.toml -p streaming
66 testes, 0 falhas

cargo test --manifest-path server/Cargo.toml
88 testes integração, 0 falhas

cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings
0 warnings

cd web/musician && npm run typecheck && npm test -- --watchAll=false && npm run build
61 testes, build OK

cd web/engineer && npm run typecheck && npm test -- --watchAll=false && npm run build
46 testes, build OK

PR #203 CI: 16/16 COMPLETED/SUCCESS
```

Revisão independente realizada em dois rounds: round 1 identificou risco de
`reconnect()` apagar pacote preservado; corrigido em commit adicional.
Round 2: PASS.

## Pendências

- PLC é CODE/SIMULATED: sem validação de rede real, PipeWire, ALSA ou hardware.
- Runtime WebRTC/DTLS-SRTP, Raspberry Pi 5 e OS audio output permanecem pendentes (GAP-003, GAP-017).
- `plc_frames_total` no receiver ainda não é conectado ao `ReceiverMetrics` observability (wire-up previsto como Phase 103+).

## Nota: PLC_FRAME_SAMPLES=960

`PLC_FRAME_SAMPLES=960` refere-se a amostras **por canal** conforme API do decodificador Opus
(`OpusDecoder::decode` retorna samples/canal). O buffer PCM é dimensionado como
`960 * 2 = 1920` floats (estéreo interleaved) via `MAX_DECODED_SAMPLES * 2`.
A constante está correta; o par de campos é coerente com a API.
