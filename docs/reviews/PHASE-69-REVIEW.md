# Phase 69 Review — validação do bundle final de release

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local concluída; runs `34711083274` (PR) e `34711080316` (push) falharam antes dos steps; CI remoto, release e hardware pendentes.

## Objetivo

Impedir publicação de bundle incompleto, adulterado ou misturado entre versões depois da coleta dos artefatos de build.

## Implementado

- `scripts/validate-release-bundle.py` exige archives server x86_64/ARM64 e web musician/engineer na versão informada.
- Cada archive exige sidecar `.sha256` com digest e nome exatos.
- Cada archive server exige assinatura detached não vazia; jobs de build e validação final verificam assinatura com chave pública fornecida por variável de repositório.
- `OPENIEM_RELEASE_SIGNING_PUBLIC_KEY_FINGERPRINT` é obrigatório, deve ser exatamente 64 hex e precisa coincidir com SHA-256 do DER da chave pública antes da verificação.
- Entradas não regulares e arquivos inesperados são rejeitados.
- Validação gera manifesto com nomes e digests; publicação usa somente lista explícita derivada do manifesto, sem glob.
- Workflow executa validação após download/flatten e antes de criar GitHub Release.
- Job Python de segurança do CI executa também `tests/test_validate_release_bundle.py`.
- SBOM continua opcional, quando gerado pelo job de build.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_bundle.py tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 42 testes.
- `python3 -m py_compile scripts/validate-release-bundle.py`: PASS.
- `git diff --check`: PASS.
- Parse YAML local do workflow: PASS.

## Limitações

CI remoto continua bloqueado antes dos steps por runner/permissões. Nenhuma release foi publicada. O workflow exige fingerprint SHA-256 provisionado independentemente, falha fechado se fingerprint ausente, malformado ou divergente, e publica somente arquivos listados no manifesto validado. Nenhum claim de suporte Windows, Raspberry Pi 5, PipeWire/ALSA ou mídia WebRTC.
