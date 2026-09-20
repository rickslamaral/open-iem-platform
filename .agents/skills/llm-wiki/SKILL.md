---
name: llm-wiki
description: Use when a project needs persistent research memory, source ingestion, linked Markdown knowledge, or wiki health checks. Build and maintain cited, interlinked notes without duplicating or fabricating knowledge.
version: 1.0.0
author: Open IEM Platform
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [wiki, knowledge-base, research, markdown, provenance, obsidian]
    category: research
    related_skills: [code-documenter, grounded-citations]
---

# LLM Wiki

Persistent, compounding knowledge base in plain Markdown. Sources stay immutable; derived pages stay linked, cited, and auditable.

## When to Use

- Research decision, architecture investigation, product comparison, or recurring domain knowledge.
- User asks to create, ingest, query, update, lint, audit, or health-check a wiki.
- Project needs knowledge that should survive sessions and avoid repeated rediscovery.

Do not use for one-off lookups, trivial notes, or undocumented claims.

## Default Layout

Use `WIKI_PATH`; fallback is `./wiki` for project-local knowledge or `$HOME/wiki` for personal knowledge. Choose project-local when knowledge belongs to repository decisions. Never silently create a second wiki.

```text
wiki/
├── SCHEMA.md
├── index.md
├── log.md
├── raw/                 # immutable fetched/pasted sources
│   ├── articles/
│   ├── papers/
│   ├── transcripts/
│   └── assets/
├── entities/
├── concepts/
├── comparisons/
├── queries/
└── _archive/
```

## Mandatory Orientation

Before every operation on an existing wiki, read:

1. `SCHEMA.md`
2. `index.md`
3. Last 20–30 lines of `log.md`
4. Search existing pages for source entities and concepts

This prevents duplicate pages, broken conventions, and silent contradictions.

## Schema Rules

- Lowercase kebab-case filenames.
- Every derived page has YAML frontmatter: `title`, `created`, `updated`, `type`, `tags`, `sources`.
- Every raw source has `source_url` when applicable, `ingested`, and SHA-256 over body only.
- Tags must exist in `SCHEMA.md` taxonomy before use.
- New or updated pages link to at least two relevant pages with `[[wikilinks]]`, unless no legitimate relationship exists; record that exception.
- New pages enter `index.md`.
- Every mutation appends one entry to `log.md`.
- Split pages above roughly 200 lines; archive superseded pages instead of deleting history.

## Provenance

Register sources when retrieved. Store raw content before synthesis. Cite derived claims with source paths or inline citation IDs. Use `confidence: high|medium|low` for uncertain or single-source claims. Use `contested: true` and `contradictions: [...]` when sources disagree.

Never:

- Treat search snippets as full-source evidence.
- Re-type or invent URLs.
- Present model knowledge as sourced fact.
- Modify `raw/` after ingestion.
- Resolve contradictions by silently overwriting older claims.

## Operations

### Initialize

1. Resolve `WIKI_PATH`.
2. Create layout.
3. Write domain-specific `SCHEMA.md`, taxonomy, thresholds, and frontmatter contract.
4. Write `index.md` and append-only `log.md`.
5. Validate with `scripts/validate-llm-wiki.py --wiki "$WIKI"`.

### Ingest

1. Fetch URL, PDF, or supplied text.
2. Save raw source with frontmatter and body hash.
3. Orient and search existing pages.
4. Update existing pages before creating duplicates.
5. Add only central entities/concepts or items supported by at least two sources.
6. Add links, sources, confidence, and contradiction markers.
7. Update index and append one log entry.
8. Run lint and report changed files.

### Query

1. Read index and search pages by terms.
2. Read relevant pages and their linked sources.
3. Answer from wiki content; distinguish `UNKNOWN` and `[unverified]` claims.
4. File substantial synthesis under `queries/` or `comparisons/`.
5. Log query and whether it was filed.

### Lint

Run:

```bash
python3 scripts/validate-llm-wiki.py --wiki "${WIKI_PATH:-./wiki}"
```

Fix, or explicitly report, broken links, missing index entries, malformed frontmatter, invalid tags, orphan pages, stale source hashes, oversized pages, and unlogged mutations.

### Archive

Move superseded pages into `_archive/` preserving relative path, remove index entry, update inbound links, and log the action. Do not delete source history.

## Project Integration

For Open IEM, use the wiki for external research, architecture decisions, compatibility evidence, and reusable operational knowledge. Keep implementation truth in source code and canonical `docs/`; link wiki findings into ADRs or research docs. Do not replace `START.md`, `TODO.md`, handoff, changelog, or release gates with wiki pages.

For a new project, copy this skill and the validator, set `WIKI_PATH`, then customize `SCHEMA.md` taxonomy and page thresholds. Keep the workflow project-agnostic.

## Common Pitfalls

- Creating notes before orientation.
- One giant page instead of linked concepts.
- Index drift after page creation.
- Raw source edits that invalidate hashes.
- Orphan pages with no links.
- Conflicting claims hidden by overwrite.
- Mixing generated wiki prose into product documentation without provenance.

## Verification Checklist

- [ ] Existing wiki oriented before mutation.
- [ ] Raw source stored before synthesis.
- [ ] Frontmatter and tags valid.
- [ ] Source hashes match.
- [ ] Links resolve; index is complete.
- [ ] No unexplained orphans.
- [ ] Log contains current action.
- [ ] Validator exits 0 or findings are explicitly reported.
