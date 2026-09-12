# Phase 63 Review — verificação fail-closed de arquivos assinados

## Follow-up de revisão

- `O_NOFOLLOW` agora é obrigatório; ausência da flag falha fechado, sem aceitar symlink.
- `O_NONBLOCK` impede que FIFO/dispositivo especial bloqueie antes do `fstat()`.
- `/usr/bin/openssl` remove dependência de `PATH` controlável.
- Review independente pós-correção: **PASS**, sem concerns de segurança ou erros lógicos.


**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local concluída; CI remoto, release e hardware pendentes.

## Objetivo

Eliminar a janela entre checagem de caminho e leitura durante verificação de assinatura detached Ed25519.

## Implementado

- `scripts/verify-release-signature.py` abre artifact, assinatura e chave com `O_NOFOLLOW` e `O_CLOEXEC`.
- Cada descritor é validado com `fstat()` como arquivo regular.
- OpenSSL recebe `/proc/self/fd/N` com `pass_fds`, mantendo conteúdo validado estável durante verificação.
- Exceções de execução do OpenSSL falham explicitamente.
- Testes cobrem symlink em artifact e assinatura.

## Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 21 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Review independente: encontrou e corrigiu caminho de `OSError` que deixava `result` não inicializado.

## Limitações

`/root/scan_patterns.py` não está disponível neste ambiente. GitHub Actions continua falhando antes dos steps por runner/permissões. Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC e release continuam não validados.

**Decisão:** PASS WITH CONDITIONS. Não fazer merge ou release até CI executável e gates remotos verdes.
