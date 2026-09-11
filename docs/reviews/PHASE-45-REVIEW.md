# Phase 45 Review — correção do label de runner

**Data:** 2026-09-11
**Estado:** implementação local commitada; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

## Objetivo

Corrigir seleção de runner que fazia todos os jobs CI falharem antes dos steps com `runner_id=0`.

## Implementado

- Trocado `runs-on: ubuntu-24.04` por `runs-on: ubuntu-latest` em todos os jobs de `.github/workflows/ci.yml`.
- Atualizados README, CHANGELOG, START, TODO e Development Log.

## Verificação

- Nenhum job usa `ubuntu-24.04` ou label `self-hosted`.
- `git diff --check` e validações locais passam antes deste commit.
- Push executado; runs `34607886129` e `34610969113` falharam em todos os nove jobs com `runner_id=0` e `steps=[]`.

## Limitações

API de permissões Actions e runners retorna HTTP 403 para token atual. Falha ocorre antes de checkout, portanto código, dependências e gates não foram executados no GitHub. Não há evidência de CI verde, release ou suporte de hardware.

## Decisão

**BLOCKED:** configuração de workflow corrigida, mas merge permanece bloqueado até administrador desbloquear runner/permissões e todos os jobs executarem e passarem.
