# ADR-012 — Quotas de conexões WebSocket

## Status

Accepted — Phase 28.

## Contexto

O teto process-wide de 64 conexões não impedia um único usuário ou endereço IP de consumir toda capacidade do servidor. O servidor roda como processo único no MVP; não há coordenador distribuído.

## Decisão

Aplicar admissão atômica em memória com três limites:

- 64 conexões por processo;
- 4 conexões por `user_id` autenticado;
- 16 conexões por `IpAddr` do peer TCP.

A identidade de rede vem de `ConnectInfo<SocketAddr>`. `X-Forwarded-For` não é confiável e não participa da política. Reserva usa guarda RAII; descarte da guarda libera contadores. Limite global retorna HTTP 503; limites por usuário/IP retornam HTTP 429, ambos com `Retry-After: 5`.

## Consequências

Positivas:

- Evita concentração de conexões por usuário ou IP.
- Reserva e liberação são serializadas em uma seção crítica única.
- Não adiciona banco, Redis ou dependência operacional.

Limitações:

- Quota vale somente dentro de cada processo. Múltiplas réplicas exigem coordenador compartilhado.
- Clientes atrás de NAT compartilham quota por IP.
- Falhas de autenticação `/ws/v1` agora têm limiter local bounded: 5 por IP em 60 segundos, com no máximo 4.096 IPs retidos; HTTP 429 usa `Retry-After: 5`.
- Revogação pós-emissão de access JWT foi implementada na Phase 31 com associação persistida à sessão refresh; validação CI e hardware continuam pendentes.

## Validação

Testes unitários cobrem limites, liberação RAII, ausência de alteração em reserva rejeitada e concorrência. Suite Rust, Clippy e validações de documentação devem passar antes do commit.
