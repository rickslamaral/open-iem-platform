# Phase 61 Review — abertura fail-closed de archives

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local; CI remoto, release e hardware pendentes.

## Objetivo

Impedir que o validador siga symlinks ou aceite diretórios como archives e eliminar a janela entre validação de tamanho e abertura do arquivo.

## Implementado

- `scripts/validate-release-archive.py` abre a entrada com `O_NOFOLLOW` e `O_CLOEXEC`.
- `fstat()` valida arquivo regular e tamanho no mesmo descritor usado por `tarfile`.
- Falha de abertura e entradas não regulares retornam erro de validação fail-closed.
- Testes offline cobrem symlink e diretório como entradas rejeitadas.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 19 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Review independente: recomendou descritor com `O_NOFOLLOW` e `fstat()` para evitar TOCTOU; implementação aplicada.

## Limitações

CI remoto continua falhando antes dos steps no run `34685058398` e no push correspondente; nenhum release, Caddy, Raspberry Pi 5, PipeWire/ALSA ou mídia WebRTC foi validado.

**Decisão:** PASS WITH CONDITIONS. Não fazer merge/release até CI executável e validação de hardware concluída.
