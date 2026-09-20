# Phase 97 Review — Hardening de gates PipeWire/Opus simulados

## Escopo

Hardening dos gates simulated de PipeWire e Opus no CI, incluindo smoke script
de grafo virtual sink/source e round-trip determinístico.

## Implementação

- `scripts/ci/run-pipewire-software-e2e.sh` criado: inicia PipeWire/WirePlumber
  com runtime privado, cria nós null sink/source via `pw-cli` e valida
  propriedades (`node.name`, `media.class`, `audio.rate=48000`, `audio.channels=2`).
- `server/streaming/tests/opus_roundtrip.rs`: round-trip Opus determinístico de
  um pacote adicionado com codificação/decodificação via `audiopus`.
- Stability gate (`scripts/run-software-release-gates.sh`) corrigido para falhar
  fechado quando qualquer passo retorna código não-zero.
- `docs/HEADLESS-AUDIO-TESTING.md` atualizado com nível de evidência
  SOFTWARE/SIMULATED.

## Evidência

```text
cargo test --manifest-path server/Cargo.toml  — todos os testes existentes passam
bash -n scripts/ci/run-pipewire-software-e2e.sh  — syntax OK
Local: PIPEWIRE_SOFTWARE_E2E: BLOCKED (pw-cli ausente no host de CI local)
CI remoto (PR #183): CI 16/16 SUCCESS no SHA 0b1a4cd
```

Evidência permanece SOFTWARE/SIMULATED. PipeWire runtime, WebRTC/Opus em rede,
hardware e Raspberry Pi 5 não foram validados.

## Pendências

- Executar smoke contra PipeWire real com hardware conectado (L3/L4).
- Validar round-trip Opus em rede WebRTC real.
- Não declarar runtime ou suporte de hardware com base nesta fase.
