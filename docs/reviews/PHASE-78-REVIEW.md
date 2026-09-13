# Phase 78 Review — rejeição de dados residuais em archives

**Data:** 2026-09-13
**Status:** PASS WITH CONDITIONS — validação local aprovada; CI remoto e hardware pendentes.

## Objetivo

Impedir que dados anexados depois de um `tar.gz` válido sejam aceitos sem validação. Isso inclui bytes residuais e streams gzip concatenados.

## Implementado

- O validador percorre o stream gzip completo com `zlib` após validar e consumir o TAR.
- Stream gzip truncado, inválido, com bytes residuais ou com segundo stream gzip falha fechado.
- Limite de descompressão permanece aplicado durante essa verificação.
- Testes offline cobrem bytes residuais e stream gzip concatenado.

## Gates locais

- `python3 -m pytest -q tests/test_validate_release_archive.py`: 24 aprovados.
- `python3 -m py_compile scripts/validate-release-archive.py tests/test_validate_release_archive.py`: aprovado.
- `git diff --check`: aprovado.

## Segurança

A correção impede conteúdo fora do primeiro stream gzip de permanecer fora do conjunto validado e eventualmente ser carregado por consumidores permissivos.

## Limitações

CI remoto segue falhando antes dos steps (`runner_id=0`, `steps=[]` nos runs consultados). Nenhum release, PipeWire/ALSA, mídia WebRTC ou Raspberry Pi 5 foi validado.
