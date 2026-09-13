# Phase 77 Review — validação estrutural antes do payload

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local aprovada; CI remoto e hardware pendentes.

## Objetivo

Rejeitar archives inválidos antes de consumir payloads, reduzindo custo de CPU/IO em entradas maliciosas.

## Implementado

- Nomes, raiz, tipos, allowlist, duplicatas e membros requeridos são validados antes da leitura dos payloads.
- Arquivos regulares aceitos continuam consumidos integralmente em chunks de até 1 MiB.
- Teste confirma que membro inesperado falha sem chamar o consumidor de payload.

## Gates locais

- `python3 -m pytest -q tests/test_validate_release_archive.py`: executar no gate final.
- `python3 -m py_compile scripts/validate-release-archive.py tests/test_validate_release_archive.py`: executar no gate final.
- `git diff --check`: executar no gate final.

## Segurança

A mudança reduz amplificação de CPU/IO causada por leitura de membros que seriam rejeitados por estrutura. Não altera allowlist, limites, proteção `O_NOFOLLOW` ou rejeição de links/arquivos especiais.

## Limitações

CI remoto segue bloqueado antes dos steps (`runner_id=0`, `steps=[]`). Nenhum release, PipeWire/ALSA, mídia WebRTC ou Raspberry Pi 5 foi validado.
