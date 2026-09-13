# Phase 70 Review — hardening do validador de bundle

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — validação local concluída; CI remoto, release e hardware pendentes.

## Implementado

- Limites fail-closed: 512 MiB por arquivo, 2 GiB por bundle e 64 KiB por manifesto.
- Leituras usam `O_NOFOLLOW` e `O_NONBLOCK`; SBOM opcional precisa ser JSON objeto válido.
- Manifesto é criado com `O_NOFOLLOW`, modo restrito e validação de arquivo regular.
- Testes offline cobrem limite, SBOM inválido e manifesto symlink.

## Verificação

- `pytest==9.1.1`: 45 testes dos validadores aprovados.
- `make validate`: aprovado.
- `py_compile` e `git diff --check`: aprovados.

## Limitações

CI remoto continua bloqueado antes dos steps por runner/permissões. TOCTOU entre validação e cópia posterior no workflow permanece follow-up. Raspberry Pi 5, PipeWire/ALSA, WebRTC, Windows runtime e release continuam não validados.
