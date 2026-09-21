# Open IEM Platform — Hermes Agent Interface

> Este arquivo é carregado automaticamente pelo Hermes Agent quando o `workdir`
> do cron job aponta para `/workspace/open-iem-platform/`. Ele complementa
> `CLAUDE.md` / `.claude/CLAUDE.md` e fornece instruções específicas para
> agentes autônomos rodando via Hermes (crons `361e70c8e264` e `7aee82067e22`).

---

## Missão do Agente

Você é o agente autônomo de engenharia do Open IEM Platform.
Repositório canônico: `https://github.com/rickslamaral/open-iem-platform`
Workspace: `/workspace/open-iem-platform/`

**Sempre comece com:**

```bash
cd /workspace/open-iem-platform
set -a; . /root/.hermes/.env; set +a
```

Leia nesta ordem antes de agir:

1. `START.md` — contrato de engenharia
2. `docs/TODO.md` — backlog executável
3. `docs/DEVELOPMENT-LOG.md` — histórico
4. `git status`, `git log --oneline -10` — estado do repositório
5. `gh pr list --base main --state open` — PRs abertas

---

## Regras de operação autônoma

### O que o agente PODE fazer autonomamente

- Corrigir documentação e status obsoletos (PRs já mergeadas, CI antigo, branches inexistentes)
- Implementar tarefas bem delimitadas do backlog (itens `[ ]` do TODO)
- Rodar gates locais: `cargo fmt`, `cargo clippy`, `cargo test`, typecheck e build frontend
- Abrir PRs — **nunca fazer push direto em `main`**
- Fazer squash merge de PRs quando CI local passou e revisão independente aprovada

### O que EXIGE confirmação explícita de Ricardo

- Publicar release (`v0.3.1` ou qualquer tag)
- Alterar configuração de CI/CD de forma estrutural
- Expor serviço externo ou modificar `deployment/` em produção
- Operações que afetem dados de usuário real

### Credenciais

```bash
# Carregar somente na memória do processo — NUNCA gravar em arquivo
set -a; . /root/.hermes/.env; set +a
```

Nunca imprimir, logar, commitar ou expor token, chave ou segredo.
Se `.env` ausente ou `gh auth` falhar, reportar bloqueio sem pedir gravação de credenciais.

---

## Gates obrigatórios antes de qualquer commit

```bash
# Rust
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --manifest-path server/Cargo.toml

# Nota: cargo clippy --all-features falha neste host (jack.pc ausente)
# Use --all-targets sem --all-features

# Frontend musician
cd web/musician && npm run typecheck && npm test -- --watchAll=false && npm run build && cd ../..

# Frontend engineer
cd web/engineer && npm run typecheck && npm test -- --watchAll=false && npm run build && cd ../..

# Segurança (quando tocar scripts de release)
python3 tests/validate_archive.py
```

---

## Workflow por fase

```text
SPEC → PLAN → IMPLEMENT → TEST → INDEPENDENT REVIEW → SECURITY REVIEW
  → DOCUMENT → COMMIT (branch) → PR → CI check → MERGE
```

- Revisão independente: subagente separado, contexto isolado
- Commit obrigatório: `[verified]` no prefixo quando todos os gates passaram
- Branch: `feat/phaseNN-*`, `fix/*`, `chore/*`, `docs/*` — nunca `main`

---

## Estado atual (atualizar a cada merge)

```text
Phase 116 — Observability Metrics reset endpoint e Engineer Console: implementado e mergeado em main
GAP-018/019 — PairingRegistry integrado em api-server: PR #189 mergeado, CI 16/16 SUCCESS; DTLS-SRTP e runtime pendentes
P2 — Preset catalog/application: implementado em código/CI; runtime permanece pendente
Release v0.3.1: pendente confirmação + instalação/validação física
CI remoto: exigir jobs reais no exact HEAD; runner_id=0/steps=[] não conta
Validação física: PipeWire/ALSA, WebRTC/Opus, Raspberry Pi 5 — SIMULATED
```

---

## Restrições permanentes

| Restrição | Motivo |
|-----------|--------|
| Nunca declarar validado sem evidência real | RPi5, PipeWire, ALSA e WebRTC SIMULATED até hardware |
| Nunca publicar release sem confirmação | Requer instalação real + validação Ricardo |
| Nunca push direto em main | Toda mudança via branch + PR |
| Nunca gravar credenciais em arquivo | Token só em memória de processo |
| CI runner_id=0 / steps=[] ≠ CI verde | Exigir jobs executados de verdade |

---

## Links úteis

- `START.md` — contrato completo de engenharia (fonte canônica)
- `.claude/CLAUDE.md` — instruções para Claude Code
- `docs/TODO.md` — backlog executável com prioridades
- `docs/DEVELOPMENT-LOG.md` — log de decisões e implementações
- `docs/ARCHITECTURE-GAPS.md` — desvios arquiteturais detectados
- `CHANGELOG.md` — histórico de mudanças (Keep a Changelog)

## Robustez de execução Hermes

- Não usar `read_file`, `search_files` ou outros helpers Hermes dentro de `terminal`.
- Não assumir `rg` instalado; preferir comandos POSIX disponíveis ou helpers Hermes.
- Evitar Python complexo inline em `python3 -c`; usar script temporário/arquivo.
- Rust sempre aponta para `server/Cargo.toml` quando executado na raiz.
- Máximo de 1 tarefa implementada por ciclo e 40 chamadas de modelo; ao atingir limite, salvar estado e encerrar limpo.
- Se contexto ultrapassar 35% do limite, compactar antes de nova sequência de ferramentas.
- Não repetir envio Telegram após `flood_control`; registrar bloqueio e usar saída local.
- Não iniciar ciclo se já houver execução ativa no workspace; aguardar próximo tick.

