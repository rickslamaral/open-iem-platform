# Phase 15 Review — Atomic mix ownership

**Status:** PASS — independent review approved
**Data:** 2026-09-09
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

## Entrega

- `mix_assignment_lock` agora cobre listagem de assignments, leitura de sends e as três mutações de sends.
- Cada operação valida ownership e acessa estado de controle enquanto assignment não pode mudar em paralelo.
- Assign/unassign já usavam mesmo lock. Signaling e snapshot continuam usando lock compartilhado.

## Verificação local

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check` — PASS
- `cargo test --manifest-path server/Cargo.toml --all` — PASS: 141 testes
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings` — PASS

## Segurança

A janela TOCTOU entre consulta de assignment e mutação de send foi fechada para rotas protegidas. Nenhum segredo, shell injection, SQL dinâmico ou hardware real foi introduzido.

## Follow-up de documentação e CI

- README e Musician Guide agora descrevem comportamento entregue, sem pendências já resolvidas.
- Versão `0.2.0` sincronizada em `web/musician` e `web/engineer`, incluindo lockfiles.
- CI dos frontends Musician e Engineer agora falha em instalação, typecheck e teste; scripts existentes não são mais mascarados por `|| echo`.
- Quality gate de release executa typecheck, testes e `npm audit` dos dois frontends.
- Musician Guide PDF regenerado via pandoc/xelatex; `file` confirma PDF 1.5. Extração não validada porque `pdftotext` não está instalado no VPS.

## Limitações

PipeWire, Opus, mídia WebRTC e telemetria real continuam SIMULATED no VPS. TLS de exposição externa continua obrigatório e pendente.

Hardware Raspberry Pi 5 permanece não validado.

## Correção posterior

- O exemplo de bind LAN foi corrigido para `OPENIEM_BIND_ADDR` e `OPENIEM_ALLOW_INSECURE_HTTP`, que são os nomes lidos pelo binário atual.
- README, ADR-008 e TODO foram alinhados ao estado real da autenticação.

## Veredito do follow-up

Documentação, configuração publicada e gate de CI alinhados ao estado real. Aviso HTTP LAN explicita risco e isolamento; rate limiting HTTP não é alegado. Entrega não altera mídia, autenticação ou contratos de API.
