# Phase 87 Review — Engineer Console: master gain/mute via WebSocket

**Status:** PASS — 18 testes verdes, typecheck limpo, build limpo, revisão independente aprovada.

**Date:** 2026-09-13

---

## Objetivo

Adicionar controles interativos de master gain e master mute por mix ao Engineer Console, usando WebSocket com subprotocolo `openiem-v1` e RBAC Engineer/Admin existente no servidor.

## Implementado

| Arquivo | Descrição |
|---------|-----------|
| `web/engineer/src/protocol.ts` | Tipos TypeScript: `ClientMessage`, `ServerMessage`, `MixMasterState`, `WsStatus`, `PROTOCOL_VERSION` |
| `web/engineer/src/useEngineerWs.ts` | Hook React: conexão WS, `GetState` on open, `MasterAck`/`State`/`Error` handlers, reconexão 3 s, guard `mounted` |
| `web/engineer/src/App.tsx` | `WsBadge`, `MixMasterControl` (slider gain + botão mute), integração na App |
| `web/engineer/src/useEngineerWs.test.ts` | 13 testes: disconnect, connect, MasterAck (0/1/OOB), State, Error, send gain/mute, cleanup |
| `web/engineer/src/App.test.tsx` | 5 testes: autenticação, falha API, badge WS, MasterAck gain, MasterAck mute |

## Gates

| Gate | Resultado |
|------|-----------|
| `npm test` — 18/18 testes | ✅ PASS |
| `npx tsc --noEmit` | ✅ PASS |
| `npm run build` | ✅ PASS |
| `cargo test --workspace` (servidor) | ✅ PASS (sem mudanças Rust) |
| Scan segurança (secrets/injection) | ✅ LIMPO |
| Revisão independente | ✅ passed=true |

## Decisões de design

- **Token em URL**: WebSocket não suporta headers customizados no upgrade de browser; `?access_token=` é trade-off padrão, consistente com o hook do músico.
- **Slider dispara gain no `onMouseUp`/`onKeyUp`**: evita flood de mensagens WS durante arrasto contínuo.
- **Guard `mounted`**: cleanup seta `mounted=false` antes de `wsRef.current?.close()`; `onclose` checa `!mounted` → não agenda novo reconnect após unmount.
- **isMasterAck type guard**: valida `msg.type === 'MasterAck'` e `typeof data.mix_index === 'number'`; bounds check 0..1 feito inline antes de atualizar estado.

## Limitações

- Controles não testados em servidor real; servidor SIMULATED em VPS.
- PipeWire, WebRTC e Raspberry Pi 5 permanecem pendentes.
- Sem debounce de `onKeyUp` (sugestão não-bloqueadora do revisor); aceitável para range com step 0.5.
- Sem back-off exponencial no reconnect (sugestão não-bloqueadora); 3 s fixo aceito para MVP.
