# Contrato API — Snapshot e telemetria

**Versão:** 1 · **Estado:** implementado no backend · **Data:** 2026-09-09

## Snapshot

`GET /api/v1/state` exige JWT e retorna estado de controle consistente, capturado sob um único lock.

```json
{
  "schema_version": 1,
  "revision": 2,
  "channels": [],
  "mixes": []
}
```

`channels` contém somente slots configurados. Cada canal expõe `index`, `id`, `name`, `gain_db`, `muted`, `locked`, `enabled` e `revision`.

`mixes` contém somente slots configurados. Cada mix expõe `index`, `id`, `name`, `master_gain_db`, `master_muted`, `revision` e `sends`. Cada send expõe `channel_index`, `channel_id`, `mix_id`, `gain_db`, `pan`, `muted`, `solo`, `enabled`, `locked` e `revision`.

| Papel | Visibilidade |
|---|---|
| MUSICIAN | canais configurados e mix atribuído; sem assignment, nenhum mix |
| ENGINEER | canais e todos mixes configurados |
| ADMIN | canais e todos mixes configurados |

`revision` é contador global monotônico do engine. Revisões de canal, mix e send são contadores locais.

## Telemetria

`GET /api/v1/telemetry` exige papel `ENGINEER` ou `ADMIN`.

No VPS, resposta real:

```json
{
  "schema_version": 1,
  "availability": "simulated",
  "backend": "simulated",
  "sample_rate_hz": null,
  "frames_processed": null,
  "xrun_count": null
}
```

`null` significa métrica não conectada. Zero não representa dado desconhecido. PipeWire, Opus, XRUN e frames reais permanecem pendentes até validação no Raspberry Pi 5.

Fora do contrato: medidores RMS/peak/LUFS, histórico, Prometheus, SSE/WebSocket de telemetria e métricas de rede.
