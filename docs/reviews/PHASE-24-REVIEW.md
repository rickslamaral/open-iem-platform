# Phase 24 Review — Release 0.3.1 version consistency

## Objetivo
Corrigir falha real do pipeline de release: `v0.3.0` apontava para commit anterior à sincronização dos manifests.

## Status: PASS WITH CONDITIONS — Phase 24 local gates pass; CI remoto bloqueado por infraestrutura

## Implementação Phase 24
- `ws_musician_db_failure_denies_set_send_gain` remove `mix_assignments` em teste, confirma `FORBIDDEN` e valida autorização fail-closed.
- WebSocket envia Ping a cada 30 segundos, aceita Pong e encerra após 60 segundos sem Pong com `CONNECTION_TIMEOUT`.
- Todas as escritas WebSocket têm timeout de 10 segundos contra clientes lentos que retenham tarefas e recursos.
- ARM64 CI já possuía target Rust, `gcc-aarch64-linux-gnu` e linker configurado; nenhuma alteração necessária.

## Verificação Phase 24
- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: 216 testes PASS, 0 falhas.
- Testes de protocolo Ping/Pong e frame binário: PASS; testes temporais determinísticos de keepalive ainda pendentes, pois intervalos de produção são 30/60 segundos.
- Quota WebSocket conta todo frame recebido, incluindo control frames; quotas por usuário/IP continuam pendentes.
- PipeWire, WebRTC media, ARM64 runtime e GitHub Actions continuam não validados nesta máquina.

## Auditoria de estado (09/09/2026)
- `main` está em `8db59da`; PR #38 já foi mergeada. Registros antigos que diziam PR aberta estavam obsoletos.
- Rust workspace: testes locais passaram após estabilizar dois testes de broadcast WebSocket sujeitos a corrida de agendamento.
- Frontends Musician e Engineer: typecheck, testes e build passaram localmente.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh`, `cargo fmt` e `cargo clippy -D warnings`: PASS.
- PipeWire, mídia WebRTC real, ARM64 em Raspberry Pi e CI remoto: não validados nesta máquina; permanecem `SIMULATED`/`BLOCKED`.

## Alterações
- Workspace Rust, Musician, Engineer e lockfiles usam `0.3.1`.
- README, CHANGELOG, TODO e Development Log registram estado real.
- Tag histórica `v0.3.0` não foi reescrita.

## Verificação
- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- Frontends typecheck, testes e build: PASS na rodada desta execução.
- `scripts/validate-skills.sh`: PASS.
- `scripts/validate-docs.sh`: PASS.
- CI remoto anterior `34408047367`: FAIL em `Validate version consistency`; demais jobs skipped.

## Segurança
O helper destrutivo usado para fault injection de autorização é compilado somente em builds de debug (`cfg(debug_assertions)`) e não fica disponível no binário de release. Nenhum secret adicionado. Tag não foi force-pushed.

## Limitações
`v0.3.1` foi commitado e tagueado, mas CI/Release falharam imediatamente no GitHub Actions antes de executar steps: CI `34413429554`, Release `34413431504` (reexecução 2). Artefatos não publicados. ARM64, PipeWire, Opus e WebRTC continuam SIMULATED/não validados no VPS.

## Follow-up desta rodada
- Corrigido diagnóstico de Docker Compose para verificar plugin ou executável standalone.
- `make run-local` agora descreve execução local com áudio SIMULATED; não finge fornecer configuração ou servidor de simulação separado.
- Frames WebSocket binários agora retornam `INVALID_MESSAGE` e encerram conexão, alinhando implementação com contrato fail-closed.
- Removida entrada duplicada no Development Log.
- Reviews independentes registraram ausência de testes temporais de keepalive e backlog para quota de conexões, revogação pós-emissão e redaction explícito.

## Follow-up desta rodada
- Política de keepalive extraída para `KEEPALIVE_INTERVAL`/`KEEPALIVE_TIMEOUT`.
- Helper puro `keepalive_expired` cobre fronteiras antes e exatamente no timeout.
- Adicionado `MAX_WEBSOCKET_CONNECTIONS = 64` com `Semaphore`; upgrades acima do limite retornam HTTP 503 e `Retry-After: 5`.
- Permissão acompanha conexão até o fim do handler, evitando vazamento de slots.
- Loop WebSocket ainda usa intervalos de produção de 30/60 segundos; teste temporal de integração continua pendente.

## Segurança e limites restantes
- Quota por conexão conta todo frame recebido, incluindo control frames; quotas por usuário/IP e revogação pós-emissão de JWT ainda não implementadas.
- Revogar sessão refresh ou excluir usuário não derruba access JWT já emitido até expiração.
- CLI administrativo ainda exige disciplina operacional para não expor token/senha em argv e não deve ser usado sobre HTTP remoto.
- Nenhum secret adicionado.

## Follow-up Phase 25 — redação de erros e observabilidade
- Erros de protocolo WebSocket agora usam códigos/mensagens genéricos (`INVALID_JSON`, `UNSUPPORTED_VERSION`, `INVALID_REQUEST_ID`, `MESSAGE_TOO_LARGE`); detalhes de parser não são enviados.
- Falha de transporte e fechamento normal não são classificados como expiração JWT.
- Logs de recebimento não incluem erro bruto; usam `session_id`.
- Teste unitário confirma que conteúdo inválido não vaza na mensagem pública.

## Phase 27 follow-up — recuperação de estado após lag
- `RecvError::Lagged` em ambos canais de broadcast agora envia `State` com revisão autoritativa.
- Musician refaz `GET /api/v1/state` ao receber `State`, com proteção contra snapshots atrasados já existente.
- Musician valida e aplica `MasterAck` no snapshot local.
- Teste do hook cobre atualização de master por `MasterAck`.

## Próximo
Investigar indisponibilidade/configuração do GitHub Actions; não declarar release nem merge até CI executar todos os gates e passar. Revogação pós-emissão de JWT, quotas por usuário/IP e teste temporal do loop continuam pendentes.
