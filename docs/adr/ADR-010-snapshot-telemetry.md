# ADR-010 — Contrato de snapshot e telemetria

**Status:** Accepted
**Data:** 2026-09-09

## Decisão

`GET /api/v1/state` expõe snapshot versionado do estado de controle. O snapshot usa DTOs explícitos, omite slots não configurados e filtra mixes para `MUSICIAN` pelo assignment persistido. `ENGINEER` e `ADMIN` veem estado completo.

`GET /api/v1/telemetry` exige `ENGINEER` ou `ADMIN`. No VPS retorna `availability=simulated` e `null` para métricas sem backend de áudio conectado.

## Consequências

- Engineer Console ganha contrato estável sem serializar structs internos de DSP.
- Nenhum valor de telemetria é inventado.
- PipeWire, Opus, XRUN e frames reais continuam dependentes de Raspberry Pi 5.
- Alterações futuras exigem incremento de `schema_version` quando incompatíveis.
