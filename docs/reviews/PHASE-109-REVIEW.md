# Phase 109 Review — Receiver fail-safe output failure metrics

## Escopo
Contador `output_failures` adicionado a `ReceiverMetrics` e conectado em todos os pontos onde `OpusReceiver` ativa o latch de mute fail-safe. Cada caminho de falha registra o contador exatamente uma vez.

## Implementação
- `ReceiverMetrics::record_output_failure()` adicionado
- Campo `output_failures: AtomicU64` adicionado a `ReceiverMetrics` com campo de snapshot `output_failures`
- Todos os caminhos de latch de mute em `playout` chamam `record_output_failure()` exatamente uma vez: esgotamento de budget PLC, erro de decode, erro em `output.write`, falha de output em pacote stale, mute de corrida de reconexão
- Teste `metrics_record_output_failure_once_when_plc_budget_exhausts` verifica contagem única no esgotamento de PLC
- `output_failures` incluído em `ReceiverSnapshot` e serialização REST

## Evidência
- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `cargo test --manifest-path server/Cargo.toml`: PASS.
- Streaming: 1 unitário, 0 integração e 0 round-trip PASS.
- Frontends Musician: typecheck, 0 testes e build PASS.
- Frontend Engineer: typecheck, 0 testes e build PASS.
- Revisão independente: PASS.

Evidência permanece CODE. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e
Raspberry Pi 5 não foram validados.

## Pendências
- Validar métricas em runtime WebRTC/DTLS-SRTP real.
- Validar em hardware PipeWire/ALSA e Raspberry Pi 5.
- Integrar ao binário headless receiver quando output OS for implementado.
