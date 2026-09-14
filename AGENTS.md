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
Phase 92 — WS EQ Band Control: implementado, mergeado em main (#54)
                                Frontend EQ controls: pendente (Phase 93+)
Release v0.3.1: pendente confirmação + validação física
CI remoto: aguarda runner GitHub Actions (jobs reais, não runner_id=0/steps=[])
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
