# ADR-012 — Limite process-wide de conexões WebSocket

- **Status:** Accepted
- **Data:** 2026-09-10
- **Fase:** 24

## Contexto

Cada upgrade WebSocket cria um handler persistente. Sem limite, clientes autenticados podem abrir conexões ilimitadas e consumir memória, tarefas e sockets do processo.

## Decisão

`api-server` mantém `tokio::sync::Semaphore` com 64 permits em `AppState`. O permit é reservado antes do upgrade e movido para o handler, sendo liberado automaticamente quando a conexão termina. Quando não há permit, `/ws/v1` retorna HTTP 503 com `Retry-After: 5`.

## Consequências

- Há proteção global por processo contra crescimento ilimitado de conexões.
- Limite não é distribuído entre réplicas.
- Não há quotas separadas por usuário/IP.
- Revogação pós-emissão de access JWT ainda precisa ser implementada; expiração continua sendo verificada no loop WebSocket.
- Valor 64 deve ser reavaliado após medição em Raspberry Pi 5.
