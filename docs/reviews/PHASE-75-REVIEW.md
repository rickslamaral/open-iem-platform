# Phase 75 Review — fonte única de versão e gate reproduzível

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local aprovada; CI remoto, release e hardware pendentes.

## Escopo

- Centralizar versão em `VERSION`.
- Validar SemVer, manifests Rust/frontend e tag sem parsing frágil no workflow.
- Cobrir divergência de versão com testes locais.

## Mudanças

- `VERSION` contém `0.3.1`.
- `scripts/validate-version.py` valida `VERSION` com `tomllib`, `server/Cargo.toml`, os dois `package.json` e tag `vX.Y.Z`.
- `release.yml` repete o gate com `tomllib` inline, sem executar script mutável do checkout antes de validar a versão.
- `make test` executa `tests/test_validate_version.py`.
- README, CHANGELOG, TODO e Development Log registram estado real.

## Segurança e limites

- Nenhum secret incluído.
- Gate falha fechado em SemVer inválido, drift de manifest ou tag divergente.
- Assinatura de release ainda precisa separar código de checkout de segredo de assinatura antes de publicação.
- CI remoto permanece bloqueado antes dos steps; nenhum release foi publicado.
- ARM64 é somente cross-build até execução real; PipeWire/ALSA, WebRTC media e Raspberry Pi 5 permanecem não validados.

## Gates locais

- `python3 scripts/validate-version.py --tag v0.3.1`: PASS — imprimiu `0.3.1`.
- `python3 -m pytest -q tests/test_validate_version.py`: PASS — 4 testes.
- `make validate`: PASS — Rust, frontend, testes, documentação, PDF e skills.
- `git diff --check`: PASS.

## Arquivos

- `VERSION`
- `scripts/validate-version.py`
- `tests/test_validate_version.py`
- `.github/workflows/release.yml`
- `Makefile`
- `README.md`
- `CHANGELOG.md`
- `docs/TODO.md`
- `docs/DEVELOPMENT-LOG.md`
