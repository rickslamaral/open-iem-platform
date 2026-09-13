# Phase 67 Review — nomes seguros entre plataformas

**Data:** 2026-09-12 12:33 BRT
**Status:** PASS WITH CONDITIONS — validação local concluída; CI remoto, release e hardware pendentes.

## Objetivo

Rejeitar separadores Windows e caracteres de controle em nomes de archive. Consumidores POSIX e Windows não podem interpretar o mesmo membro de formas diferentes.

## Implementado

- `scripts/validate-release-archive.py` rejeita `\\` e caracteres U+0000–U+001F antes da validação estrutural.
- Mensagem para nomes de controle usa `repr`, evitando saída CLI ambígua ou quebra de linha.
- Testes offline cobrem backslash e newline.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 26 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- `make validate`: PASS antes desta alteração; repetir no gate final.
- Review independente: PASS; nenhum concern de segurança ou erro lógico de alta confiança.

## Limitações

CI remoto falha antes dos steps por runner/permissões. Nenhum claim de suporte Windows, Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC ou release publicada.
