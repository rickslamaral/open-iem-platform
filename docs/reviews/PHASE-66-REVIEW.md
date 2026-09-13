# Phase 66 Review — raiz canônica em archives

**Data:** 2026-09-12 12:04 BRT
**Status:** PASS WITH CONDITIONS — validação local concluída; CI remoto, release e hardware pendentes.

## Objetivo

Rejeitar diretórios raiz não canônicos em archives de release, evitando nomes ambíguos (`.` e `..`) e mantendo paths determinísticos antes da extração.

## Implementado

- `scripts/validate-release-archive.py` rejeita raiz `.`/`..` e qualquer raiz cuja forma normalizada difira do nome original.
- Teste offline cobre raiz `.`.
- Nenhuma permissão de extração foi ampliada.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: executar no gate final.
- Review independente: PASS; sem concerns de segurança ou erros lógicos de alta confiança.
- CI remoto: bloqueado antes dos steps por runner/permissões.

## Limitações

Nenhum claim de suporte Windows, Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC ou release publicada.
