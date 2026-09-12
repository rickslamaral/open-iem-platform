# Phase 69 Review — validação do bundle final de release

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local concluída; CI remoto, release e hardware pendentes.

## Objetivo

Impedir publicação de bundle incompleto, adulterado ou misturado entre versões depois da coleta dos artefatos de build.

## Implementado

- `scripts/validate-release-bundle.py` exige archives server x86_64/ARM64 e web musician/engineer na versão informada.
- Cada archive exige sidecar `.sha256` com digest e nome exatos.
- Cada archive server exige assinatura detached não vazia.
- Entradas não regulares e arquivos inesperados são rejeitados.
- Workflow executa validação após download/flatten e antes de criar GitHub Release.
- SBOM continua opcional, quando gerado pelo job de build.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_bundle.py tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 39 testes.
- `python3 -m py_compile scripts/validate-release-bundle.py`: PASS.
- `git diff --check`: PASS.
- Parse YAML local do workflow: PASS.

## Limitações

CI remoto continua bloqueado antes dos steps por runner/permissões. Nenhuma release foi publicada. Assinatura do bundle é validada como presença e integridade estrutural; confiança criptográfica da chave pública continua responsabilidade do consumidor Raspberry Pi. Nenhum claim de suporte Windows, Raspberry Pi 5, PipeWire/ALSA ou mídia WebRTC.
