# Phase 68 Review — nomes Windows e limites de assinatura

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local concluída; CI remoto, release e hardware pendentes.

## Objetivo

Fechar interpretações perigosas de nomes em consumidores Windows e limitar entradas pequenas do verificador detached antes de executar OpenSSL.

## Implementado

- `scripts/validate-release-archive.py` rejeita caracteres inválidos (`<>:\"|?*`), pontos/espaços finais e nomes reservados DOS case-insensitive (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`) em cada componente.
- `scripts/verify-release-signature.py` rejeita assinatura e chave pública acima de 64 KiB usando `fstat()` no descritor protegido.
- Testes cobrem roots inválidas, componente aninhado reservado, assinatura oversized e chave pública oversized.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 30 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Review independente: encontrou risco DoS por entradas de assinatura/chave sem limite; correção aplicada.

## Limitações

CI remoto continua falhando antes dos steps por runner/permissões. Nenhum claim de suporte Windows, Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC ou release publicada.
