# ADR-009 — Ownership no signaling de áudio

- **Status:** Accepted
- **Data:** 2026-09-09
- **Fase:** 13

## Contexto

`POST /api/v1/audio/offer` aceitava `mix_id` fornecido por músico sem comparar
com assignment persistido. O transporte de mídia continua SIMULATED no VPS,
mas esse valor será usado no roteamento real.

## Decisão

Para `Role::Musician`, `mix_id` deve ser índice numérico válido e igual ao mix
atribuído ao `uid` do JWT. Falha retorna `403 Forbidden`. Engineer e Admin
mantêm capacidade operacional sobre mix IDs válidos. Ausência de `mix_id`
continua permitida para preservar negociação sem seleção explícita.

A validação ocorre antes de SDP parsing e antes de criar/substituir sessão.

## Consequências

- Impede músico de negociar mix de outro músico no boundary HTTP.
- Mantém API compatível para Engineer/Admin e ofertas sem mix explícito.
- O contrato ainda precisa resolver `mix_id` para um identificador de mix
  canônico antes de ativar PipeWire/Opus real.
- Rate limiting, limite global de sessões e revogação pós-emissão de access token
  são tratados por decisões posteriores (Phases 28/29/31); não são mascarados por esta decisão.
