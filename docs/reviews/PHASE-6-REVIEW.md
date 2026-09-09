# Phase 6 Review — Engineer Console

**Data:** 2026-09-09
**Status:** PASS WITH CONDITIONS
**Ambiente:** VPS Linux; áudio SIMULATED, sem PipeWire ou Raspberry Pi 5

## Entrega

- `web/engineer` deixou de ser scaffold.
- Login chama `POST /api/v1/auth/login`; access token fica somente em memória.
- Resposta 401 tenta renovar via cookie HttpOnly em `POST /api/v1/auth/refresh`; falha de renovação limpa sessão.
- Dashboard chama `GET /api/v1/audio/sessions`, `GET /api/v1/mixes` e `GET /api/v1/state`.
- Polling limitado a 5 segundos, ativo somente quando documento está visível.
- Engineer atribui e remove mixes pelos endpoints existentes.
- Erros HTTP e rede aparecem sem dados fictícios.
- Interface marca claramente áudio e sessões como `SIMULATED`.

## Verificação

- `npm run typecheck --prefix web/engineer`: PASS
- `npm test --prefix web/engineer -- --run`: PASS — 2 testes
- `npm run build --prefix web/engineer`: PASS — bundle JS 147.59 kB, gzip 47.61 kB
- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 172 testes
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS
- `bash scripts/validate-docs.sh`: PASS

## Condições e pendências

- API atual retorna somente revision, assignments e sessões; UI não inventa canais, meters ou DSP.
- Assignment usa ID numérico porque catálogo de usuários exige Admin.
- Token não sobrevive a reload; não usar `localStorage`.
- PipeWire, Opus, WebRTC media path e latência real seguem não validados.
- TLS continua obrigatório antes de exposição externa.

## Próxima fase

Adicionar snapshot de canais e telemetria somente após contrato API, testes RBAC e validação em hardware. Não declarar suporte Raspberry Pi baseado neste build VPS.