## Desenvolvimento paralelo de branches

Regra: nunca ficar ocioso aguardando CI. Fluxo por ciclo:

1. Ler AGENTS.md, START.md e docs/DEVELOPMENT-HANDOFF.md.
2. Verificar PRs abertas com `gh pr list --base main --state open`.
3. **Se há PR com CI em andamento:**
   - Verificar status CI: `gh run list --branch <branch> --limit 1`.
   - CI ainda rodando → iniciar próxima tarefa em branch nova independente (nunca baseada na branch pendente).
   - CI verde → fazer merge squash + delete branch antes de iniciar nova tarefa.
   - CI falhou → diagnosticar e corrigir branch com falha; não iniciar nova tarefa.
4. **Se não há PR aberta** → selecionar próxima tarefa e implementar.
5. Limite: máximo 1 PR aguardando CI + 1 branch em desenvolvimento local ao mesmo tempo.
6. Nunca basear branch nova em branch não mergeada em main.
7. Gates locais completos antes de qualquer push (Rust fmt/clippy/testes, frontends, security scan).
8. Relatório obrigatório: progresso X/Y (Z%), PRs abertas com estado CI, ação executada, próxima tarefa planejada.


## Ritmo de ciclo — máximo throughput local

Objetivo: completar 2–3 tarefas reais por ciclo de 30 min aproveitando CI em background.

Padrão esperado por ciclo:
- Implementar tarefa A → gates locais → push → PR aberta.
- Enquanto CI roda em PR A: implementar tarefa B localmente (branch independente de main).
- CI PR A verde → merge squash + delete + sync main.
- Push tarefa B → PR → checar CI.
- Se tempo restante: iniciar tarefa C.

Regra de limite: máximo 2 PRs abertas simultaneamente. Com 2 PRs abertas, aguardar merge antes de abrir terceira.

Nunca: basear branch em branch não mergeada; fazer push sem gates completos; aceitar CI com runner_id=0 ou steps=[].

## Limpeza obrigatória de branches e PRs

Antes de iniciar qualquer nova tarefa, sempre executar:

1. `gh pr list --state open` — se houver PR com CI verde, fazer merge squash + delete imediatamente.
2. `git branch -r | grep -v HEAD` — identificar branches remotas sem PR aberta (órfãs); deletar com `git push origin --delete <branch>`.
3. `git branch --merged main | grep -v main` — branches locais já mergeadas; deletar com `git branch -d`.
4. Nunca acumular mais de 2 PRs abertas. Se houver 2 e CI de uma ficou verde, merge antes de abrir terceira.
5. Ao final de cada ciclo: confirmar que branches da sessão foram removidas local e remotamente.

Checklist de limpeza (executar no início e fim de cada ciclo):
```bash
git fetch --prune
gh pr list --state open --json number,headRefName,statusCheckRollup
git branch --merged main | grep -v '^\*\|main'
```

## Política de merge e correção — obrigatória

### PRs com CI verde
- Merge squash imediato. Não aguardar instrução externa.
- Delete branch local e remota após merge.
- Sync main: `git checkout main && git pull origin main`.

### PRs com CI vermelho
- Não abandonar. No mesmo ciclo ou no próximo: criar commit de correção na mesma branch e aguardar novo CI.
- Se CI falhou por flakiness (timeout/runner), re-trigger: `gh run rerun <run_id> --failed`.
- Se falha é de código: corrigir, gates locais, push; CI re-executa automaticamente.
- Nunca fechar PR com falha sem corrigir. Nunca abrir PR nova sobre problema não resolvido.

### Ordem de prioridade ao iniciar ciclo
1. Merge PRs com CI verde (sempre primeiro).
2. Corrigir PRs com CI vermelho.
3. Implementar nova tarefa.

## Proibição de encerramento prematuro

Concluir uma task NÃO encerra janela horária. Após cada commit validado:

1. Reconsultar backlog e DEVELOPMENT-HANDOFF.
2. Verificar se PR pendente tem CI em andamento; não esperar passivamente.
3. Continuar próxima task independente na mesma branch horária.
4. Repetir até entrar nos 5 minutos finais.

Relatório com `1 task` exige informar motivo objetivo do encerramento. Não parar apenas porque uma task foi commitada, revisada ou enviada para PR. PR pendente não é bloqueio para desenvolvimento local.

## Checkpoints sem interrupção

A cada aproximadamente 15 minutos, criar checkpoint local somente se houver unidade segura de trabalho concluída. Checkpoint não pausa desenvolvimento: não executar gates completos, não fazer push, não abrir PR e não aguardar CI nesse ponto. Voltar imediatamente à próxima task.

Somente no intervalo final de 5 minutos:
- parar novas tasks;
- executar gates completos;
- commitar qualquer trabalho seguro restante;
- fazer um único push da branch horária;
- abrir ou atualizar uma única PR.

O objetivo é maximizar tasks concluídas por janela, não maximizar quantidade de checkpoints. Nunca interromper implementação no meio apenas para criar checkpoint; salvar depois do milestone seguro mais próximo.
