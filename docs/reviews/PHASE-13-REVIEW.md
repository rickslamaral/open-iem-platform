# Phase 13 Review — Signaling ownership hardening

**Data:** 2026-09-09
**Ambiente:** VPS Linux; PipeWire/Opus/mídia real SIMULATED
**Status:** PASS WITH CONDITIONS

## Escopo

`POST /api/v1/audio/offer` deixou de aceitar `mix_id` arbitrário para músicos.
O handler valida índice numérico, capacidade (`MAX_MIXES`) e assignment SQLite
do `uid` autenticado antes de chamar registry WebRTC.

## Gates

| Gate | Resultado |
|---|---|
| Formatação Rust | PASS |
| Testes workspace | PASS — 173 testes, 0 falhas |
| Clippy `-D warnings` | PASS |
| Teste novo de músico sem assignment | PASS — 403 |
| Secret scan | PASS — nenhuma credencial adicionada |
| Security review independente | PASS após correção do finding de ownership |
| Hardware PipeWire/Opus | SIMULATED — VPS sem hardware |

## Segurança

- `MUSICIAN` não pode negociar mix não atribuído.
- `ENGINEER` e `ADMIN` mantêm controle operacional.
- Middleware JWT, Origin/CSRF e limite global de body permanecem ativos.
- Nenhum token, SDP ou endereço de rede é adicionado a logs/respostas.

## Condições pendentes

1. Adicionar rate limiting HTTP para login, offer e ICE.
2. Limitar globalmente sessões WebRTC e aplicar idle timeout.
3. Revogar access tokens imediatamente após exclusão ou alteração de privilégio.
4. Validar mix/roteamento no caminho PipeWire real em Raspberry Pi 5.
5. Criar contrato versionado de snapshot/telemetria antes de ampliar Engineer Console.

## Veredito

Entrega segura para merge no escopo definido. Não autoriza declarar mídia real,
telemetria real ou suporte de hardware Raspberry Pi validado.
