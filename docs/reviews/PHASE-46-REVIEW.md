# Phase 46 Review — alinhamento do runner de release

**Data:** 2026-09-11
**Estado:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Alinhar workflow de release ao runner `ubuntu-latest` já usado no CI, removendo inconsistência entre os dois workflows.

## Implementado

- Alterados seis jobs em `.github/workflows/release.yml`: `validate-version`, `quality-gate`, `build-server-x86`, `build-server-arm64`, `build-web` e `github-release`.
- Nenhuma permissão, dependência, regra de versão, etapa de qualidade ou empacotamento foi alterada.
- README, CHANGELOG, TODO e Development Log alinhados ao estado real.

## Verificação

- `release.yml` contém seis ocorrências de `runs-on: ubuntu-latest`.
- `release.yml` não contém `ubuntu-24.04`.
- Gatilho de release permanece restrito a tags semver `vX.Y.Z`.
- Gates locais Rust, documentação, PDF e skills passam; `cargo audit` continua encontrando somente `RUSTSEC-2023-0071`, tratado pela política existente.

## Limitações

`ubuntu-latest` é alias móvel e não prova runner disponível. Runs remotos `34614028392` e `34614025997` falharam antes dos steps com `runner_id=0` e `steps=[]`. Token atual não permite consultar configuração de runners (HTTP 403). Não há evidência de CI verde, release publicada, hardware PipeWire/ALSA, mídia WebRTC ou Raspberry Pi 5.

## Decisão

**BLOCKED:** mudança de configuração consistente e validada localmente. Merge e release aguardam execução remota real com todos os gates verdes.
