# Phase 59 Review — limites de recursos no validador de archives

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local; CI remoto, release e hardware pendentes.

## Objetivo

Limitar consumo de recursos durante validação de archives antes de checksum, assinatura e extração no instalador Raspberry Pi.

## Implementado

- `scripts/validate-release-archive.py` rejeita archives acima de 512 MiB comprimidos.
- Rejeita archives com mais de 32 membros.
- Rejeita membro regular acima de 256 MiB e total declarado acima de 512 MiB.
- Testes determinísticos cobrem limite de membros e tamanho de membro.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 15 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Reviews independentes: sem blocker; recomendação de limites de recursos implementada.

## Limitações

Os limites reduzem decompression bomb e consumo previsível, mas não substituem autenticação Ed25519. CI remoto continua falhando antes dos steps nos runs `34669494781` e `34669493264`. Nenhum release, Caddy, Raspberry Pi 5, PipeWire/ALSA ou mídia WebRTC foi validado.

**Decisão:** PASS WITH CONDITIONS. Não fazer merge/release até CI executável, secret/fingerprint provisionados e validação de hardware concluída.
