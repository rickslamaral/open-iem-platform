# Phase 36 Review — Telemetria no Engineer Console

**Data:** 2026-09-11
**Estado:** implementação local; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Exibir contrato de telemetria já existente no Engineer Console sem transformar `null` em métrica falsa.

## Implementado

- Engineer Console consulta `GET /api/v1/telemetry` junto com snapshot, sessões e assignments.
- Dashboard exibe backend e contador de XRUNs.
- Contador ausente aparece como `UNKNOWN`; VPS continua explicitamente `SIMULATED`.
- Testes frontend atualizados para cobrir resposta de telemetria.

## Verificação

- `npm test --prefix web/engineer -- --run --reporter=dot`: PASS — 2 testes.
- `npm run typecheck --prefix web/engineer`: PASS.
- `npm run build --prefix web/engineer`: PASS — Vite produziu `dist/`.
- `git diff --check`: PASS.
- PipeWire, métricas reais e Raspberry Pi 5 continuam não validados.

## Segurança e limites

- Endpoint permanece protegido por JWT e papel Engineer/Admin no backend.
- Nenhum dado de áudio, XRUN ou disponibilidade foi inventado no frontend.

## Reviews independentes

- Test/code review após correção: **PASS**, sem achados; fallback de telemetria validado.
- Security review após correção: **PASS**, sem achados; JWT/RBAC existente permanece aplicado e sem novo vazamento de token.

## Decisão

**PASS WITH CONDITIONS:** gates locais passam. Merge depende de CI remoto executado com sucesso.

## Gate pendente

CI remoto continua bloqueado antes dos steps com `runner_id=0`. PR #40 não pode ser declarado mergeável nem release pode ser publicada com essa evidência.
