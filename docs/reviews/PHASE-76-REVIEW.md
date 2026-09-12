# Phase 76 Review — leitura integral de payloads de archive

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local aprovada; CI remoto e hardware pendentes.

## Objetivo

Impedir que o validador de archives aceite metadados de membros sem consumir e conferir payload completo.

## Implementado

- `scripts/validate-release-archive.py` consome arquivos regulares em chunks limitados após validar headers.
- Payload ausente ou truncado falha fechado; `tarfile` limita leitura ao tamanho declarado pelo header.
- Teste offline cobre archive gzip com payload de membro truncado.

## Gates locais

- `python3 -m pytest -q tests/test_validate_release_archive.py`: executar no gate final.
- `python3 -m py_compile scripts/validate-release-archive.py tests/test_validate_release_archive.py`: executar no gate final.
- `git diff --check`: executar no gate final.

## Segurança

A validação continua rejeitando symlinks, arquivos especiais, traversal, nomes não canônicos e limites de recursos. Nenhum secret foi adicionado.

## Limitações

CI remoto segue falhando antes dos steps (`runner_id=0`, `steps=[]`). Os runs mais recentes são `34724845040` (PR) e `34724842446` (push); nenhum job executou steps. Archives, release, mídia WebRTC, PipeWire/ALSA e Raspberry Pi 5 permanecem não validados em ambiente real.

## Impacto de release

Nenhum artefato foi publicado. O gate local ficou mais estrito; archives truncados não avançam para checksum/upload.
