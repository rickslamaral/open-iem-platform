# Matriz de validação de plataforma

**Data:** 2026-09-11
**Escopo:** evidência executada para Open IEM Platform.
**Regra:** `PASS` exige comando executado e resultado registrado; `PASS parcial` significa que somente componentes explicitamente listados passaram; `CONFIG VALIDATED` cobre parsing/configuração, não runtime; `SIMULATED` não é suporte; `PENDING` exige ambiente ou hardware ausente.

## Estado atual

| Plataforma / caminho | Configuração | Evidência disponível | Estado |
|---|---|---|---|
| VPS Linux x86_64 — Rust control plane | Cargo, SQLite, REST, WebSocket | Testes locais do `api-server` e crates; fmt/clippy por pacote | PASS parcial |
| VPS Linux x86_64 — áudio | Mix engine determinístico sem dispositivo | `cargo test -p audio-engine`; harness usa backend `SIMULATED` | SIMULATED |
| Linux x86_64 — Docker Compose | API + duas UIs | `docker compose config`; build/startup de containers não executado | CONFIG VALIDATED |
| Raspberry Pi 5 ARM64 — binário | Cross-compilação definida no release workflow | Nenhum runtime em Pi real | PENDING |
| Raspberry Pi 5 ARM64 — PipeWire/ALSA | PipeWire, WirePlumber, interface USB | Nenhum dispositivo disponível nesta execução | PENDING |
| Raspberry Pi 5 ARM64 — WebRTC media | Captura, transporte, latência e recuperação | Nenhum teste de mídia executado em hardware | PENDING |
| Windows x64 — Docker Desktop | Linux containers via WSL2 | Guia e configuração Compose; startup não executado em Windows | DOCUMENTED, NOT VALIDATED |
| Windows x64 — áudio nativo | WASAPI/ASIO | Backend nativo não implementado | UNSUPPORTED |
| macOS | Servidor e áudio | Fora dos targets validados atuais | BACKLOG |

## Gates para fechar Phase 8/9

1. Executar binário ARM64 no Raspberry Pi 5 alvo.
2. Registrar modelo da interface, kernel, versão PipeWire/WirePlumber e sample rate.
3. Confirmar grafo com `pw-top` e `pw-link`; salvar saída sanitizada em artefato de teste.
4. Medir XRUNs, latência, jitter, perda e recuperação durante sessão controlada.
5. Executar smoke test REST/WebSocket com TLS via Caddy.
6. Repetir após reinício do serviço e perda temporária de rede.
7. Registrar comandos, datas, versões, resultado e limitações nesta matriz.

## Comandos de evidência

### VPS sem hardware

```bash
cargo fmt --manifest-path server/Cargo.toml --all -- --check
cargo clippy --manifest-path server/Cargo.toml -p api-server --all-targets -- -D warnings
cargo test --manifest-path server/Cargo.toml -p audio-engine
cargo test --manifest-path server/Cargo.toml -p api-server --test integration
npm run typecheck --prefix web/musician
npm test --prefix web/musician -- --run --reporter=dot
npm run typecheck --prefix web/engineer
npm test --prefix web/engineer -- --run --reporter=dot
docker compose config
bash scripts/validate-docs.sh
bash scripts/validate-skills.sh
git diff --check
```

Esses comandos validam código e configuração. Não validam PipeWire, áudio realtime, WebRTC media ou ARM64 runtime.

## Segurança

- Não marcar hardware como validado por cross-compilação.
- Não expor Compose HTTP fora de localhost/LAN isolada.
- Validar TLS antes de qualquer exposição externa.
- Remover tokens, chaves, IPs públicos e dados de músicos dos artefatos anexados.
