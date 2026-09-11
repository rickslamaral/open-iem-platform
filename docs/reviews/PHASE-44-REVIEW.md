# Phase 45 Review — correção do label de runner

**Data:** 2026-09-11
**Estado:** implementação local; PR #40 aberto; CI remoto ainda não reexecutado; não mergeado; não lançado.

## Objetivo

Corrigir seleção de runner que fazia todos os jobs CI falharem antes dos steps com `runner_id=0`.

## Implementado

- Trocado `runs-on: ubuntu-24.04` por `runs-on: ubuntu-latest` em todos os jobs de `.github/workflows/ci.yml`.
- Atualizados README, CHANGELOG, START e TODO.

## Verificação

- YAML revisado: nenhum job usa `ubuntu-24.04` ou label `self-hosted`.
- `git diff --check`: será confirmado antes do commit.
- CI remoto precisa executar novamente para validar correção.

## Limitações

Runs anteriores `34605340696` e `34605336044` falharam antes dos steps; API de runners/permissões Actions retorna HTTP 403 para token atual. Não há evidência de CI verde nesta rodada.

## Decisão

**PASS WITH CONDITIONS:** correção de configuração pronta; merge permanece bloqueado até jobs remotos executarem e passarem.

---

# Phase 44 Review — correção do gate de whitespace documental

**Data:** 2026-09-11
**Estado:** correção local; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

## Objetivo

Corrigir whitespace introduzido em documentação que fazia o job `documentation` falhar no diff completo da branch.

## Implementado

- Removido trailing whitespace de três reviews de fase e da matriz de validação de plataforma.
- Atualizados README, CHANGELOG, TODO, START e DEVELOPMENT-LOG para registrar estado real.

## Verificação

- `git diff --check origin/main...HEAD`: PASS.
- Testes Rust, clippy, typecheck/test/build dos dois frontends, validação de docs, PDF e skills: PASS localmente.
- Review independente confirmou falha lógica do gate e não encontrou preocupação de segurança no diff.

## Limitações

GitHub Actions continua falhando antes dos steps por ausência de runner executável (`34603073532`). PipeWire/ALSA, WebRTC media, Raspberry Pi 5 e release permanecem não validados.

## Decisão

**PASS WITH CONDITIONS:** correção documental pronta para commit; merge permanece bloqueado até CI remoto executar todos os jobs e passar.
