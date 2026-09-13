# Phase 51 Review — hardening do fluxo de deployment ARM64

**Data:** 2026-09-11
**Status:** PASS WITH CONDITIONS — local; PR #40 aberto; não mergeado.

## Escopo

Hardening do guia `deployment/raspberry-pi/README.md` para reduzir riscos em download, diretórios de serviço, geração de chaves JWT e instalação da unit systemd. Nenhuma execução em Raspberry Pi 5 foi alegada.

## Implementado

- `curl` restringe redirects para HTTPS com `--proto '=https' --proto-redir '=https'`.
- `/etc/openiem` e `/etc/openiem/keys` usam `root:openiem`/`0750`, permitindo travessia/leitura pelo serviço e bloqueando demais usuários; diretórios de dados usam `openiem:openiem` e `0750`.
- Chaves JWT são criadas em diretório temporário com limpeza por `trap` e instaladas via `sudo install` com modos `0600`/`0644`.
- Unit systemd é resolvida por `git rev-parse --show-toplevel`, exige arquivo regular, passa por `systemd-analyze verify` e é instalada com ownership root e modo `0644`.

## Verificação

- `git diff --check`: PASS.
- Extração dos blocos Bash e `bash -n`: PASS.
- `make test`: PASS — Rust, frontend e harness determinístico.
- `bash scripts/validate-docs.sh`: PASS.
- `bash scripts/validate-pdf.sh`: PASS.
- `bash scripts/validate-skills.sh`: PASS.

## Condições e limites

- CI remoto continua falhando antes dos steps (`runner_id=0`, `steps=[]`); runs observados `34634194257` e `34634188913`.
- Não há release publicado, fixture offline do instalador, runtime ARM64, PipeWire/ALSA, mídia WebRTC ou Raspberry Pi 5 validado.
- `git rev-parse` exige clone Git confiável; isso é requisito explícito do procedimento.

## Decisão

Aceitar hardening local com condições. Bloquear merge/release até CI remoto executar gates reais e hardware validar instalação/runtime.
