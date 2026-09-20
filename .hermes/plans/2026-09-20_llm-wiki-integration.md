# Plano — integrar skill de wiki interligada

## Objetivo
Absorver padrão de knowledge base interligada em Markdown, reutilizável no Open IEM e copiável para projetos novos.

## Escopo
- Criar `.agents/skills/llm-wiki/SKILL.md` genérica.
- Criar lint sem dependências externas para links, índice, frontmatter, tags, órfãos e hash de fontes.
- Registrar skill em `docs/SKILLS.md`.
- Integrar no workflow de engenharia do `START.md` sem obrigar wiki para tarefas triviais.

## Limites
- Não migrar `docs/research/` automaticamente.
- Não criar páginas de conhecimento sem fontes.
- Não alterar código de produto.
- Não commitar, fazer push ou abrir PR automaticamente.

## Gates
- `scripts/validate-skills.sh` PASS.
- lint da skill executa em wiki fixture real.
- links e frontmatter da skill válidos.
- `git diff --check` PASS.
