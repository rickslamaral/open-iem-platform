# Phase 49 Review — correção do guia de instalação ARM64

**Data:** 2026-09-11
**Estado:** documentação corrigida localmente; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Garantir que guia Raspberry Pi use nome e layout reais dos artefatos gerados pelo workflow de release.

## Implementado

- Corrigida URL do artefato para `open-iem-server-<versão>-aarch64-linux.tar.gz`.
- O checksum publicado é baixado e verificado com `sha256sum --check` antes de `tar` e `install`.
- `curl` falha em erros HTTP.
- Corrigido caminho do binário após extração: `open-iem-server-<versão>-aarch64-linux/api-server`.
- Versão de exemplo atualizada para `v0.3.1`, ainda não publicada.

## Verificação

- Comparação manual do guia com `.github/workflows/release.yml` confirmou nomes idênticos.
- `bash scripts/validate-docs.sh`: PASS.
- `bash scripts/validate-pdf.sh`: PASS.
- `bash scripts/validate-skills.sh`: PASS.
- `git diff --check`: PASS.

## Limitações

Não existe release `v0.3.1` publicada para teste de download. CI remoto continua falhando antes dos steps com `runner_id=0`. Runtime ARM64, PipeWire/ALSA e Raspberry Pi 5 continuam não validados.

## Decisão

**PASS WITH CONDITIONS:** documentação alinhada ao empacotamento implementado. Não publicar release nem declarar suporte ARM64 antes de CI e hardware reais passarem.
