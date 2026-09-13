# Phase 35 Review — CLI de desenvolvimento `iem`

**Data:** 2026-09-11
**Estado:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

## Objetivo

Criar binário `iem` com paridade explícita aos alvos seguros do Makefile, sem duplicar lógica, inventar instalação global ou prometer suporte de plataforma.

## Implementado

- Novo binário `iem` em `server/admin-cli/src/bin/iem.rs`.
- Comandos fixos: `help`, `status`, `diagnostics`, `docs`, `test`, `build`, `up` e `down`.
- Dispatcher usa `std::process::Command` sem shell e sem argumentos arbitrários.
- Código de saída do Makefile é preservado; falha ao iniciar `make` retorna 127.
- `iem run` permanece fora do contrato.
- Contrato registrado em `docs/CLI.md`; README, TODO e CHANGELOG atualizados.

## Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo test --manifest-path server/Cargo.toml -p admin-cli`: PASS — 3 testes (1 `iem`, 2 `open-iem-admin`).
- `cargo clippy --manifest-path server/Cargo.toml -p admin-cli --all-targets -- -D warnings`: PASS.
- `cargo run --quiet --manifest-path server/Cargo.toml --bin iem -- --help`: PASS; lista 8 comandos.
- `cargo run --quiet --manifest-path server/Cargo.toml --bin iem -- docs`: PASS.
- `cargo run --quiet --manifest-path server/Cargo.toml --bin iem -- run`: PASSA como rejeição — exit 2, subcomando não reconhecido.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.
- CI remoto: runs recentes falham antes dos steps com `runner_id=0`; não representa execução de código.

## Limitações

- Não há instalação global, pacote ou artefato de release para `iem`.
- `up` e `down` dependem de Docker Compose; `test` e `build` dependem de toolchains locais.
- PipeWire, WebRTC media e Raspberry Pi 5 permanecem não validados.

## Decisão

**PASS WITH CONDITIONS:** aceitar após gates locais e reviews finais; merge depende de CI remoto executado com sucesso.
