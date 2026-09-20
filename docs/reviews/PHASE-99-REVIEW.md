# Phase 99 Review — Smoke PipeWire virtual no CI

## Escopo

Integrar smoke de grafo virtual PipeWire/WirePlumber ao job CI
`Audio Lab L1/L2 (SIMULATED)`, com dbus session privado e cleanup
seguro de processos por identidade.

## Implementação

- `scripts/ci/run-pipewire-software-e2e.sh` atualizado para:
  - Criar `dbus-daemon` de sessão privado (sem dependência de `$DISPLAY`).
  - Validar `timeout --foreground` disponível antes de iniciar processos.
  - Identificar cada daemon por `starttime` via `/proc/<pid>/stat`.
  - Cleanup usa `pidfd_open`/`pidfd_send_signal` para evitar sinalização de PID reciclado.
- CI job envolve smoke em `timeout --foreground 120s`.
- `docs/DEVELOPMENT-LOG.md` registra correções de session bus, cleanup e limite de timeout.

## Evidência

```text
CI remoto (PR #186, run audio-lab): COMPLETED/SUCCESS no SHA 93bef11
CI remoto (PR #187, reconciliação): 16/16 SUCCESS no SHA 6254e5e
bash -n: scripts/ci/run-pipewire-software-e2e.sh — syntax OK
Local: PIPEWIRE_SOFTWARE_E2E: BLOCKED (pw-cli ausente no host local)
```

Evidência permanece SOFTWARE/SIMULATED. Enumeração de nós CI não prova fluxo de
áudio real, WebRTC/Opus, runtime ou hardware físico.

## Pendências

- `pw-cli` ausente no host local; execução validada somente em runner CI.
- PipeWire runtime, WebRTC/Opus em rede e Raspberry Pi 5 continuam pendentes.
- Nenhum claim de produção com base neste smoke.
