# ADR-014 — Segurança operacional do Compose de desenvolvimento

- **Status:** Accepted
- **Data:** 2026-09-11

## Contexto

O Compose de desenvolvimento usa HTTP e não valida áudio real. Além disso, iniciar imagens sem rebuild após alterações pode executar código obsoleto, e publicar servidores Vite em todas as interfaces pode expor UIs sem TLS.

## Decisão

- `make up` sempre executa `docker compose up -d --build`.
- Portas das UIs Musician e Engineer ficam vinculadas a `127.0.0.1` no Compose padrão.
- Acesso LAN exige override explícito, rede isolada e avaliação de TLS.
- Compose permanece somente para desenvolvimento; não é configuração de produção.

## Consequências

Alterações locais entram nas imagens iniciadas por `make up`, ao custo de rebuild potencialmente mais lento. Acesso por outro dispositivo não funciona no Compose padrão; isso evita exposição acidental.

## Limites

Esta decisão não valida Docker runtime, TLS, PipeWire/ALSA, WebRTC media, latência ou Raspberry Pi 5.
