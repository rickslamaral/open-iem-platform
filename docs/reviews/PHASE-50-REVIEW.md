# Phase 50 Review — ARM64 installer hardening

**Data:** 2026-09-11
**Status:** PASS WITH CONDITIONS — local; PR #40 aberto; não mergeado.

## Escopo

Hardening do bloco de instalação ARM64 em `deployment/raspberry-pi/README.md`, sem alegar execução em Raspberry Pi 5.

## Implementado

- `set -euo pipefail` e `curl --fail` interrompem falhas de download.
- Tag `vX.Y.Z` validada antes de gerar URL e caminhos.
- Arquivos baixados em diretório temporário exclusivo, removido via `trap`.
- SHA-256 conferido contra arquivo e archive baixados.
- Membros com caminho absoluto, traversal, symlink e hard link rejeitados antes da extração.
- `api-server` deve ser arquivo regular no diretório versionado esperado.

## Verificação

- `make test`: PASS.
- `bash scripts/validate-docs.sh`: PASS.
- `bash scripts/validate-pdf.sh`: PASS.
- `bash scripts/validate-skills.sh`: PASS.
- `bash -n` do bloco Bash: PASS.
- `git diff --check`: PASS.
- Review independente: riscos de falha não tratada, arquivo stale e archive inseguro encontrados no fluxo anterior; correções aplicadas.

## Condições e limites

- Checksum hospedado no mesmo release HTTPS prova integridade de transferência, não autenticidade independente.
- Não existe teste automatizado de instalação contra artefato fixture.
- CI remoto continua falhando antes dos steps (`runner_id=0`, `steps=[]`); não há release publicado.
- Runtime ARM64, PipeWire/ALSA, WebRTC media e Raspberry Pi 5 continuam não validados.

## Decisão

Aceitar hardening documental local. Bloquear merge/release até CI remoto executar gates reais e hardware validar instalação/runtime.
