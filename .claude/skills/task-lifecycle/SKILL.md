---
description: Execute uma task pendente do Open IEM do backlog até merge com testes, revisão, CI e documentação.
argument-hint: <task ou ID da task>
---

# Task Lifecycle — Backlog até Merge

Use para iniciar qualquer task pendente e conduzi-la até merge. Não pule fases. Registre bloqueios com evidência.

## 1. Seleção e baseline

1. Ler `CLAUDE.md`, `.claude/CLAUDE.md`, `docs/TODO.md` e `docs/DEVELOPMENT-LOG.md`.
2. Identificar task exata, fase, dependências e critérios de aceite.
3. Confirmar `git status`, branch atual e HEAD.
4. Verificar que task não está duplicada, já implementada ou bloqueada.
5. Criar branch de trabalho: `feat/phase-<N>-<short-name>` ou `fix/<short-name>`.
6. Registrar baseline de testes antes de editar.

## 2. Especificação e plano

1. Descrever escopo, fora de escopo, riscos, arquivos prováveis e testes.
2. Criar ou atualizar ADR quando houver decisão arquitetural.
3. Quebrar task em subtarefas verificáveis.
4. Não implementar enquanto critérios de aceite estiverem ambíguos.

## 3. Implementação

1. Escrever ou atualizar testes antes da implementação quando houver comportamento novo.
2. Implementar menor mudança suficiente.
3. Preservar segurança, ownership, revisão de WebSocket, limites realtime e compatibilidade.
4. Manter `SIMULATED` explícito para áudio, Opus, WebRTC media e Raspberry Pi não validados.
5. Não alterar workflows, release, instalador ou hardware sem necessidade da task.

## 4. Verificação local

Executar comandos afetados e depois `.claude/scripts/run-all-gates.sh` quando possível:

- Rust: fmt, clippy, workspace tests
- Frontend: typecheck, tests, build em app afetada
- Python: pytest
- Documentação: `.claude/scripts/validate-phase-completion.py` e `.claude/scripts/validate-trilingual-guides.py`

Comparar com baseline. Só falhas novas bloqueiam. Não inventar resultado ausente.

## 5. Revisão independente

Executar agents separados:

- `test-reviewer` para cobertura, regressões e gates
- `security-reviewer` para segurança
- `ci-diagnostics` quando houver falha remota
- `documentation-sync` quando docs mudarem

Corrigir findings BLOCKER/HIGH/MEDIUM e repetir revisão. Máximo 2 ciclos automáticos; depois escalar.

## 6. Documentação e commit

Antes do commit:

- Atualizar `docs/TODO.md`
- Atualizar `docs/DEVELOPMENT-LOG.md`
- Atualizar `CHANGELOG.md` em `[Unreleased]`
- Atualizar `README.md` se status, limitações ou interface mudarem
- Criar/atualizar `docs/reviews/PHASE-<N>-REVIEW.md`
- Atualizar guias `docs/guides/pt`, `docs/guides/en`, `docs/guides/es` quando user-facing
- Executar `git diff --check`
- Confirmar ausência de segredos

Commitar com Conventional Commits. Não usar `[verified]` sem revisão independente real.

## 7. Pull request

1. Push da branch exige confirmação explícita do usuário.
2. Abrir PR com objetivo, escopo, testes reais, limitações e checklist.
3. Não usar `--force`.
4. Aguardar checks. `runner_id=0` e `steps=[]` significa CI não executado, não aprovação.

## 8. CI e correções

1. Inspecionar todos os jobs e logs.
2. Se falhar por código, corrigir na branch e repetir gates.
3. Se falhar por quota/permissão/runner, registrar bloqueio operacional sem mascarar como sucesso.
4. Revalidar PR após cada correção.

## 9. Merge e fechamento

Só fazer merge quando:

- revisão independente passou;
- testes locais passaram;
- CI executou jobs reais e passou;
- checks obrigatórios estão verdes;
- conflitos resolvidos;
- critérios de aceite atendidos;
- documentação sincronizada;
- usuário confirmou merge quando política do repositório exigir.

Depois do merge:

1. Confirmar SHA do commit mergeado e branch remota.
2. Atualizar `docs/TODO.md` e `DEVELOPMENT-LOG.md` se necessário.
3. Confirmar workspace limpo.
4. Reportar evidência separada para local, CI, release e hardware.

Nunca declarar merge concluído sem URL/ID/SHA verificável.
