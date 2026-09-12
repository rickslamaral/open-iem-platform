# Phase 70 Review — hardening do validador de bundle

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local concluída; CI remoto, release e hardware pendentes.

## Implementado

- Limites fail-closed: 512 MiB por arquivo, 2 GiB por bundle e 64 KiB por manifesto.
- Leituras usam `O_NOFOLLOW` e `O_NONBLOCK`; SBOM opcional precisa ser JSON objeto válido.
- Manifesto é criado com `O_NOFOLLOW`, modo restrito e validação de arquivo regular.
- Testes offline cobrem limite, SBOM inválido e manifesto symlink.

## Verificação

- `pytest==9.1.1`: 45 testes dos validadores aprovados.
- `make validate`: aprovado.
- `py_compile` e `git diff --check`: aprovados.

## Limitações

CI remoto continua bloqueado antes dos steps por runner/permissões. TOCTOU entre validação e cópia posterior no workflow permanece follow-up. Raspberry Pi 5, PipeWire/ALSA, WebRTC, Windows runtime e release continuam não validados.

---

# Phase 69 Review — validação do bundle final de release

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local concluída; runs `34711659175` (PR) e `34711656770` (push) falharam antes da execução dos jobs; no PR, os 10 jobs retornaram `runner_id=0` e `steps=[]`; CI remoto, release e hardware pendentes.

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
