# Phase 81 Review — hardening do instalador e concorrência CI/release

**Data:** 2026-09-13
**Status:** PASS WITH CONDITIONS — gates locais e CI do PR aprovados; instalação e hardware pendentes.

## Objetivo

Reduzir risco operacional do instalador Linux e impedir perda de evidência pós-merge ou disputa entre publicações de uma mesma tag.

## Implementado

- `scripts/install.sh --dry-run` descreve dependências, clone, builds, instalação e systemd sem alterar host.
- `--ref` aceita SHA-1 completo usando clone sem checkout, fetch limitado e checkout detached.
- Caminhos absolutos rejeitam newline, `&`, `\\` e `#` antes de substituições `sed` no unit file.
- CI executa em pull requests e pushes para `main`, com cancelamento apenas dentro do mesmo grupo.
- Release serializa execuções por referência de tag, sem cancelar execução em andamento.

## Verificação real

- `bash -n scripts/install.sh`: aprovado.
- `scripts/install.sh --dry-run --skip-deps --ref b3c0fb2`: aprovado; host não alterado.
- YAML de `.github/workflows/ci.yml` e `.github/workflows/release.yml`: parseado com sucesso.
- `git diff --check`: aprovado.
- CI PR #41: runs `34735649836` e `34735649634` concluídos com sucesso.
- Test Agent: sem BLOCKER/HIGH/MEDIUM; suíte local reportada: Rust 247, Python 60, Musician 34, Engineer 2.
- Security Review: sem BLOCKER; ressalva residual: instalador compila checkout remoto e ainda não verifica assinatura/checksum do código-fonte.

## Limitações

- ShellCheck não está instalado no host.
- Instalação real, release, runtime Windows/ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC não foram validados.
- `--ref` suporta SHA-1 completo, não SHA abreviado.

## Decisão

Manter PR #41 aberto até verificação final dos checks. Não publicar release nem declarar suporte de hardware.