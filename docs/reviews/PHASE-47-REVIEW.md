# Phase 47 Review — hardening dos workflows

**Data:** 2026-09-11
**Estado:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Corrigir filtro de tag de release e reduzir permissões GitHub Actions ao mínimo necessário.

## Implementado

- Filtro de tag usa glob candidato compatível com Actions.
- Job `validate-version` rejeita qualquer tag fora de `vX.Y.Z`, incluindo componentes com zero à esquerda.
- CI e jobs de leitura do release usam `contents: read`.
- Job `github-release` recebe `contents: write` exclusivamente.

## Verificação

- Validação local de YAML/estrutura e `git diff --check` executadas após alteração.
- `make test`, validação de documentação, PDF e skills passam nesta rodada.
- Auditoria independente identificou o filtro anterior como incorreto; correção aplicada.

## Limitações

Actions continuam referenciando tags mutáveis de terceiros; pinning por SHA permanece follow-up. CI remoto segue falhando antes dos steps com `runner_id=0` e `steps=[]`; GitHub API do token atual retorna 403 para permissões/runners. Nenhuma release ou hardware foi validado.

## Decisão

**BLOCKED:** hardening local concluído. Merge aguarda CI remoto verde e revisão dos pins de Actions.
