# Phase 40 Review — segurança operacional do Compose

**Data:** 2026-09-11
**Estado:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Evitar dois erros previsíveis no fluxo local: iniciar imagens Docker obsoletas após mudanças e expor UIs Vite de desenvolvimento em todas interfaces de rede.

## Implementado

- `make up` usa `docker compose up -d --build`.
- Musician UI e Engineer UI publicam portas somente em `127.0.0.1`.
- `make fmt` executa apenas `cargo fmt`; não mascara ausência de scripts frontend inexistentes.
- Documentação obrigatória alinhada ao comportamento real.

## Verificação

- `docker compose config`: PASS.
- `make -n up`: PASS.
- `make test-audio`: PASS — 4 testes.
- `bash scripts/validate-docs.sh`: PASS.
- `bash scripts/validate-skills.sh`: PASS.
- `git diff --check`: PASS.

## Review independente

- Risco de exposição dos UIs e rebuild ausente identificado.
- Correções aplicadas.
- Nenhum shell injection, segredo hardcoded ou comando arbitrário adicionado.

## Limitações

Compose permanece somente para desenvolvimento. API usa HTTP inseguro dentro da configuração isolada; áudio permanece `SIMULATED`. CI remoto falha antes dos steps com `runner_id=0`. Docker runtime, TLS, PipeWire/ALSA, WebRTC media e Raspberry Pi 5 não foram validados.

## Decisão

**PASS WITH CONDITIONS:** mudança local aprovada; merge depende de CI remoto executado com sucesso.
