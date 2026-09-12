# Phase 57 Review — validação de archive e origem HTTPS

**Data:** 2026-09-11
**Status:** PASS WITH CONDITIONS — implementação local; CI remoto bloqueado; não mergeado.

## Objetivo

Fechar dois gaps encontrados em revisão independente: extração ARM64 dependia de validação manual mais fraca que o validador versionado, e o serviço não declarava origem HTTPS usada pelo Caddy.

## Implementado

- `deployment/raspberry-pi/README.md` valida archive com `scripts/validate-release-archive.py` antes da extração.
- `deployment/systemd/openiem-server.service` define `OPENIEM_ALLOWED_ORIGINS=https://iem.local`.
- Guia documenta atualização de Caddyfile e allowlist quando mDNS não estiver disponível.
- README, CHANGELOG, TODO e DEVELOPMENT-LOG atualizados.

## Verificação

- `make validate`: PASS.
- `cargo audit --file server/Cargo.lock --ignore RUSTSEC-2023-0071`: PASS — 0 vulnerabilidades.
- `npm audit --audit-level=high` em `web/musician` e `web/engineer`: PASS.
- `python3 -m pytest -q tests/test_validate_release_archive.py`: PASS — 7 testes.
- Revisões independentes: findings corrigidos; nenhum blocker de segurança ou lógica permanece no diff atual.

## Limitações

CI remoto falha antes dos steps com `runner_id=0`; API de permissões/runners retorna HTTP 403. Caddy, instalação privilegiada, runtime ARM64, PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 não foram validados.

Checksum continua vindo do mesmo release que fornece archive; attestation independente ainda depende de CI funcional e não substitui validação em hardware.

## Decisão

PASS WITH CONDITIONS. Merge e release permanecem bloqueados até CI remoto executar gates reais e hardware Raspberry Pi 5 validar deployment e áudio.
