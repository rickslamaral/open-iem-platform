# Phase 65 Review — limites incrementais durante leitura de archives

**Data:** 2026-09-12 11:35 BRT
**Status:** PASS WITH CONDITIONS — validação local concluída; CI remoto, release e hardware pendentes.

## Objetivo

Aplicar limites de tamanho de membros e tamanho descomprimido total enquanto o `tarfile` itera o archive, rejeitando entrada abusiva sem consumir headers posteriores.

## Implementado

- `scripts/validate-release-archive.py` valida `member.size` no primeiro loop.
- O acumulador de bytes descomprimidos falha assim que excede `MAX_UNCOMPRESSED_BYTES`.
- Teste garante que membro oversized no início não permite consumir membro posterior.
- Removida segunda passagem redundante para limites de tamanho.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 23 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- `make validate`: PASS antes desta alteração; repetir no gate final.
- Review independente: código atual sem concerns de segurança ou erros lógicos de alta confiança; recomendação de validação incremental aplicada.

## Limitações

CI GitHub falha antes dos steps por runner/permissões. Nenhum claim de suporte Windows, Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC ou release publicada.
