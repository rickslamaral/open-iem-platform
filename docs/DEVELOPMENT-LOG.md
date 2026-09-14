## 2026-09-14 — P0-005: clock, drift and adaptive resampling

**Status:** implementação SIMULATED; validação L1/L2 local concluída; hardware pendente.

- Adicionado `streaming::clock` com `SampleTimestamp` baseado no contador de amostras da captura, sem usar relógio de parede.
- `DriftEstimator` aplica filtro limitado a ±500 ppm; `AdaptiveResampler` limita razão a 0,9995–1,0005 e usa profundidade de buffer alvo.
- Testes cobrem baseline, convergência, limites e soak de 10.000 atualizações sem crescimento ilimitado.
- Nenhuma alegação de sincronização entre receptores foi adicionada.

**Próximo:** P0-006 — pairing, identidade, revogação e binding DTLS-SRTP.

# Development Log

All significant milestones documented here in reverse chronological order.

---

## 2026-09-14 — P0-002 Audio Lab L1/L2 (PR #57)

**Branch:** feat/p0-002-audio-lab
**Commit:** 89a807e
**Status:** PR aberta, aguarda CI remoto

### O que foi implementado

- `server/audio-engine/tests/audio_lab_l1_l2.rs` — 8 testes SIMULATED:
  - L1 (Docker/PipeWire virtual): pipeline determinism, frame budget 48k frames, control queue overflow (200 > 64 capacity), xrun count invariante
  - L2 (ALSA virtual): buffer 48kHz/64f, buffer 48kHz/256f, stereo isolation ch0→mix0/ch1→mix1, master mute silence
  - Cada teste emite linha JSON de evidência: {lab_profile, frames, xrun, cpu_us, evidence_level=SIMULATED}
- `.github/workflows/ci.yml` — job `audio-lab` adicionado com SHAs de ação fixados; verifica marcadores de evidência no output

### Gates locais

- `cargo fmt --check`: OK
- `cargo clippy -D warnings`: OK
- `cargo test --test audio_lab_l1_l2`: 8/8 OK
- Revisão independente: passed=true, sem security_concerns, sem logic_errors

### Nível de evidência

SIMULATED — usa SimulatedBackend. Não é PipeWire runtime, não é ALSA hardware real, não é Docker.
L3/L4 permanecem hardware-only conforme ADR-010.

### Impacto em GAPs

- GAP-014: IMPLEMENTATION GAP → resolvido no nível CI (evidência SIMULATED)
- GAP-010/GAP-009: sem mudança — PipeWire real e Pi 5 pendentes (P0-008/P0-010)

### Próximo

P0-003 — Media Plane: conectar frames do MixEngine à sessão WebRTC.

---

### 2026-09-14 — P0-001: fronteira realtime bounded

**Objetivo:** remover `Arc<Mutex<JackState>>` do callback JACK antes de qualquer validação de hardware.

**Implementado:** `audio-engine::rt_boundary` agora usa `crossbeam-channel::bounded(64)`. Produtor de controle usa `try_send`; fila cheia rejeita comando mais novo. `RealtimeProcessor` consome comandos com `try_recv` e processa `MixEngine` sem mutex, I/O ou espera bloqueante. Callback JACK usa processador diretamente e processa cada frame do período sem alocação no callback.

**Testes:** `cargo test --manifest-path server/Cargo.toml --workspace` PASS; `cargo clippy --manifest-path server/Cargo.toml -p audio-engine --all-targets -- -D warnings` PASS.

**Limitações:** feature JACK/PipeWire e hardware não foram executados neste host; callback permanece stub de backend, sem conexões de portas e sem métricas de XRUN. Nenhuma claim de runtime/hardware alterada.

**Próximo:** P0-002 — Audio Lab L1/L2.

---

### 2026-09-14 — Final architecture closure and development handoff

**Status:** Architecture decisions closed with warnings; implementation and physical validation remain pending.

- ADR-001..010 updated with technical decisions, consequences, risks and validation gates.
- `docs/ARCHITECTURE-GAPS.md` reconciled into 33 canonical GAPs.
- `docs/DEVELOPMENT-HANDOFF.md` created; P0-001 is complete in code and next item is P0-002, L1/L2 Audio Lab.
- No audio/media/runtime/hardware claim changed to validated.
- No ESP32 scope added.
- Development cron reactivation and exact verification recorded in final report.


---

### 2026-09-14 — Documentation and status synchronization

**Status:** Documentation cleanup after PR #49, #50, #52 and #53 merged into `main`.

- README status now reflects Phase 92 as current work.
- Stale claims of blocked CI, unmerged PRs and `runner_id=0` removed from current status sections.
- CHANGELOG consolidated; duplicate Phase 85 entry removed.
- TODO now records current release, hardware and Phase 92 gates.
- Release `v0.3.1` tag exists, but GitHub Release is not published.
- PipeWire/ALSA, WebRTC/Opus, dedicated Linux installation and Raspberry Pi 5 remain unvalidated.

---

### 2026-09-14 — Phase 92: WebSocket EQ band control

**Status:** Backlog. No implementation commit or open PR is confirmed in current repository state.

Phase 92 requires protocol design, validation, broadcast/RBAC implementation, local gates, independent review and real CI before PR.

---

### 2026-09-13 — Phase 89: Engineer Console Channel Strip

**Goal:** Expose existing authenticated channel gain/mute API in Engineer Console.

**Implemented:**
- Render input channels from `/api/v1/state`.
- Add gain slider (`-144..+12 dB`) and mute button per channel.
- Keep Bearer authentication, disable locked channels, debounce gain writes and guard optimistic state against stale mutation/load responses.

**Validation:** 10 frontend tests, typecheck and production build passed locally. CI, real server, PipeWire/ALSA and Raspberry Pi 5 remain pending.

---

### 2026-09-13 — Phase 88: Musician UI master gain/mute genuinamente somente leitura

**Goal:** Corrigir UX enganoso: slider de master gain no músico alterava estado local mas nunca enviava ao servidor (RBAC bloqueia Musician para SetMasterGain/SetMasterMute desde Phase 23/87). Tornar o controle genuinamente read-only.

---

### 2026-09-13 — examples/minimal-mix: runnable mix engine example

**Goal:** Create `examples/` with a minimal mix scenario (LOW backlog item).

**Implemented:**
- `server/mix-engine/examples/minimal_mix.rs`: self-contained runnable example demonstrating two independent monitor mixes from two input channels (vocals at +3 dB trim, kick at unity). Shows per-send gain/pan/mute, master gain (−3 dB), master mute, and the full `Sum → EQ → Compressor → Master Gain → Limiter` audio chain. Self-checking assertions verify: Mix 0 centre-panned with L≈R; Mix 1 L>R with vocals hard-left; muted mix outputs exactly 0.0.
- `examples/minimal-mix/README.md`: signal graph, expected output, run command and notes.

**Tests/Verification:**
- `cargo run --example minimal_mix` → all assertions pass; expected output confirmed.
- `cargo fmt --all -- --check` → PASS.
- `cargo clippy --all-targets -- -D warnings` → PASS (0 warnings).
- `cargo test --workspace` → PASS.
- Frontend typecheck/tests/build and static scan → PASS/CLEAN.

**Limitations:** Audio processing SIMULATED. No hardware.

### 2026-09-13 — Phase 86 — lock-free broadcast fan-out

**Status:** IMPLEMENTED — local gates pass.

### Implemented

- Removed `mix_assignment_lock` from the two WebSocket broadcast fan-out branches (`send-delta` and `master-delta` receivers) in `server/api-server/src/ws.rs`.
- Ownership check on receiver side (`musician_assigned_to_mix`) is now a lock-free SQLite read.
- Lock remains held by mutation senders, preserving mutation ordering.
- Narrow stale-read race is documented; client reconciles via REST snapshot.

### Rationale

Read-only ownership checks no longer contend on the mutation mutex during broadcast fan-out.

### Verification

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | ✅ PASS |
| `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| `cargo test --workspace` | ✅ PASS |
| Independent review | ✅ passed=true |

### Limitations

Release `v0.3.1`, real installer, PipeWire/WebRTC media and Raspberry Pi 5 hardware remain pending.

## 2026-09-13 — Phase 85 — Rust code coverage reporting

**Status:** IMPLEMENTED — CI job added; remote CI pending runner quota.

### Implemented

- `make coverage` target: guards on `cargo-llvm-cov` presence, prints summary, writes `coverage/lcov.info`.
- CI job `Rust Code Coverage` in `.github/workflows/ci.yml`:
  - Rust stable + `llvm-tools-preview` component.
  - `cargo-llvm-cov 0.9.1 --locked` installed per run.
  - Generates summary (stdout) + LCOV artifact (`coverage-lcov`, 30-day retention).
  - Pins: `actions/checkout` `3d3c42e5`, `dtolnay/rust-toolchain` `6bed0761`, `actions/cache` `0057852b`, `actions/upload-artifact` `043fb46d` (all SHA-locked).
- `coverage/` already in `.gitignore`; no generated files committed.
- Backlog item checked off in `docs/TODO.md`.

### Baseline (local, 2026-09-13)

| Metric | Value |
|--------|-------|
| Lines covered | **81.22%** (4542/5592) |
| Functions covered | **73.85%** (545/738) |
| Crates | mix-engine, audio-engine, control-protocol, control-server, api-server, streaming, admin-cli |
| Backend | SIMULATED (no Raspberry Pi 5 hardware) |

### Limitations

Audio backend, PipeWire/ALSA and hardware paths remain SIMULATED. Release `v0.3.1` and Raspberry Pi 5 validation still pending.

---

## 2026-09-13 — Phase 85 — Rust code coverage reporting

**Status:** IMPLEMENTED — local coverage target and CI job added; remote CI evidence pending.

### Implemented

- `make coverage` checks for `cargo-llvm-cov`, prints summary, creates `coverage/`, and writes `coverage/lcov.info`.
- CI job `Rust Code Coverage` installs `cargo-llvm-cov 0.9.1`, generates LCOV, and uploads `coverage-lcov` artifact for 30 days.
- Coverage baseline: 81.21% lines and 73.85% functions across Rust workspace; audio remains `SIMULATED`.

### Verification

- Independent review found and fixed fresh-checkout output-directory failure.
- Existing fan-out lock-free change preserved; no unrelated code reverted.

### Limitations

Remote CI, release `v0.3.1`, real installation, PipeWire/ALSA and Raspberry Pi 5 remain pending.

## 2026-09-13 — Phase 84 — reprodutibilidade do pipeline de release

**Status:** IMPLEMENTED — confirmação de dois builds e release ainda pendentes.

### Implementado

- Workflow de release usa `--locked` em clippy, testes e builds Rust.
- Archives server e web usam `SOURCE_DATE_EPOCH` derivado do commit, ordenação estável, timestamps fixos e ownership numérico zero.
- Checksums e assinaturas passam a ser calculados depois do empacotamento determinístico.

### Verificação real

- YAML do workflow validado pelo editor após cada alteração.
- Revisão independente identificou o risco de não reprodutibilidade; correção aplicada somente no workflow.
- Não foi possível alegar dois builds CI do mesmo tag nesta execução.

### Limitações

Release `v0.3.1`, secrets de assinatura, instalação real e Raspberry Pi 5 continuam pendentes. Áudio e mídia permanecem `SIMULATED`.

---

## 2026-09-13 — CI verification refresh

**Status:** VERIFIED — runs `34761731828` e `34763037883` passaram todos 9 jobs em `main`.

### Evidence

- GitHub Actions confirmed Rust, frontend, security, archive and documentation gates.
- Local Musician: typecheck, 42 tests and production build passed.
- Local Engineer: typecheck, 2 tests and production build passed.
- `make validate` passed; PDF extraction/rendering and 11 skill validations passed.
- Local full Rust gate is blocked by missing system `jack.pc`; no code failure shown.
- `pip check` reports pre-existing environment conflict: `pyopenssl 25.0.0` requires `cryptography<45`, host has `cryptography 46.0.7`.

### Decision

No implementation change. Release `v0.3.1`, real installer execution, PipeWire/WebRTC media and Raspberry Pi 5 validation remain pending.

---

## 2026-09-13 — CI recovery after actions/cache pin correction

**Status:** VERIFIED — run `34760297250` passed all 9 jobs on `main`.

### Evidence

- Rust format, clippy, tests and x86_64/ARM64 cross-build passed.
- Musician and Engineer typecheck, tests and builds passed.
- Python release archive/bundle security tests passed.
- Rust and npm security audits passed.
- Documentation, PDF and agent skill validation passed.
- Release workflow signing commands now include `-rawin`, required by OpenSSL 3.5 for raw Ed25519 input.

### Decision

CI is no longer blocked by runner/quota failure. Release `v0.3.1`, real installer execution, PipeWire/WebRTC media and Raspberry Pi 5 validation remain pending. No hardware support claim changed.

---

## 2026-09-13 — Hotfix CI — actions/cache SHA inválido

**Status:** MERGED — ff0c085 em main; CI em curso.

### Causa raiz

Commit `8a34d5b` introduziu SHA `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` rotulado como `actions/cache@v6.1.0`. Esse SHA não existe no repositório `actions/cache`; todos os 10 jobs de CI falharam em ~5 segundos sem executar nenhum step.

### Correção

SHA revertido para `6849a6489940f00c2f30c0fb92c6274307ccb58a` (v4.1.2) — SHA verificado via `gh api repos/actions/cache/git/ref/tags/v4.1.2`. Aplicado em 5 ocorrências: ci.yml (linhas 32 e 76) e release.yml (linhas 72, 138, 257).

### Verificação local

- `python -m pytest -q tests/`: 60 aprovados.
- `cargo test --all`: ok.
- `cargo audit --ignore RUSTSEC-2023-0071`: limpo.
- `validate-skills.sh`, `validate-docs.sh`, `validate-pdf.sh`: todos passando.
- Reviewer independente: `passed=true`, sem security_concerns, sem logic_errors.

### Limitações

CI remoto em andamento. Instalação real, Raspberry Pi 5 e runtime PipeWire continuam não validados.

---

## 2026-09-13 — Phase 83 — Controles de pan e mudo master na UI do músico

**Status:** MERGED — e0f0d40 em main; CI bloqueado por quota de Actions (runner_id=0).

### Implementado

- `Channel.tsx`: slider de panorama estéreo (-1 a +1) com rótulo L/C/R; desabilitado quando mudo.
- `Channel.module.css`: estilos `.panRow` e `.panLabel`.
- `Channel.test.tsx`: 6 novos testes (total: 42 passando).
- `MixControl.tsx`: prop `masterMuted` (somente leitura — sem `SetMasterMuted` no protocolo); badge MASTER MUTED visível quando servidor reporta mudo master; `panByChannel` e `onChannelPan` propagados para cada canal.
- `MixControl.module.css`: estilos `.masterRow` e `.masterMutedBadge`.
- `MixControl.test.tsx`: 2 novos testes para badge de mudo master.
- `App.tsx`: estado `panByChannel` sincronizado do snapshot; callback `handleChannelPan` envia `SetSendPan`; `masterMuted` derivado de `ws.snapshot?.mixes[0]?.master_muted`.

### Verificação local

- `npm run typecheck --prefix web/musician`: aprovado.
- `npm test --prefix web/musician -- --run`: 42 testes, 5 arquivos, todos aprovados.
- `npm run build --prefix web/musician`: aprovado.
- Reviewer independente: `passed=true`, sem security_concerns, sem logic_errors.
- Scan estático: sem secrets, sem shell injection, sem eval/exec.

### Limitações

CI remoto bloqueado por quota GitHub Actions (`runner_id=0`). `masterMuted` é somente leitura (sem `SetMasterMuted` no protocolo WebSocket — TODO registrado). Áudio real, PipeWire e Raspberry Pi 5 não validados.

---

## 2026-09-13 — Phase 82 — Node.js preflight antes de mutar host

**Status:** PASS WITH CONDITIONS — preflight corrigido; instalação real, CI novo e Raspberry Pi 5 não validados.

### Implementado

- `preflight_node_check()` adicionada a `scripts/install.sh`; chamada imediatamente antes de `install_deps`.
- Se `node` já estiver em PATH e versão < 20, fatal sem instalar pacotes. Se `node` ausente, host não mutado antes da verificação.
- `check_tools()` mantém validação final pós-instalação.

### Verificação real

- `bash -n scripts/install.sh`: aprovado.
- `scripts/install.sh --dry-run --skip-deps --ref <sha40>`: aprovado; host não alterado.
- Node.js v22.22.3 no host: preflight passa corretamente (major=22 >= 20).
- Review independente: `passed=true`, sem security_concerns, sem logic_errors.

### Limitações

Instalação real, rollback exercitado, CI novo, release v0.3.1, PipeWire/ALSA, WebRTC e Raspberry Pi 5 continuam não validados.

---

## 2026-09-13 — Phase 82 — pin e staging atômico do instalador

**Status:** BLOCKED — correção local; novo CI e revisão independente pendentes; PR #41 não mergeado.

### Implementado

- `scripts/install.sh` exige SHA-1 completo de 40 caracteres e recusa branch/tag mutável.
- Checkout usa fetch limitado, detached checkout e validação de `git rev-parse HEAD` antes de compilar.
- Binários e UIs são montados em staging limpo e publicados em `$PREFIX/releases/$REF`; link `$PREFIX/current` só aponta para release completa.
- Falha antes do commit remove release nova e restaura link anterior.
- README, CHANGELOG, TODO e review Phase 82 atualizados.

### Verificação real

- `bash -n scripts/install.sh`: aprovado.
- `scripts/install.sh --dry-run --skip-deps --ref b3c0fb2`: rejeitou corretamente SHA curto.
- `scripts/install.sh --dry-run --skip-deps --ref be78f14cf33c8dd6dbd903840da9ac2bde8c9ae4`: aprovado; host não alterado.

### Limitações

Node.js preflight ainda ocorre depois de instalação de pacotes e precisa ser movido antes de mutar host. Instalação real, rollback exercitado, CI novo, release, PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 continuam não validados.

---

## 2026-09-13 — Phase 82 — auditoria de segurança do instalador

**Status:** BLOCKED — findings HIGH/MEDIUM abertos; PR #41 não deve ser mergeado.

### Resultado

- CI remoto mais recente do PR #41: run `34739632116`, concluído com sucesso.
- Testes locais `make validate`: aprovados.
- Code review independente: HIGH no checkout remoto mutável sem verificação criptográfica; MEDIUM em Node.js instalado pelo gerenciador do SO e instalação parcial/reexecução de assets.
- Security review: findings confirmados como risco operacional; nenhum hardware foi alegado.

### Decisão

Não fazer merge nem release. Próxima implementação deve fixar e verificar fonte/artefato, montar árvore de instalação atômica com rollback e validar Node.js >= 20 antes de mutar host. Corrigir também cópia repetida das UIs sem usar `rm -rf` destrutivo sobre árvore ativa.

---

## 2026-09-13 — Phase 81 — hardening do instalador e concorrência CI/release

**Status:** implementação local; PR #41 aberto; CI remoto verde; não mergeado; não lançado.

### Implementado

- `scripts/install.sh --dry-run` descreve operações completas sem alterar host.
- `--ref` aceita SHA-1 completo via clone sem checkout e `checkout --detach`.
- Caminhos de instalação rejeitam caracteres que quebrariam substituições `sed`.
- CI volta a validar pushes em `main`; release usa concorrência serializada por tag.
- README, CHANGELOG, TODO e START atualizados.

### Verificação real

- `bash -n scripts/install.sh`: aprovado.
- `scripts/install.sh --dry-run --skip-deps --ref b3c0fb2`: aprovado; host não alterado.
- YAML dos workflows CI/release: parseado com sucesso.
- `git diff --check`: aprovado.
- CI PR #41: runs `34735649836` e `34735649634` concluídos com sucesso.
- Reviews independentes: sem BLOCKER/HIGH; recomendação de hardening do instalador aplicada.

### Limitações

Instalação real, release, runtime ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados. ShellCheck não disponível no host.

---

## 2026-09-13 — Phase 80 — compatibilidade TypeScript 7 no Engineer Console

**Status:** correção local; CI remoto pendente; PR #41 aberto; não mergeado; não lançado.

### Implementado

- Adicionado `web/engineer/src/vite-env.d.ts` com referência padrão `vite/client`.
- Corrigido `TS2882` no import lateral `./style.css` após atualização para TypeScript 7.
- Nenhuma alteração em código de produção, permissões ou secrets.

### Verificação real

- `npm run typecheck --prefix web/engineer`: aprovado.
- `npm run test --prefix web/engineer -- --run`: 2 testes aprovados.
- `npm run build --prefix web/engineer`: build Vite aprovado.
- `make validate`: aprovado.
- Reviews independentes: test/code aprovado; security sem BLOCKER/HIGH.

### Limitações

CI remoto `34732920670` falhou somente no typecheck Engineer; demais jobs passaram. Novo CI precisa confirmar correção. Release, runtime ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

### Phase 79 — compatibilidade OpenSSL 3.5 na geração de assinaturas

**Status:** correção local; CI remoto aguardando novo run; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Testes Ed25519 passaram a informar `-rawin` ao `openssl pkeyutl -sign`, compatível com OpenSSL 3.5.

### Verificação real

- Suíte combinada local: 56 aprovados.

### Limitações

Release, ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-13 — Phase 79 — compatibilidade OpenSSL 3.5 na geração de assinaturas

**Status:** correção local; CI remoto aguardando novo run; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Testes Ed25519 passaram a informar `-rawin` ao `openssl pkeyutl -sign`, compatível com OpenSSL 3.5.
- Corrige falha do run `34728518286`, onde 10 de 56 testes falharam no helper de assinatura antes de exercitar os validadores.

### Verificação real

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py tests/test_validate_release_bundle.py`: 56 aprovados.
- `python3 -m py_compile tests/test_verify_release_signature.py tests/test_validate_release_bundle.py`: aprovado.
- `git diff --check`: aprovado.

### Limitações

CI ainda não confirmou esta correção. Release, ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-13 — Phase 78 — rejeição de dados residuais em archives

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Validador verifica stream gzip completo depois de validar e consumir TAR.
- Bytes residuais e streams gzip concatenados falham fechado.
- Testes offline cobrem ambos os casos.
- README, CHANGELOG, TODO, START e review da Phase 78 atualizados.

### Verificação real

- `python3 -m pytest -q tests/test_validate_release_archive.py`: 24 aprovados.
- `py_compile` e `git diff --check`: aprovados.

### Limitações

CI remoto continua falhando antes dos steps: runs `34727191187` (PR) e `34727189605` (push). Release, ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-12 — Phase 77 — validação estrutural antes do payload

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Validador rejeita nomes, raiz, tipos, allowlist, duplicatas e membros inesperados antes de consumir payloads.
- Arquivos aceitos continuam consumidos integralmente em chunks limitados; payload truncado falha fechado.
- Teste confirma que membro inesperado não chama consumidor de payload.
- README, CHANGELOG, TODO, START e review da Phase 77 atualizados.

### Limitações

CI remoto continua bloqueado por runner/permissões: runs consultados falharam antes dos steps com `runner_id=0` e `steps=[]`. Release, ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-12 — Phase 76 CI status refresh

**Status:** PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Verificação real

- Push run `34724842446` e PR run `34724845040` falharam em aproximadamente 3–4 segundos, antes de qualquer step executável.
- Jobs consultados retornaram `runner_id=0` e `steps=[]`; nenhum teste ou build remoto executou.
- Working tree permaneceu limpo antes desta atualização documental.

### Decisão

Não fazer merge ou release. Causa continua classificada como `RUNNER / PLATFORM / CONFIGURATION FAILURE`; gates locais permanecem aprovados; hardware Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-12 — Phase 76 CI status refresh

**Status:** commit `57ec4db` publicado; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Verificação real

- Push run `34724816608` e PR run `34724820149` falharam em aproximadamente 3–4 segundos, antes de qualquer step executável.
- Nenhum teste ou build remoto executou. Causa continua classificada como `RUNNER / PLATFORM / CONFIGURATION FAILURE`.
- Working tree ficou limpo após remover caches Python gerados.

### Decisão

Não fazer merge ou release. Gates locais permanecem aprovados; CI remoto, release, hardware Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-12 — Phase 76

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Validador de archive consome payload completo de arquivos regulares em chunks de até 1 MiB após validar headers.
- Payload truncado ou ausente falha fechado; `tarfile` limita leitura ao tamanho declarado pelo header.
- Teste offline cobre archive gzip truncado durante a leitura.
- README, CHANGELOG, TODO e review da Phase 76 atualizados.

### Limitações

CI remoto continua bloqueado por runner/permissões: run `34723511315` falhou antes dos steps, com jobs sem steps executados. Nenhum teste/build remoto executou. Release, ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-12 — Phase 76 — leitura integral de payloads de archive

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Validador de archive consome payload completo de arquivos regulares em chunks de até 1 MiB após validar headers.
- Payload truncado ou ausente falha fechado; `tarfile` limita leitura ao tamanho declarado pelo header.
- Teste offline cobre archive gzip truncado durante a leitura.
- README, CHANGELOG, TODO e review da Phase 76 atualizados.

### Limitações

CI remoto continua bloqueado por runner/permissões: run `34723511315` falhou antes dos steps, com jobs sem steps executados. Nenhum teste/build remoto executou. Release, ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-12 — Phase 75 — fonte única de versão e gate reproduzível

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- `VERSION` virou fonte canônica `0.3.1`.
- `scripts/validate-version.py` valida SemVer, manifests Rust/frontend e tag de release.
- `release.yml` deixou de extrair versão com `grep/head/sed`; repete gate `tomllib` inline, fail-closed e sem executar script mutável do checkout antes da validação.
- `make test` inclui testes do gate de versão.
- README, CHANGELOG, TODO e review da Phase 75 atualizados.

### Limitações

CI remoto continua bloqueado por runner/permissões: último run `34721822744` teve 10 jobs com `runner_id=0` e `steps=[]`. Nenhum teste/build remoto executou. ARM64, Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC, runtime Windows e release continuam não validados.

---

## 2026-09-12 — Phase 74 — documentação visual das interfaces

**Status:** documentação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Adicionado `docs/INTERFACES.md` com Login, Musician PWA, Engineer Console, Admin CLI/API, controles de mix e estado real de validação.
- Gerados SVG/PNG documentais a partir de `scripts/generate-ui-doc-images.py`.
- README, CHANGELOG, guia do músico e guia Windows + Docker Desktop referenciam imagens e deixam explícito que são mockups, não screenshots de runtime.

### Verificação real

- `make validate`: aprovado; Rust, TypeScript, testes, documentação/PDF e skills passaram.
- `python3 scripts/generate-ui-doc-images.py`: cinco SVG e PNG gerados.
- `git diff --check` e `py_compile`: aprovados.

### Limitações

Imagens não provam execução da UI. CI remoto, release, mídia WebRTC, PipeWire/ALSA, Raspberry Pi 5 e runtime Windows continuam não validados.

---

## 2026-09-12 — Phase 73 — Guia do Músico alinhado ao signaling HTTP

**Status:** documentação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Guia documenta sequência real de `POST /api/v1/audio/offer` e `POST /api/v1/audio/ice-candidate`.
- Guia distingue control plane/Sans-IO validado de mídia `SIMULATED`.
- PDF será regenerado e validado pelo gate documental.

### Limitações

CI remoto, release, mídia WebRTC, PipeWire/ALSA, Raspberry Pi 5 e runtime Windows continuam não validados.

---

## 2026-09-12 — Phase 72 — integração HTTP de signaling

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Teste de integração autentica Musician e executa negociação SDP pela rota HTTP.
- O mesmo teste envia candidato trickle ICE pela rota HTTP após oferta aceita.
- README, CHANGELOG, TODO e review da Phase 72 atualizados.

### Verificação real

- `cargo test -p api-server --test integration musician_negotiates_offer_and_trickles_ice_candidate_over_http`: 1 aprovado; 57 filtrados.

### Limitações

CI remoto continua bloqueado por runner/permissões: run `34719135360` permanece `queued`; job Python terminou `failure` sem steps e demais jobs seguem sem execução. Fluxo valida somente control plane HTTP e Sans-IO; mídia WebRTC, PipeWire/ALSA, Raspberry Pi 5, runtime Windows e release continuam não validados.

---

## 2026-09-12 — Phase 71 — snapshot validado para publicação

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Validador abre bundle e entradas via descritores `O_DIRECTORY|O_NOFOLLOW` e `fstat()`, reduzindo races de caminho durante captura.
- Bundle limitado a 32 entradas.
- Snapshot usa cópia em chunks de 1 MiB e staging privado; output só aparece após validação completa.
- Workflow calcula checksums e lista de upload somente sobre `release-upload`, sem recópia de `dist` após validação.

### Verificação real

- 52 testes dos validadores aprovados.
- `py_compile` e `git diff --check` aprovados.
- Scan estático do código adicionado: sem hits para hardcoded secrets, shell injection, eval/exec, pickle ou SQL dinâmico.

### Limitações

CI remoto continua bloqueado por runner/permissões. Nenhum release, hardware Raspberry Pi 5, PipeWire/ALSA, WebRTC ou runtime Windows foi validado.

---

## 2026-09-12 — Phase 70 — hardening do validador de bundle

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Leituras do bundle usam `O_NOFOLLOW`, `O_NONBLOCK`, limite individual de 512 MiB e limite total de 2 GiB.
- SBOM opcional é validado como JSON objeto antes de entrar no manifesto.
- Escrita do manifesto rejeita symlink, exige arquivo regular e limita conteúdo a 64 KiB.
- Testes cobrem SBOM inválido, manifesto symlink e limite de arquivo.

### Verificação real

- Suíte dos validadores: 45 testes aprovados em `/tmp/open-iem-venv` com `pytest==9.1.1`.
- `make validate`: aprovado; `py_compile` e `git diff --check`: aprovados.

### Limitações

CI remoto continua bloqueado por runner/permissões. TOCTOU entre validação e cópia no workflow exige hardening posterior. Raspberry Pi 5, PipeWire/ALSA, WebRTC, Windows runtime e release continuam não validados.

---

## 2026-09-12 — Phase 69 follow-up — novo bloqueio pré-step do CI

**Status:** commit local de atualização documental; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Verificação real

- Local: `make validate` aprovado; 42 testes dos validadores aprovados; `py_compile` e `git diff --check` aprovados.
- PR run `34711659175` e push run `34711656770` falharam antes da execução dos jobs; os 10 jobs do PR retornaram `runner_id=0` e `steps=[]`; nenhum teste ou build remoto executou.
- Consulta detalhada de checks permanece limitada por HTTP 403 do token atual.

### Decisão

Não fazer merge ou release. O bloqueio continua em runner/permissões do GitHub Actions. Hardware Raspberry Pi 5, PipeWire/ALSA, WebRTC, runtime Windows e publicação de release continuam não validados.

---

## 2026-09-12 — Phase 69 follow-up — resultado CI após hardening do upload

**Status:** commit `13ee341` publicado; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Verificação real

- Local: 42 testes dos validadores aprovados; `py_compile`, parse YAML e `git diff --check` aprovados.
- Push run `34711045056` falhou antes dos steps: jobs terminaram sem executar steps (`steps=[]`); consulta de annotations retornou HTTP 403 para o token atual.

### Decisão

Não fazer merge ou release. O bloqueio continua em runner/permissões do GitHub Actions. Hardware Raspberry Pi 5, PipeWire/ALSA, WebRTC, runtime Windows e publicação de release continuam não validados.

---

## 2026-09-12 — Phase 69 follow-up — novo bloqueio pré-step do CI

**Status:** commit `d1d2530` publicado; PR #40 aberto; CI remoto continua bloqueado; não mergeado; não lançado.

### Verificação real

- Push/PR runs `34711621971` e `34711619779` falharam antes dos steps.
- No run PR `34711621971`, os 10 jobs retornaram `runner_id=0` e `steps=[]`; nenhum teste ou build remoto executou.
- Consulta detalhada de checks permanece limitada por HTTP 403 do token atual.

### Decisão

Não fazer merge ou release. O bloqueio é de runner/permissões do GitHub Actions, não falha reproduzida no código. Gates locais seguem aprovados; Raspberry Pi 5, PipeWire/ALSA, WebRTC, runtime Windows e publicação de release continuam não validados.

---

## 2026-09-12 — Phase 69 follow-up — cobertura do validador de bundle no CI

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Job `python-security-tests` agora executa `tests/test_validate_release_bundle.py` junto aos validadores de archive e assinatura.
- Jobs de build e validação final exigem `OPENIEM_RELEASE_SIGNING_PUBLIC_KEY_FINGERPRINT`, validam formato estrito de 64 hex e comparam fingerprint SHA-256 da chave pública derivada antes da verificação.
- Validação gera manifesto temporário com nomes e digests exatos; publicação usa lista explícita derivada desse manifesto, sem glob amplo.
- README, CHANGELOG, TODO e review da Phase 69 atualizados.

### Verificação

- Suíte local dos validadores: 42 testes aprovados.
- Manifesto é rechecado com `sha256sum -c` antes da publicação.
- `make validate`: aprovado pelo agente de testes independente.
- `git diff --check`: aprovado.

### Limitações

CI remoto continua falhando antes dos steps por runner/permissões. A validação de assinatura no bundle permanece estrutural; verificação criptográfica no workflow depende de chave pública confiável provisionada por canal independente. Raspberry Pi 5, PipeWire/ALSA, WebRTC, Windows runtime e release continuam não validados.

---

## 2026-09-12 — Phase 69 — validação do bundle final de release

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Novo `scripts/validate-release-bundle.py` valida conjunto final após download dos artefatos.
- Exige archives server x86_64/ARM64 e web musician/engineer na versão da tag.
- Recalcula checksums com descritores `O_NOFOLLOW`, exige assinaturas server não vazias e rejeita arquivos inesperados, symlinks ou não regulares.
- Workflow executa gate antes de `Create GitHub Release`.
- README, CHANGELOG, TODO e review da Phase 69 atualizados.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_bundle.py tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 39 testes.
- `python3 -m py_compile scripts/validate-release-bundle.py`: PASS.
- Parse YAML local do workflow: PASS.
- `git diff --check`: PASS.

### Limitações

CI remoto falha antes dos steps por runner/permissões. Nenhuma release foi publicada. Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC e runtime Windows continuam não validados.

---

---

## 2026-09-12 — Phase 68 — nomes Windows e limites de assinatura

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Validador rejeita caracteres inválidos, pontos/espaços finais e nomes reservados Windows em todos os componentes de archive.
- Verificador detached rejeita assinatura e chave pública acima de 64 KiB antes de executar OpenSSL.
- Testes offline cobrem nomes Windows e entradas oversized.
- README, CHANGELOG, TODO e review da Phase 68 atualizados.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 30 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Review independente: risco DoS identificado e correção aplicada.

### Limitações

CI remoto falha antes dos steps por runner/permissões. Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC, Windows runtime e release continuam não validados.

---

## 2026-09-12 — Phase 67 — nomes seguros entre plataformas

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Validador rejeita separador `\\` e caracteres de controle em nomes de archive antes da validação estrutural.
- Testes offline cobrem separador Windows e newline; mensagens de nomes de controle usam representação segura.
- README, CHANGELOG, TODO e review da Phase 67 atualizados.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 26 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Review independente: PASS, sem concerns de segurança ou erros lógicos de alta confiança.

### Limitações

CI remoto falha antes dos steps por runner/permissões. Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC e release continuam não validados.

---


## 2026-09-12 — Phase 66 — raiz canônica em archives

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Validador rejeita raiz `.`/`..` e nomes de raiz não canônicos antes de validar conteúdo.
- Adicionado teste offline para raiz não canônica.
- README, CHANGELOG, TODO e review da Phase 66 atualizados.

### Verificação

- Review independente: PASS, sem concerns de segurança ou erros lógicos de alta confiança.
- Testes Python e `make validate`: executar no gate final.

### Limitações

CI remoto falha antes dos steps por runner/permissões. Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC e release continuam não validados.

---

## 2026-09-12 — Phase 65 — limites incrementais durante leitura de archives

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Validador aplica limite por membro e limite total descomprimido durante o primeiro loop do `tarfile`.
- Rejeição de entrada oversized não consome membros posteriores.
- Adicionado teste offline para confirmar falha imediata.
- README, CHANGELOG, TODO e review da Phase 65 atualizados.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 23 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- `make validate`: PASS — Rust, frontends, documentação, PDF e skills.
- Review independente: PASS, sem concerns de segurança ou erros lógicos de alta confiança; recomendação de validação incremental aplicada.

### Limitações

CI remoto falha antes dos steps por runner/permissões. Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC e release continuam não validados.

---

---

## 2026-09-12 — Phase 64 — tratamento fail-closed da portabilidade do validador

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- CLI de `scripts/validate-release-archive.py` captura `RuntimeError` de plataforma sem `O_NOFOLLOW` e retorna código 1 controlado.
- Evita traceback e mantém validação fail-closed em plataformas sem a proteção necessária.
- README, CHANGELOG e TODO atualizados.

### Verificação

- Testes Python dos validadores: PASS — 22 testes.
- `make validate`: PASS.
- Review independente pendente nesta rodada.

### Limitações

CI remoto falha antes dos steps por runner/permissões. Último PR run: `34698206761`; jobs retornaram `steps=[]`. Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC e release continuam não validados.

---

---

## 2026-09-12 — CI status after Phase 63 archive verifier portability

**Status:** commit `c535eac` publicado; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Verificação real

- Run push `34695497246` e PR run `34695498591` terminaram `failure` em todos os 10 jobs.
- Todos os jobs retornaram `steps=[]`, falhando antes da execução do runner.
- Gates locais no commit passaram: `make validate`, 22 testes Python dos validadores e `git diff --check`.

### Decisão

Não fazer merge, release ou alegação de CI verde. Bloqueio segue em GitHub Actions/runner/permissões; Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---


## 2026-09-12 — Phase 64 — tratamento fail-closed da portabilidade do validador

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- CLI de `scripts/validate-release-archive.py` captura `RuntimeError` de plataforma sem `O_NOFOLLOW` e retorna código 1 controlado.
- Evita traceback e mantém validação fail-closed em plataformas sem a proteção necessária.
- README, CHANGELOG e TODO atualizados.

### Verificação

- Testes Python dos validadores: PASS — 22 testes.
- `make validate`: PASS.
- Review independente pendente nesta rodada.

### Limitações

CI remoto falha antes dos steps por runner/permissões. Último PR run: `34698206761`; jobs retornaram `steps=[]`. Raspberry Pi 5, PipeWire/ALSA, mídia WebRTC e release continuam não validados.

---

## 2026-09-12 — CI status after Phase 63 archive verifier portability

**Status:** commit `c535eac` publicado; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Verificação real

- Run push `34695497246` e PR `34695498591` terminaram `failure` em todos os 10 jobs.
- Todos os jobs retornaram `steps=[]`, falhando antes da execução do runner.
- Gates locais no commit passaram: `make validate`, 22 testes Python dos validadores e `git diff --check`.

### Decisão

Não fazer merge, release ou alegação de CI verde. Bloqueio segue em GitHub Actions/runner/permissões; Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---


---

## 2026-09-12 — Phase 63 archive verifier portability — fechamento do fallback permissivo

**Status:** correção local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Validador de archives agora exige suporte explícito a `O_NOFOLLOW`; ausência da proteção falha fechado em vez de usar flag zero.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 21 testes.
- `make validate`: PASS — Rust, frontends, documentação, PDF e skills.
- `git diff --check`: PASS.
- Review independente: PASS, sem concerns de segurança ou erros lógicos.

### Limitações

CI remoto continua bloqueado por falha pré-execução do runner. Release, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

---

## 2026-09-12 — Phase 63 follow-up — fechamento de gaps fail-closed

**Status:** correção local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Removido fallback que tratava ausência de `O_NOFOLLOW` como zero; plataforma sem essa proteção falha explicitamente.
- Adicionado `O_NONBLOCK` antes da validação `fstat()`, evitando bloqueio em FIFO ou dispositivo especial controlado por atacante.
- OpenSSL agora é chamado por `/usr/bin/openssl`, eliminando substituição via `PATH`.

### Verificação

- Review independente pós-correção: PASS, sem concerns de segurança ou erros lógicos.
- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 22 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `make validate`: PASS — Rust, frontends, documentação, PDF e skills.
- `git diff --check`: PASS.

### Limitações

Execução validada permanece Linux com `/usr/bin/openssl`; CI remoto continua bloqueado: run `34694228445` falhou em todos os 10 jobs às 12:38:36Z, com `steps=[]` e falha pré-execução. Release, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

### Próximo passo

Executar gates locais finais, publicar branch e revalidar CI remoto. Não fazer merge ou release sem runner executável e gates reais.

---

## 2026-09-12 — Phase 63 — verificação fail-closed de arquivos assinados

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Verificador Ed25519 abre artifact, assinatura e chave com `O_NOFOLLOW` e mantém descritores estáveis durante OpenSSL.
- Testes offline rejeitam symlink em artifact e assinatura.
- Erro de execução do OpenSSL falha explicitamente, sem acessar resultado não inicializado.
- Atualizados README, START, CHANGELOG, TODO e review da Phase 63.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 21 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Review independente encontrou e corrigiu falha no caminho de `OSError` do subprocesso.

### Limitações

Scanner `/root/scan_patterns.py` não disponível neste ambiente. CI remoto continua falhando antes dos steps. Release, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

### Próximo passo

Executar gates locais finais, publicar branch e revalidar CI remoto. Não fazer merge ou release sem runner executável e gates reais.

---

---

## 2026-09-12 — Phase 62 — testes de segurança Python no CI

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Adicionado job `python-security-tests` ao workflow CI.
- Job configura Python, instala `pytest` e executa testes offline de validação de archive e assinatura.
- Atualizados README, CHANGELOG, TODO e review da Phase 62.

### Limitações

Runs GitHub Actions continuam falhando antes dos steps por runner/permissões. Release, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

### Próximo passo

Executar gates locais finais, publicar branch e revalidar CI remoto. Não fazer merge ou release sem runner executável e gates reais.

---

## 2026-09-12 — Phase 61 — abertura fail-closed de archives

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Validador abre archive com `O_NOFOLLOW` e `O_CLOEXEC`, exige arquivo regular e mede tamanho via `fstat()` no mesmo descritor entregue ao `tarfile`.
- Testes offline cobrem symlink e diretório como entradas rejeitadas.
- README, CHANGELOG, TODO e review da Phase 61 atualizados.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 19 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Review independente recomendou descritor com `O_NOFOLLOW` e `fstat()` para evitar TOCTOU; implementação aplicada.

### Limitações

CI remoto continua falhando antes dos steps no run `34685058398` e no push correspondente. Secret/fingerprint Ed25519, release, Caddy, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

### Próximo passo

Executar gates locais finais, publicar branch e revalidar CI remoto. Não fazer merge ou release sem runner executável e gates reais.

---

## 2026-09-12 — Phase 60 — validação incremental de archives

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Validador itera membros do `tarfile` e interrompe imediatamente acima de `MAX_MEMBERS`, evitando materialização ilimitada de headers.
- Testes offline cobrem `MAX_ARCHIVE_BYTES` e `MAX_UNCOMPRESSED_BYTES`, além dos limites já existentes.
- README, CHANGELOG, TODO e review da Phase 60 atualizados.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 17 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `make validate`: PASS — Rust (incluindo integração), frontends, PDF, documentação e skills.
- `git diff --check`: PASS.
- Reviews independentes identificaram e corrigiram risco MEDIUM de consumo de memória/CPU via `getmembers()`.

### Limitações

CI remoto continua falhando antes dos steps nos runs `34674883204`, `34674880716`, `34674858746` e `34674856987`. Secret/fingerprint Ed25519, release, Caddy, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

### Próximo passo

Executar gates locais finais, publicar branch e revalidar CI remoto. Não fazer merge ou release sem runner executável e gates reais.

---

## 2026-09-12 — Phase 59 — limites de recursos no validador de archives

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Validador rejeita archives acima de 512 MiB comprimidos, mais de 32 membros, membro acima de 256 MiB ou total descomprimido acima de 512 MiB.
- Testes offline cobrem limites de contagem e tamanho.
- README, CHANGELOG, TODO e review da Phase 59 atualizados.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py`: PASS — 15 testes.
- `python3 -m py_compile scripts/validate-release-archive.py scripts/verify-release-signature.py`: PASS.
- `git diff --check`: PASS.
- Reviews independentes: sem blocker; recomendação de limites de recursos implementada.

### Limitações

CI remoto continua falhando antes dos steps nos runs `34669494781` e `34669493264`. Secret/fingerprint Ed25519, release, Caddy, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

### Próximo passo

Desbloquear GitHub Actions; executar CI real. Depois provisionar chave pública autenticada e validar instalação/áudio em Raspberry Pi 5.

### CI após push

- Push run `34674856987` e PR run `34674858746` falharam antes dos steps; os 9 jobs de cada run terminaram com `steps=[]`.
- O bloqueio continua sendo runner/permissão GitHub Actions; não representa falha executada nos testes do commit.

---

## 2026-09-12 — Phase 58 — assinatura Ed25519 no workflow

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Workflow de release assina archives x86_64 e ARM64 com secret externo `OPENIEM_RELEASE_SIGNING_KEY_PEM`.
- Chave privada usa arquivo temporário, `trap` de limpeza, validação explícita de presença/tipo Ed25519 e saída não vazia.
- `.sig` é transferido entre jobs e publicado junto com archive/checksum na GitHub Release.
- README, CHANGELOG, TODO e review da Phase 58 atualizados.

### Verificação

- `make validate`: PASS — Rust, frontends, documentação, PDF e skills.
- `python3 -m pytest -q tests/test_verify_release_signature.py tests/test_validate_release_archive.py`: PASS — 12 testes.
- Reviews independentes de segurança, código e testes encontraram gaps de publicação/limpeza; correções aplicadas.
- CI remoto continua falhando antes dos steps; não há execução remota validada.

### Limitações

Secret ainda precisa ser provisionado no GitHub e fingerprint/chave pública distribuídos por canal independente. Raspberry Pi 5, Caddy, runtime ARM64, PipeWire/ALSA e mídia WebRTC não foram validados.

### Próximo passo

Provisionar secret Ed25519 e chave pública autenticada; desbloquear runner; executar CI real antes de merge/release.

---

## 2026-09-12 — Phase 58 — verificação local de assinatura Ed25519

**Status:** implementação local; CI remoto bloqueado; não mergeado; não lançado.

### Implementado

- Criado `scripts/verify-release-signature.py` para verificar assinatura detached Ed25519 via OpenSSL.
- Instalador Raspberry Pi baixa e verifica `.sig` antes de checksum, validação estrutural e extração.
- Chave pública precisa ser arquivo regular em `/etc/openiem/release-signing-key.pem`, distribuída por canal independente.
- Adicionados testes offline e review da fase.

### Limitações

Workflow agora produz `.sig`; gestão do secret, fingerprint e distribuição autenticada da chave pública continuam pendentes. CI remoto segue falhando antes dos steps (`runnerId: null`). Raspberry Pi 5, Caddy, runtime ARM64, PipeWire/ALSA e mídia WebRTC não foram validados.

### Próximo passo

Provisionar secret Ed25519, publicar fingerprint por canal independente e executar CI real. Não fazer merge ou release sem esses gates.

---

## 2026-09-11 — Phase 57 — validação de archive e origem HTTPS

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Instalador ARM64 agora chama `scripts/validate-release-archive.py` antes da extração; validação cobre caminhos canônicos, diretório raiz único, allowlist, duplicatas, links e arquivos especiais.
- Unit systemd define `OPENIEM_ALLOWED_ORIGINS=https://iem.local`, alinhando validação de Origin ao endpoint TLS do Caddy.
- Guia documenta atualização coordenada para LAN IP quando mDNS não estiver disponível.
- README, CHANGELOG, TODO e review desta fase atualizados.

### Verificação

- `make validate`: PASS — Rust, frontends, documentação, PDF, skills e diff check.
- `cargo audit --file server/Cargo.lock --ignore RUSTSEC-2023-0071`: PASS — 0 vulnerabilidades.
- `npm audit --audit-level=high` nos dois frontends: PASS.
- Reviews independentes: uma aprovação; uma encontrou e corrigiu inconsistência de Phase 56 no README e gaps de archive/origin.

### Limitações

CI remoto continua falhando antes dos steps (`runner_id=0`); permissões API de Actions/runners retornam HTTP 403. Caddy, instalação privilegiada, runtime ARM64, PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 continuam não validados.

### Próximo passo

Executar gates locais finais, push e revalidar CI remoto. Não fazer merge ou release sem runner executável e gates reais.

---

## 2026-09-11 — CI status refresh pós-Phase 56

**Status:** documentação atualizada; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Verificação real

- Run PR `34656659602` e push `34656657286` falharam antes dos steps.
- Os 9 jobs do PR terminaram com `steps=[]`; não houve execução de runner nem resultado de teste remoto.
- API de runners permanece inacessível ao token atual com HTTP 403.
- `make test`: PASS — Rust, frontends e harness `SIMULATED`.
- `make validate`: PASS — Rust, frontends, documentação, PDF e skills.
- `git diff --check`: PASS antes desta atualização documental.

### Decisão

Não fazer merge, release ou alegação de CI verde. Próximo bloqueio é GitHub Actions/runner; depois executar CI real e validar attestation, instalação ARM64 e Raspberry Pi 5.

---

## 2026-09-11 — Phase 56 — correção do lifetime do diretório temporário do Caddyfile

**Status:** implementação local; CI remoto bloqueado; PR #40 aberto; não mergeado; não lançado.

### Implementado

- Removida limpeza antecipada de `CERT_WORK_DIR` no guia Raspberry Pi.
- Cópia segura do Caddyfile agora ocorre antes da limpeza do diretório temporário.
- Atualizados README, START, CHANGELOG, TODO e review da Phase 56.

### Verificação

- `make test`: PASS.
- `make validate`: PASS.
- `git diff --check` e `bash -n` do bloco Bash: PASS.
- Dois reviews independentes: PASS, sem findings de segurança ou lógica.

### Limitações

Caddy, instalação real, runtime ARM64, PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 continuam não validados. GitHub Actions falha antes dos steps com `runner_id=0`.

### Próximo passo

Desbloquear runner/permissões Actions; executar CI real. Depois validar instalação e áudio no Raspberry Pi 5.

---

## 2026-09-11 — Phase 55 — hardening de caminho no deployment

**Status:** implementação local; commit `71d931e`; PR #40 aberto; CI remoto bloqueado; não mergeado; não lançado.

### CI remoto pós-push

Run `34654997321` falhou antes dos steps em todos os 9 jobs; `runner_id=0` e `steps=[]`. O bloqueio é infraestrutura/permissão do GitHub Actions, não falha de teste do código.

### Implementado

- Guia Raspberry Pi resolve `deployment/caddy/Caddyfile` a partir de `REPO_ROOT`, em vez de depender do diretório corrente.
- Guia rejeita symlink e instala Caddyfile com `sudo install`, ownership root e modo `0644`.
- README registra Phase 54 na tabela incremental e marca Phase 53 como validação local concluída.
- TODO registra Phase 55 e os runs remotos mais recentes.

### Verificação

- `make test`: PASS — Rust, frontends e harness `SIMULATED`.
- `make validate`: PASS — documentação, PDF e skills.
- `cargo fmt`, `cargo clippy`, frontend typecheck e `git diff --check`: PASS.
- Review independente: sem finding de segurança no diff; encontrada e corrigida inconsistência da tabela de fases; gap relativo a caminho do Caddyfile corrigido.

### Limitações

`caddy validate`, instalação e runtime ARM64 não foram executados neste VPS. GitHub Actions continua falhando antes dos steps (`runner_id=0`, `steps=[]`) nos runs `34652471047` e `34652466951`. Nenhum release, attestation remota ou hardware Raspberry Pi foi validado.

### Próximo passo

Desbloquear runner/permissões Actions; executar CI e verificar attestation. Depois validar instalação e áudio no Raspberry Pi 5.

---

## 2026-09-11 — Phase 54 — provenance de artefatos de release

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado; não mergeado; não lançado.

### Implementado

- Added attestation Sigstore/GitHub para archives de servidor x86_64 e ARM64 antes do upload.
- Fixada `actions/attest-build-provenance@v2` por SHA completo.
- Permissões de build limitadas a `contents: read`, `id-token: write` e `attestations: write`.
- Atualizados README, START, CHANGELOG, TODO e review da Phase 54.

### Verificação

- `python3 -m pytest -q tests/test_validate_release_archive.py`: PASS — 7 testes.
- `make test`: PASS — Rust, frontends e harness `SIMULATED`.
- `make validate`: PASS — documentação, PDF e skills.
- `cargo fmt`, `cargo clippy` e `git diff --check`: PASS.

### Limitações

GitHub Actions continua falhando antes dos steps com `runner_id=0`; attestation remota ainda não foi publicada nem verificada. Nenhum release foi publicado. Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC permanecem não validados.

### Próximo passo

Desbloquear runner; executar CI; verificar attestation dos archives antes de merge/release.

---

## 2026-09-11 — Phase 53 CI status refresh

**Status:** implementação local; PR #40 aberto; não mergeado; não lançado.

### Verificação

- GitHub run `34645508776` (PR) e push `34645504292` falharam antes dos steps.
- Todos os 9 jobs do PR terminaram com `runner_id=0` e `steps=[]`; incidente classificado como `RUNNER / PLATFORM / CONFIGURATION FAILURE`.
- `make test`: PASS — Rust, frontends e harness `SIMULATED`.
- `make validate`: PASS — documentação, PDF e skills.
- `cargo fmt --all -- --check` e `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `npm run lint`: indisponível nos dois frontends; scripts `lint` não existem. `npm audit` não foi executado por causa do loop fail-fast.
- Scanner `/root/scan_patterns.py` não existe neste ambiente; nenhum resultado foi inventado.

### Documentação

README, CHANGELOG e este log atualizados com estado real dos runs.

### Próximo passo

Desbloquear runner/permissões Actions. Não fazer merge ou release antes de CI remoto executar gates reais.

---

## 2026-09-11 — Phase 53 follow-up — gates de release fail-closed

**Status:** implementação local; PR #40 aberto; CI remoto falha pré-steps; não mergeado; não lançado.

### Implementado

- Validador de archive rejeita basenames obrigatórios fora da allowlist.
- Teste CLI adicionado; suíte do validador agora tem 7 testes.
- Empacotamento web falha quando `dist/` de Musician ou Engineer falta.
- Upload web usa `if-no-files-found: error`.
- Guia ARM64 exige e instala `api-server` e `open-iem-admin`.

### Verificação

- `make test`: PASS — Rust, frontends e harness SIMULATED.
- `make validate`: PASS — documentação, PDF e skills.
- Reviews independentes: findings de gate corrigidos localmente; assinatura independente, CI remoto e hardware seguem pendentes.

### Próximo passo

Executar re-review após atualização documental; não fazer merge ou release até CI remoto executar gates reais.

---

## 2026-09-11 — Phase 53 — validação estrutural de archives de release

**Status:** commit `15cc584`; PR #40 aberto; CI remoto falhou pré-steps no run `34642754837`; não mergeado; não lançado.

### Implementado

- Criado `scripts/validate-release-archive.py` para validar archives de servidor antes do upload.
- Archives devem conter um diretório raiz único, somente arquivos regulares allowlisted e binários `api-server`/`open-iem-admin`.
- Traversal, caminhos absolutos, links, arquivos especiais, membros inesperados e binários ausentes são rejeitados.
- Workflow de release executa validador em artefatos x86_64 e ARM64 antes de gerar checksum.
- Testes offline cobrem archive válido, traversal, symlink e membro inesperado.

### Verificação

- Teste dedicado do validador: PASS — 5 testes (`python3 -m pytest -q tests/test_validate_release_archive.py`).
- CI remoto segue falhando antes dos steps, com `runner_id=0`; hardware ARM64/PipeWire continua não validado.

### Limitações

Validação estrutural não autentica origem. Assinatura independente, execução em Raspberry Pi 5 e CI remoto continuam pendentes.

### Próximo passo

Executar gates locais completos; depois revisar diff com agentes independentes. Não fazer merge/release até CI real executar.

---

## 2026-09-11 — Phase 52 — follow-up de verificação independente

**Status:** correção local; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

### Implementado

- Admin CLI desativa redirects do `reqwest`, preservando a política HTTPS/loopback e evitando downgrade com Bearer token.
- `cargo-audit` fixado em `0.22.2` nos workflows CI e release.
- Certificados LAN gerados em diretório temporário; instalação usa `sudo install` com ownership explícito e chave privada `0640` para o grupo `caddy`.
- Corrigidas inconsistências documentais de `make fmt` e da tabela de fases do README.

### Verificação

- `make validate`: PASS — Rust, frontends, documentação, PDF, skills e diff check.
- Reviews independentes: test-master PASS; security-review encontrou e confirmou correções acima; code-review encontrou inconsistências documentais, corrigidas.
- CI remoto continua falhando pré-steps com `runner_id=0`; não há hardware ARM64/PipeWire validado.

### Limitações

Arquivo de checksum continua sem assinatura independente; validação robusta de archives maliciosos e execução em Raspberry Pi 5 permanecem pendentes. Nenhum release foi publicado.

---

## 2026-09-11 — Phase 51 — hardening do fluxo de deployment ARM64

**Status:** correção local; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

### Implementado

- Downloads do instalador restringem redirects para HTTPS.
- Diretórios `/etc/openiem`, `/etc/openiem/keys`, `/var/lib/openiem` e `/opt/openiem` recebem ownership e modos explícitos; `/etc/openiem` e `/etc/openiem/keys` usam grupo `openiem` para permitir acesso do serviço.
- Chaves JWT são geradas em diretório temporário limpo por `trap` e instaladas via `sudo install` com modos restritos.
- Unit systemd é obtida de clone Git confiável, validada como arquivo regular por `systemd-analyze verify` e instalada com ownership root.

### Verificação

- Extração dos blocos Bash e `bash -n`: PASS.
- `git diff --check`: PASS.
- `make test`, validações de documentação/PDF/skills e reviews independentes: executar antes do commit.

### Limitações

CI remoto segue falhando antes dos steps com `runner_id=0`; últimos runs observados `34634194257` (PR) e `34634188913` (push). Nenhum release foi publicado. Instalação, runtime ARM64, PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 permanecem não validados.

---

## 2026-09-11 — Phase 50 — hardening do instalador ARM64

**Status:** correção local; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

### Implementado

- Instalador valida tag `vX.Y.Z` sem zeros à esquerda, usa `set -euo pipefail` e baixa arquivos para diretório temporário exclusivo.
- Falhas HTTP, checksum inválido e arquivo ausente interrompem fluxo antes de instalação privilegiada.
- Membros de archive com caminho absoluto, traversal, symlink ou hard link são rejeitados antes da extração.
- `api-server` precisa existir como arquivo regular no diretório esperado antes de `sudo install`.

### Verificação

- `make test`: PASS — Rust, frontend e harness de áudio determinístico.
- `bash scripts/validate-docs.sh`, `bash scripts/validate-pdf.sh`, `bash scripts/validate-skills.sh`, `bash -n` do bloco de instalação e `git diff --check`: PASS.
- Review independente encontrou gaps de automação e riscos no fluxo anterior; correção aplicada e revalidação local executada.

### Limitações

Checksum continua sendo baixado do mesmo release HTTPS e não substitui assinatura/autenticidade independente. Nenhum release foi publicado. CI remoto segue falhando antes dos steps com `runner_id=0`; instalação, runtime ARM64, PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 permanecem não validados.

---

## 2026-09-11 — Phase 49 — correção do guia de instalação ARM64

**Status:** documentação corrigida localmente; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

### Implementado

- Alinhado `deployment/raspberry-pi/README.md` ao nome real gerado por `.github/workflows/release.yml`: `open-iem-server-<versão>-aarch64-linux.tar.gz`.
- Corrigido caminho do binário `api-server` após extração do diretório versionado.
- O guia agora baixa o checksum publicado, usa `sha256sum --check` e aborta antes da extração se integridade falhar.
- `curl` usa modo fail-closed para erros HTTP.
- Atualizados CHANGELOG, TODO e review da Phase 49.

### Verificação

- Validação textual confirmou que URL, arquivo baixado, checksum, arquivo extraído e caminho instalado usam o mesmo nome de artefato.
- `bash scripts/validate-docs.sh`, `bash scripts/validate-pdf.sh`, `bash scripts/validate-skills.sh` e `git diff --check` executados nesta rodada.

### Limitações

Nenhum release foi publicado. CI remoto continua falhando antes dos steps por `runner_id=0`; instalação, runtime ARM64, PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 permanecem não validados.

---


## 2026-09-11 — Phase 48 — pinning imutável das actions

**Status:** implementação local; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

### Implementado

- Fixadas actions de CI/release em commits SHA completos, incluindo `dtolnay/rust-toolchain` na revisão atual de `stable`.
- Comentários de versão mantidos para revisão humana sem depender de tags mutáveis.
- Atualizados README, CHANGELOG, TODO e review da Phase 48.

### Verificação

- SHAs confirmados com `git ls-remote` nos repositórios upstream.
- `make test`: PASS — Rust, frontend e harness de áudio determinístico.
- `bash scripts/validate-docs.sh`, `bash scripts/validate-pdf.sh`, `bash scripts/validate-skills.sh` e `git diff --check`: PASS.

### Limitações

CI remoto segue falhando antes dos steps com `runner_id=0` e `steps=[]`; após push, runs `34629014531` (PR) e `34629009541` (push) falharam; hardware PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 continuam não validados.

---

## 2026-09-11 — Phase 47 — hardening dos workflows de release

**Status:** implementação local; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

### Implementado

- Corrigido filtro glob de tags em `.github/workflows/release.yml`.
- `validate-version` agora rejeita tags que não correspondem exatamente a `vX.Y.Z`, incluindo componentes numéricos com zero à esquerda.
- CI e jobs de leitura de release receberam `contents: read`; escrita ficou restrita a `github-release`.
- Atualizados README, CHANGELOG, TODO e review da Phase 47.

### Verificação

- `make test`: PASS — Rust e frontends.
- `bash scripts/validate-docs.sh`, `bash scripts/validate-pdf.sh`, `bash scripts/validate-skills.sh` e `git diff --check`: PASS.
- Revisão independente encontrou filtro de tag inválido e permissões amplas; ambos corrigidos.

### Limitações

Actions de terceiros ainda usam referências mutáveis por tag; pinning integral por SHA permanece follow-up. Após o push, runs `34626195615` (PR) e `34626191122` (push) falharam antes dos steps (`runner_id=0`, `steps=[]`); hardware PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 continuam não validados.

---

## 2026-09-11 — Phase 46 follow-up — correções apontadas em revisão independente

**Status:** correção local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Adicionado `make test-audio` à lista de comandos documentada em `docs/CLI.md`.
- Restringido `scripts/validate-pdf.sh` a caminhos PDF existentes sob `docs/guides`, com resolução canônica para rejeitar traversal e arquivos fora do escopo.

### Verificação

- `make test` passou: Rust, frontend e harness de áudio determinístico.
- `bash scripts/validate-docs.sh`, `bash scripts/validate-pdf.sh`, `bash scripts/validate-skills.sh` e `git diff --check` passaram.

### Limitações

CI remoto segue falhando antes dos steps (`runner_id=0`, `steps=[]`); `cargo clippy --all-features` permanece bloqueado pela ausência local de `jack.pc`. Hardware PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 continuam não validados.

---

## 2026-09-11 — Phase 46 — alinhamento do runner de release

**Status:** implementação local; PR #40 aberto; CI remoto falhou antes dos steps; não mergeado; não lançado.

### Implementado

- Alterado `runs-on` de `ubuntu-24.04` para `ubuntu-latest` nos seis jobs de `.github/workflows/release.yml`.
- Mantidos inalterados permissões, gates, dependências, artefatos e gatilho semver.
- README, CHANGELOG, TODO e review da Phase 46 atualizados.

### Verificação

- Validação estrutural confirmou seis jobs de release em `ubuntu-latest` e nenhum `ubuntu-24.04`.
- Gates locais Rust, documentação, PDF e skills executaram; `cargo audit` manteve o residual documentado `RUSTSEC-2023-0071`.
- `git diff --check origin/main...HEAD` passou sobre o diff completo antes do commit.

### Limitações

Runs `34614028392` e `34614025997` falharam antes dos steps com `runner_id=0` e `steps=[]`; token atual recebe HTTP 403 ao consultar runners/permissões. CI remoto, release, hardware PipeWire/ALSA, mídia WebRTC e Raspberry Pi 5 permanecem não validados.

### Próximo passo

Executar CI e workflow de release quando administrador desbloquear runner/permissões. Não fazer merge ou publicar release antes de gates remotos verdes.

---


## 2026-09-11 — Phase 45 — correção do label de runner

**Status:** implementação commitada; PR #40 aberto; CI remoto falhou antes dos steps; não mergeado; não lançado.

### Implementado

- Alterado `runs-on` de `ubuntu-24.04` para `ubuntu-latest` nos nove jobs do CI.
- README, CHANGELOG, START, TODO e review atualizados.

### Verificação

- Revisão da API GitHub confirmou todos os jobs anteriores com `runner_id=0`, sem steps executados (`34605340696`).
- `git diff --check` e gates locais pendentes antes do commit.

### Limitações

Token atual não permite consultar runners/configuração Actions (HTTP 403). CI remoto, hardware PipeWire/ALSA, mídia WebRTC, Raspberry Pi 5 e release continuam não validados.

### Próximo passo

Investigar desbloqueio de runner/permissões Actions com administrador; não fazer merge enquanto gates remotos não estiverem verdes.

---

## 2026-09-11 — Phase 44 — correção do gate de whitespace documental

**Status:** correção local; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

### Implementado

- Removido trailing whitespace de `docs/reviews/PHASE-37-REVIEW.md`, `PHASE-38-REVIEW.md`, `PHASE-39-REVIEW.md` e `docs/validation/PLATFORM-VALIDATION-MATRIX.md`.
- README, CHANGELOG e TODO alinhados ao estado real.

### Verificação

- `git diff --cached --check`: PASS antes do commit; o diff completo contra `origin/main` deve ser revalidado após commit.
- Testes Rust, clippy, typecheck/test/build dos dois frontends, docs, PDF e skills: PASS localmente.
- Review independente encontrou e confirmou a falha de whitespace; nenhuma correção de segurança foi aplicada por falta de evidência reproduzível para fixture de teste já existente.

### Limitações

- Runs `34614028392` e `34614025997` falharam antes dos steps em todos os jobs, com `runner_id=0` e `steps=[]`. API de permissões e runners retorna HTTP 403 para token atual. PipeWire/ALSA, WebRTC media, Raspberry Pi 5 e release permanecem não validados.

---

## 2026-09-11 — Phase 43 — gates de verificação e consistência documental

**Status:** implementação local; PR #40 aberto; CI remoto falha antes dos steps; não mergeado; não lançado.

### Implementado

- Gerado e versionado `server/Cargo.lock`; removida exclusão global do lockfile.
- CI ganhou job `documentation` para `validate-docs.sh`, `validate-pdf.sh`, `validate-skills.sh` e `git diff --check`.
- ADRs duplicados foram renumerados: WebSocket cap para ADR-013 e Compose safety para ADR-014.
- README, CHANGELOG, TODO, START e review da Phase 43 atualizados.

### Verificação

- `cargo test --manifest-path server/Cargo.toml --all`: PASS — suites Rust concluídas sem falhas.
- `cargo generate-lockfile --manifest-path server/Cargo.toml`: PASS — 355 pacotes resolvidos.
- CI remoto continua bloqueado antes dos steps; último run observado: `34599888854`, falha sem runner executável.
- Gates locais executados nesta rodada: fmt, clippy, cargo audit com lockfile, docs, PDF, skills e diff check — PASS.

### Limitações

CI remoto, hardware PipeWire/ALSA, mídia WebRTC, Raspberry Pi 5 e release v0.3.1 continuam pendentes.

---

## 2026-09-11 — Phase 42 — validação reproduzível do Musician Guide PDF

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Criado `scripts/validate-pdf.sh` para validar existência, extração de texto e renderização do PDF.
- `pdftotext` é preferido; `mutool` cobre VPS sem Poppler.
- `make docs` agora executa validação documental e validação do PDF.
- README, CHANGELOG e TODO registram o novo gate.

### Verificação

- `mutool draw -F txt`: extração local disponível.
- `mutool draw -r 120`: renderização local disponível.
- `bash scripts/validate-pdf.sh`: será executado no gate local desta rodada.

### Limitações

PDF não valida hardware, PipeWire, WebRTC media ou runtime ARM64. CI remoto segue bloqueado antes dos steps; run `34597175306` falhou com jobs sem steps e sem runner atribuído.

---

## 2026-09-11 — Phase 41 — pacote documental do Musician Guide

**Status:** documentação local atualizada; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Atualizada versão/data de `docs/guides/MUSICIANS-GUIDE.md` para Phase 41.
- Regenerado `docs/guides/MUSICIANS-GUIDE.pdf` com Pandoc + XeLaTeX a partir do Markdown atual.
- README, CHANGELOG e TODO registram estado e limitação real.

### Verificação

- `pandoc ... --pdf-engine=xelatex`: PASS; PDF regenerado.
- Renderização de página foi iniciada com `pdftoppm`, mas validação textual falhou porque `pdftotext` não está instalado no VPS.
- `bash scripts/validate-docs.sh`: PASS.
- `bash scripts/validate-skills.sh`: PASS.
- `git diff --check`: PASS.

### Limitações

PDF não pode receber validação completa de extração de texto neste ambiente até instalar `pdftotext`. PipeWire, WebRTC media, runtime ARM64 e hardware Raspberry Pi 5 continuam não validados. CI remoto continua bloqueado antes dos steps; runs `34594953314` e `34594948336` falharam com jobs sem steps.

---

## 2026-09-11 — Phase 40 — segurança operacional do Compose e rebuild explícito

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- `make up` agora executa `docker compose up -d --build`, evitando iniciar imagens obsoletas após alterações em código, Dockerfiles ou dependências.
- Portas dos UIs Compose foram limitadas a `127.0.0.1`, evitando exposição acidental dos servidores Vite de desenvolvimento.
- `make fmt` não reporta formatação frontend como executada quando os projetos não possuem script `format`.
- README, CHANGELOG, TODO e review da Phase 40 alinhados.

### Verificação

- `docker compose config`: PASS.
- `make -n up`: PASS; confirma `--build`.
- `make test-audio`: PASS — 4 testes.
- `bash scripts/validate-docs.sh`: PASS.
- `bash scripts/validate-skills.sh`: PASS.
- `git diff --check`: PASS.

### Limitações

Compose continua desenvolvimento-only, com HTTP inseguro explicitamente configurado e áudio `SIMULATED`. CI remoto continua bloqueado antes dos steps; nenhuma validação de Docker runtime, TLS, PipeWire, WebRTC media ou Raspberry Pi 5 foi alegada.

---

## 2026-09-11 — Phase 39 — cobertura do harness de áudio no Makefile

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Criado alvo `make test-audio` para executar `audio-engine` deterministic harness.
- `make test` agora inclui esse alvo, além dos testes Rust e frontend existentes.
- Help do Makefile informa que cobertura de áudio é `SIMULATED`.
- README, CHANGELOG, TODO e review da Phase 39 alinhados.

### Verificação

- `make test-audio`: PASS.
- `make test`: PASS, se dependências locais de frontend estiverem disponíveis.
- `make -n test`: PASS.
- `git diff --check`: PASS.
- Nenhum claim novo de hardware, PipeWire, WebRTC media, Raspberry Pi ou CI remoto.

### Limitações

O harness não valida áudio realtime, hardware, desempenho, stop/start ou runtime ARM64. CI remoto continua bloqueado antes dos steps com `runner_id=0`.

---

## 2026-09-11 — Phase 38 — consistência documental da CLI

**Status:** documentação local; PR #40 aberto; CI remoto bloqueado antes dos steps (run `34572824987`); não mergeado; não lançado.

### Implementado

- `docs/CLI.md` passou a declarar `iem` como implementado, com oito comandos fixos e versão `0.3.1`.
- Removida linguagem obsoleta de planejamento e `NOT STARTED`.
- README, CHANGELOG, TODO e esta trilha foram alinhados ao binário real.

### Verificação

- `cargo test --manifest-path server/Cargo.toml -p admin-cli`: PASS — 3 testes.
- `cargo run --quiet --manifest-path server/Cargo.toml --bin iem -- --version`: PASS — `iem 0.3.1`.
- `bash scripts/validate-docs.sh`: PASS.
- `git diff --check`: PASS.
- Nenhum claim novo de hardware, áudio realtime ou CI remoto.

---

## 2026-09-11 — Phase 37 — matriz de validação de plataforma

**Status:** documentação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Criada `docs/validation/PLATFORM-VALIDATION-MATRIX.md` para separar evidência de configuração, `SIMULATED`, `PENDING` e suporte não implementado.
- Registrados targets Linux x86_64, Docker Compose, Raspberry Pi 5 ARM64, Windows via Docker Desktop, Windows áudio nativo e macOS.
- Definidos gates de hardware para PipeWire/ALSA, WebRTC media, latência, XRUNs, perda, recuperação e TLS.
- Atualizados README, CHANGELOG, TODO e review da Phase 37.

### Verificação

- `bash scripts/validate-docs.sh`: PASS.
- `bash scripts/validate-skills.sh`: PASS.
- `git diff --check`: PASS.
- Nenhum teste de Raspberry Pi, PipeWire, áudio realtime ou WebRTC media foi alegado.

### Limitações

- Run CI `34557388136` falhou com oito jobs sem steps executados; GitHub reporta `runner_id=0`.
- Fechamento de deployment e mídia depende de Raspberry Pi 5 real.

## 2026-09-11 — Phase 35 — CLI de desenvolvimento `iem`

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Criado `server/admin-cli/src/bin/iem.rs` como dispatcher tipado para alvos existentes do Makefile.
- Comandos suportados: `help`, `status`, `diagnostics`, `docs`, `test`, `build`, `up` e `down`.
- Execução usa `Command::new("make")`, sem shell e sem entrada arbitrária.
- Código de saída do alvo é preservado; falha para iniciar `make` retorna 127.
- `iem run`, instalação global, pacote e suporte de plataforma não foram inventados.
- Atualizados README, CHANGELOG, TODO, `docs/CLI.md` e review da Phase 35.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo test --manifest-path server/Cargo.toml -p admin-cli`: PASS — 3 testes.
- `cargo clippy --manifest-path server/Cargo.toml -p admin-cli --all-targets -- -D warnings`: PASS.
- `iem --help`, `iem docs` e rejeição de `iem run`: PASS; `iem run` retorna exit 2.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.
- CI remoto permanece bloqueado antes dos steps com `runner_id=0`; não há evidência de execução GitHub Actions.

### Limitações

- `up`/`down` exigem Docker Compose; `test`/`build` exigem toolchains.
- PipeWire, WebRTC media e Raspberry Pi 5 continuam não validados.

---

---

## 2026-09-11 — Phase 36 — telemetria no Engineer Console

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Engineer Console passou a consultar `GET /api/v1/telemetry` junto com dados do dashboard.
- Backend e contador de XRUNs aparecem nos cards operacionais.
- Métrica `null` permanece `UNKNOWN`; nenhuma disponibilidade de áudio foi inventada.
- Criada revisão `docs/reviews/PHASE-36-REVIEW.md`.

### Verificação

- `npm test --prefix web/engineer -- --run --reporter=dot`: PASS — 2 testes.
- `npm run typecheck --prefix web/engineer`: PASS.
- `npm run build --prefix web/engineer`: PASS — Vite produziu `dist/`.
- `git diff --check`: PASS.
- PipeWire, WebRTC media e Raspberry Pi 5 continuam não validados.

---

## 2026-09-11 — Phase 36 — telemetria no Engineer Console

**Status:** implementação local; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Engineer Console passou a consultar `GET /api/v1/telemetry` junto com dados do dashboard.
- Backend e contador de XRUNs aparecem nos cards operacionais.
- Métrica `null` permanece `UNKNOWN`; nenhuma disponibilidade de áudio foi inventada.
- Criada revisão `docs/reviews/PHASE-36-REVIEW.md`.

### Verificação

- `npm test --prefix web/engineer -- --run --reporter=dot`: PASS — 2 testes.
- `npm run typecheck --prefix web/engineer`: PASS.
- `npm run build --prefix web/engineer`: PASS — Vite produziu `dist/`.
- `git diff --check`: PASS.
- PipeWire, WebRTC media e Raspberry Pi 5 continuam não validados.

---

## 2026-09-10 — Phase 34 — CLI output correctness

**Status:** implementação local verificada; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- `open-iem-admin` aceita HTTP somente para `localhost`, `127.0.0.1` e `::1`; URLs remotas exigem HTTPS antes de qualquer request ou envio de Bearer token.
- Tabelas JSON coletam união de campos de todos objetos, evitando perda de colunas quando primeira linha tem schema parcial.
- HTTP 404 agora retorna erro genérico de recurso ausente, sem afirmar que endpoint implementado ainda está planejado.
- Testes unitários cobrem união de campos e descarte de linhas não-objeto.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml -p admin-cli`: PASS — 2 testes.
- `cargo clippy --manifest-path server/Cargo.toml -p admin-cli --all-targets -- -D warnings`: PASS.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.
- Review independente encontrou inicialmente risco de Bearer sobre HTTP remoto; correção aplicada. Nova revisão ainda necessária antes do commit.

### Limitações

- CI remoto continua falhando antes dos steps com `runner_id=0`; não há evidência de execução GitHub Actions.
- API, PipeWire, WebRTC media e Raspberry Pi 5 permanecem não validados nesta fase.

---

## 2026-09-10 — Phase 33 — CI branch trigger diagnosis

**Status:** workflow corrigido localmente; CI remoto ainda bloqueado; sem merge ou release.

### Implementado

- Adicionado padrão `feat/**` aos gatilhos de push do `.github/workflows/ci.yml`.
- Correção cobre branch canônica atual `feat/phase24-ws-resilience`, que não correspondia ao padrão anterior `feature/**`.

### Verificação

- YAML do workflow alterado sem erro de sintaxe detectado pelo editor.
- Runs remotos `34540976629`, `34541036272` e novo push `34543057940` falharam em todos os jobs com `steps=[]`; evidência aponta bloqueio de runner antes da execução.
- Nenhuma conclusão de CI, release ou suporte de hardware foi declarada.

### Próximo passo

Desbloquear runner/permissões GitHub e repetir CI real; depois executar gates completos antes de merge.

---

## 2026-09-10 — Phase 32 — deterministic audio harness

**Status:** testes locais passam; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado; não lançado.

### Implementado

- Adicionado `server/audio-engine/tests/deterministic_harness.rs`, harness de integração `SIMULATED` sem hardware, rede ou valores de relógio de parede nas asserções de áudio.
- Harness cobre determinismo, isolamento entre mixes, ganho, pan, mute, limiter e finitude das amostras.

### Verificação

- `cargo fmt --all -- --check`: PASS.
- `cargo test -p audio-engine`: PASS — 20 testes unitários, 4 testes de integração e doc-tests.

### Limitações

- Harness não cobre hardware, desempenho realtime ou stop/start.
- CI remoto continua bloqueado antes dos steps. O novo run `34540976629` falhou em 8 jobs com `steps=[]`; PR #40 permanece aberto; não há merge ou release.

---

## 2026-09-10 — Phase 31 review and CLI contract audit

**Status:** documentação atualizada; PR #40 aberto; CI remoto bloqueado antes dos steps; não mergeado.

### Objetivo

Reconciliar documentação com commits reais e registrar contrato da interface local antes de implementar um binário `iem`.

### Implementado

- README e CHANGELOG deixam de chamar mudanças commitadas de working tree.
- Criada revisão `docs/reviews/PHASE-31-REVIEW.md`.
- Criado `docs/CLI.md`, distinguindo Makefile, `open-iem-admin` existente e o contrato implementado de `iem`.
- Corrigido estado documental da CLI: `iem` implementado com oito comandos fixos e versão `0.3.1`; instalação global e pacote continuam fora do escopo.

### Verificação

- Estado Git local limpo antes da edição; documentação editada sem código executável.
- Runs remotos 34532261471 e 34532315872 falharam com `steps=[]` e `runner_id=0`.
- API de Actions retorna HTTP 403 para o token disponível; causa de configuração do runner não pôde ser confirmada.

### Decisões e limitações

- Não trocar `ubuntu-24.04` por `ubuntu-latest` sem evidência; falha ocorre antes dos steps.
- Não criar alias `iem` antes de definir instalação, códigos de saída e compatibilidade.
- PipeWire, WebRTC media e Raspberry Pi 5 continuam `SIMULATED`/`HARDWARE VALIDATION REQUIRED`.

### Próximo passo

Validar documentação e gates locais; depois investigar desbloqueio do GitHub Actions com credencial/permissão adequada.

---

## 2026-09-10 — Compose compatibility cleanup

**Status:** implementação commitada nesta branch; PR #40 aberto; CI bloqueado antes dos steps; não lançado.

### Implementado

- Removido campo top-level `version` obsoleto de `docker-compose.yml`.
- README e CHANGELOG registram que Compose continua dev-only e sem runtime validado.

### Verificação

- `docker compose config`: PASS, sem aviso de atributo `version` obsoleto.
- `git diff --check`: PASS.
- Docker build/startup, PipeWire, WebRTC media e Raspberry Pi continuam não validados.

---

## 2026-09-10 — Phase 31 follow-up — fail-closed rollback and snapshot baseline

**Status:** local implementation, uncommitted/unmerged; CI blocked; not released.

### Implementado

- Refresh resolve owner before rotation; missing user no longer consumes valid refresh token.
- Signing-failure cleanup discards replacement mapping/token without reactivating old revoked state.
- HTTP `ApiError::Internal` responses now return generic message; details stay server-side.
- Musician keeps delayed REST snapshot as baseline when WebSocket revision advanced first.

### Verificação

- `cargo test -p api-server --test integration`: PASS — 57 passed, 0 failed.
- `cargo clippy -p api-server --all-targets -- -D warnings`: PASS.
- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- Musician frontend: `npm test -- --run --reporter=dot`: PASS — 34 passed. A prior default invocation timed out at 120 s; reporter mode completed in 2.32 s.

### Implemented

- Persistent access-session mappings bind JWT `jti`, user ID and refresh-session ID.
- HTTP middleware rejects revoked, rotated, expired, deleted-user or unknown access mappings.
- Refresh rotation, logout, admin session revoke, replay-family revoke and user deletion invalidate access mappings.
- Established WebSocket connections re-check mapping state on inbound messages and keepalive ticks; revocation returns `SESSION_REVOKED` and closes loop.

### Verification

- Added focused integration regressions for refresh rotation, session revoke, user deletion and established WebSocket revocation.
- `cargo fmt --all -- --check`: PASS.
- `cargo test -p api-server --test integration`: PASS — 57 passed, 0 failed.
- `cargo clippy -p api-server --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: BLOCKED by missing system dependency `jack` (`jack.pc` / `jack-sys`); api-server integration tests pass independently.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: BLOCKED by same missing `jack` system dependency.
- CI remains blocked before workflow steps; no release claim.

---

## 2026-09-10 — Phase 30 Docker Compose development images

**Phase:** 30 — local development packaging

### Implementado

- Adicionados `server/Dockerfile.dev`, `web/musician/Dockerfile.dev` e `web/engineer/Dockerfile.dev`.
- Imagens usam toolchains oficiais, dependências instaladas durante build e servidores expostos nas portas documentadas.
- Removido `target: api-server` obsoleto do Compose; imagem API agora usa estágio único.
- Chaves JWT continuam montadas somente em runtime via `./keys` e HTTP inseguro permanece explicitamente restrito a desenvolvimento isolado.

### Verificação

- `docker compose config`: PASS; emite apenas aviso de atributo `version` obsoleto.
- Workspace Rust: testes e clippy PASS.
- Musician: 34 testes e build PASS.
- Engineer: 2 testes e build PASS.
- Build real das imagens Docker não executado nesta rodada: daemon Docker/chaves JWT locais não disponíveis para validação completa.
- CI remoto continua bloqueado antes dos steps por `runner_id=0`.

### Limitações

Docker runtime, PipeWire, WebRTC media, ARM64 real no Raspberry Pi e release continuam não validados.

---

## 2026-09-10 — Windows/Docker documentation audit

**Phase:** documentation follow-up

### Implementado

- Guia Windows + Docker Desktop adicionada em `docs/guides/WINDOWS-DOCKER-GUIDE.md`.
- Auditoria confirmou que `docker-compose.yml` referencia três Dockerfiles ausentes; guia e CHANGELOG agora marcam Compose como `BLOCKED`, sem alegar runtime validado.
- README, TODO e review da Phase 29 atualizados.

### Verificação

- Cargo workspace: testes e clippy passam.
- Musician: 34 testes e build passam.
- Engineer: 2 testes e build passam.
- CI remoto continua bloqueado antes dos steps por `runner_id=0`.

---

## 2026-09-10 — Bounded failed WebSocket authentication limiting

**Phase:** 29 — WebSocket security follow-up

### Implementado

- Adicionado limiter em memória bounded para falhas de autenticação de `/ws/v1`: 5 falhas por IP em janela de 60 segundos.
- Estado limitado a 4.096 IPs; entrada menos recentemente observada é removida quando limite é atingido.
- Apenas falhas de protocolo/token inválido contam. Requests WebSocket autenticados com sucesso não consomem orçamento.
- IP vem exclusivamente de `ConnectInfo<SocketAddr>`; cabeçalhos encaminhados não são confiáveis.
- Após threshold, resposta retorna HTTP 429 com `Retry-After: 5`.

### Verificação

- Testes unitários determinísticos cobrem threshold, reset de janela e limite de estado.
- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test -p api-server --all-targets --no-fail-fast`: PASS — 38 unitários e 52 integração.
- `cargo clippy -p api-server --all-targets -- -D warnings`: PASS.
- `scripts/validate-docs.sh` e `git diff --check`: PASS.
- CI remoto permanece indisponível; run `34509968438` falhou antes dos steps com `runner_id=0`.

---

## 2026-09-10 — WebSocket admission quotas

**Phase:** 28 — WebSocket connection fairness

### Implementado

- Adicionadas quotas atômicas em memória para 64 conexões por processo, 4 por usuário autenticado e 16 por IP do peer TCP.
- Reserva usa `ConnectInfo<SocketAddr>` e guarda RAII; descarte libera contadores mesmo em encerramento normal ou erro.
- Quota global retorna HTTP 503; quota de usuário/IP retorna HTTP 429, sempre com `Retry-After: 5`.
- `X-Forwarded-For` não é confiável e não é usado.
- Testes unitários cobrem aceitação, rejeição, liberação, rollback e concorrência.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — suite completa.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.
- Reviews independentes: PASS, sem blockers de segurança ou lógica.

### Limitações

Quota é local ao processo; múltiplas instâncias exigem coordenador compartilhado. Revogação pós-emissão de JWT, rate limit de tentativas inválidas, PipeWire, WebRTC real, runtime ARM64 no Raspberry Pi e CI remoto continuam pendentes/bloqueados.

---

## 2026-09-10 — WebSocket keepalive loop timing coverage

**Phase:** 27 — WebSocket resilience follow-up

### Implementado

- Adicionado teste determinístico com relógio Tokio pausado para validar primeiro Ping em 30 s, timeout de Pong somente após 60 s pendente e comportamento de intervalo com desafio ainda aberto.
- Habilitada feature `tokio/test-util` somente nas dependências de teste do `api-server`.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- Testes unitários de WebSocket: 3 PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `git diff --check`: PASS.

### Limitações

Teste valida política temporal isolada, não loop de transporte WebSocket real. CI remoto, PipeWire, mídia WebRTC e ARM64 Raspberry Pi continuam não validados.

---

## 2026-09-10 — WebSocket ACK revision race correction

**Phase:** 27 — WebSocket resilience follow-up

### Implementado

- Revisão monotônica do Musician agora registra `State`, `SendAck` e `MasterAck` aceitos antes de atualizar snapshot.
- Snapshot REST inicial atrasado não pode mais rebaixar revisão observada por ACK WebSocket recebido antes dele.
- Adicionado teste determinístico da corrida ACK antes do primeiro snapshot.

### Verificação

- Musician: 34 testes, typecheck e build: PASS.
- `scripts/validate-docs.sh` e `scripts/validate-skills.sh`: PASS.
- CI remoto continua falhando antes dos steps por runner/permissão.
- PipeWire, mídia WebRTC real e ARM64 Raspberry Pi permanecem não validados.

---

## 2026-09-10 — WebSocket state recovery after broadcast lag

**Phase:** 27 — WebSocket resilience follow-up

### Implementado

- Broadcast `Lagged` em canais de send/master agora envia `State` com revisão autoritativa.
- Musician refaz snapshot REST autenticado ao receber `State`, recuperando deltas perdidos.
- Musician valida e aplica `MasterAck` recebido de broadcasts no snapshot local.
- Adicionado teste do hook para atualização de master.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 218 testes Rust e 3 doc-tests.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- Musician typecheck, 33 testes e build: PASS.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.
- CI remoto segue bloqueado antes dos steps por runner/permissão.
- PipeWire, mídia WebRTC real e ARM64 Raspberry Pi permanecem não validados.

---

## 2026-09-10 — WebSocket oversized-message transport coverage

**Phase:** 26 — WebSocket resilience follow-up

### Implementado

- Adicionado teste de integração que envia mensagem Text com `16 * 1024 + 1` bytes.
- O teste confirma encerramento do transporte pelo limite do `WebSocketUpgrade`, antes do parser da aplicação.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- Teste dedicado: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- Primeira execução revelou comportamento real do `axum-test`: conexão é resetada sem handshake de fechamento; teste foi ajustado para validar erro de recebimento, sem inventar `Close`.

### Limitações

Cobertura valida mensagem Text não fragmentada. Frame binário oversized e mensagem fragmentada acima do limite continuam pendentes. CI remoto, PipeWire, mídia WebRTC real e ARM64 Raspberry Pi permanecem não validados.

---

## 2026-09-10 — WebSocket upgrade frame-size enforcement

**Phase:** 26 — WebSocket resilience follow-up

### Implementado

- Configurados `max_message_size` e `max_frame_size` do `WebSocketUpgrade` para 16 KiB.
- O limite agora bloqueia payloads antes da alocação excessiva e complementa a validação manual de mensagens de texto.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 220 testes.
- Review independente identificou o risco de defaults permissivos do Axum; corrigido nesta rodada.

### Limitações

CI remoto segue falhando antes dos steps por runner/permissão. PipeWire, mídia WebRTC real e ARM64 Raspberry Pi permanecem não validados.

---


## 2026-09-10 — WebSocket keepalive state-machine correction

**Phase:** 26 — WebSocket resilience follow-up

### Implementado

- Extraído `KeepaliveTracker` para manter desafio pendente, instante do Ping e validação de Pong em uma máquina de estados explícita.
- Timeout só é avaliado enquanto existe Pong pendente; Pong válido não provoca falso timeout no intervalo seguinte.
- Teste determinístico cobre timeout, Pong incorreto, Pong correlacionado e ausência de falso timeout após Pong válido.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 220 testes.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.

### Limitações

Teste temporal do loop WebSocket ainda pendente. CI remoto segue falhando antes dos steps por runner/permissão. PipeWire, mídia WebRTC real e ARM64 Raspberry Pi permanecem não validados.

### Próximo

Rever diff atual com agentes independentes e investigar runner GitHub.

---

## 2026-09-10 — WebSocket keepalive challenge correlation

**Phase:** 26 — WebSocket resilience follow-up

### Implementado

- Servidor envia Ping com payload por sessão e registra desafio pendente.
- Somente Pong com payload exatamente igual ao último Ping enviado atualiza liveness; Pong não solicitado ou obsoleto não estende conexão.
- Adicionado teste unitário para rejeição de Pong não solicitado e payload incorreto.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 219 testes.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- Frontends Musician/Engineer já validados nesta rodada: typecheck, testes e build PASS.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.

### Limitações

Teste temporal do loop WebSocket ainda pendente. CI remoto segue falhando antes dos steps por runner/permissão. PipeWire, mídia WebRTC real e ARM64 Raspberry Pi permanecem não validados.

### Próximo

Adicionar teste temporal determinístico do loop sem esperar 30/60 segundos; depois investigar runner GitHub.

---


## 2026-09-10 — WebSocket request correlation

**Phase:** 26 — WebSocket error correlation

### Implementado

- Erros `FORBIDDEN` gerados após envelope válido agora preservam `request_id` original, incluindo rejeições de RBAC e ownership.
- Erros de parsing continuam usando `request_id` sintético `server`, pois entrada inválida não fornece correlação confiável.
- Adicionado teste de integração para correlação em rejeição de ownership.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 218 testes.
- `git diff --check`: PASS.
- Reviews independentes: falha inicial encontrada no caminho de ownership; corrigida antes da entrega.

### Limitações

CI remoto continua bloqueado antes dos steps. PipeWire, mídia WebRTC real e ARM64 em Raspberry Pi permanecem não validados.

### Próximo

Desbloquear runner GitHub; depois implementar ressincronização explícita após perda de broadcast.

---

## 2026-09-10 — WebSocket protocol error redaction

**Phase:** 25 — WebSocket observability hardening

### Implementado

- Erros de decodificação agora usam códigos e mensagens públicas estáveis; detalhes de `serde_json`, versão recebida e conteúdo inválido não saem para o cliente.
- Logs de falha de recebimento não incluem texto bruto do erro; conexão usa `session_id` para correlação.
- Fechamento normal do peer e falha de transporte foram separados de expiração JWT; somente timeout real de leitura envia `TOKEN_EXPIRED`.
- Adicionado teste unitário de não vazamento de detalhes de parser.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 218 testes.
- PipeWire, mídia WebRTC real, ARM64 em Raspberry Pi e CI remoto continuam não validados.

### Próximo

Corrigir correlação de `request_id` em erros após envelope validado; depois implementar ressincronização após perda de broadcast.

---

## 2026-09-10 — WebSocket connection cap

**Phase:** 24 — release hardening

### Implementado

- Adicionado limite process-wide de 64 conexões WebSocket atualizadas com `tokio::sync::Semaphore`.
- Permissão é reservada antes do upgrade e mantida pelo handler; liberação ocorre no disconnect.
- Excesso recebe HTTP 503 e `Retry-After: 5`.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 216 testes.
- Frontends Musician: typecheck, 32 testes e build PASS.
- Frontend Engineer: typecheck, 2 testes e build PASS.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.

### Limitações

- Quota por conexão agora inclui todo frame recebido, inclusive control frames; quotas por usuário/IP, revogação pós-emissão de JWT e teste de saturação do limite continuam pendentes.
- CI remoto, PipeWire, mídia WebRTC real e ARM64 em Raspberry Pi permanecem não validados.

---

## 2026-09-10 — Keepalive policy extracted and unit-tested

**Phase:** 24 — release hardening

### Implementado

- Extraídos intervalos de keepalive para constantes nomeadas.
- Adicionado helper puro `keepalive_expired` com testes de fronteira no timeout de 60 segundos.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --package api-server --lib --tests ws_ -- --nocapture`: PASS — 20 testes relevantes (18 integração + 2 unitários).
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `git diff --check`: PASS.

### Limitações

- Testes de loop WebSocket com relógio controlado ainda não existem; transporte real, PipeWire e ARM64 permanecem não validados.

---

## 2026-09-10 — WebSocket control-frame coverage and quota hardening

**Phase:** 24 — release hardening

### Implementado

- Quota por minuto agora contabiliza somente mensagens `Text` de aplicação; `Ping`, `Pong` e `Close` não consomem limite.
- Adicionados testes de integração para preservação de payload em `Ping`/`Pong` e rejeição fail-closed de frame binário com `INVALID_MESSAGE`.
- Atualizados README, CHANGELOG, TODO e review da Phase 24.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test --manifest-path server/Cargo.toml --package api-server --test integration ws_ -- --nocapture`: PASS — 18 testes.

### Limitações

- Testes temporais determinísticos de keepalive ainda pendentes; intervalos de produção seguem 30/60 segundos.
- CI GitHub continua falhando antes dos steps por indisponibilidade/permissão de runner. PipeWire, WebRTC media e ARM64 real seguem não validados.

### Próximo

Extrair política de keepalive para teste com relógio controlado; depois investigar desbloqueio real do runner GitHub.

---

## 2026-09-10 — Extended engineering contract and local developer interface

**Phase:** 24 — release hardening

### Implementado

- Incorporados requisitos adicionais de continuidade, CI diagnostics, release blocking, hardware matrix, traceability, audio harness, recovery e architecture fitness em `START.md`, preservando regra de auditar antes de continuar.
- Criado `Makefile` root com targets locais exigidos; comandos não implementados não são simulados.
- Criado `scripts/validate-environment.sh` com estados explícitos para ferramentas, PipeWire, hardware e WebRTC.
- Atualizado backlog com CLI `iem`, harness de áudio e validações físicas como itens rastreáveis.

### Verificação

- `make help`: PASS.
- `make install`: PASS.
- `make diagnostics`: PASS; Docker Compose detectado, PipeWire/ALSA e Raspberry Pi marcados `HARDWARE VALIDATION REQUIRED`.
- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 216 testes executados nesta rodada.
- `scripts/validate-docs.sh`, `bash -n scripts/validate-environment.sh` e `git diff --check`: PASS.

### Revisão independente

- Test Agent: MEDIUM — keepalive ainda sem teste temporal; `run-local` não é simulação isolada.
- Security Review: sem BLOCKER/HIGH; MEDIUM — limite global de conexões e revogação pós-emissão ainda pendentes; binary frames antes eram descartados silenciosamente e agora fecham fail-closed.
- Code Review: corrigidos diagnóstico falso de Compose, nomenclatura enganosa de `run-local` e entrada duplicada no log.

### Próximo

Desbloquear CI remoto; depois implementar CLI `iem` e harness de áudio conforme backlog, sem avançar release por suposição.

---

## 2026-09-09 — Phase 24 audit: estado real e estabilização de testes

**Branch:** `main`
**Ambiente:** Linux x86_64; PipeWire, mídia WebRTC e ARM64 real não disponíveis

### Implementado

- Auditou estado existente sem reiniciar o projeto; `main` está em `8db59da` e PR #38 já foi mergeada.
- Adicionado teste de falha SQLite no lookup de ownership WebSocket; autorização permanece fail-closed.
- Adicionado keepalive Ping/Pong e timeout de inatividade no WebSocket.
- Todas as escritas WebSocket agora têm timeout de 10 segundos contra clientes lentos.
- Estabilizados testes de broadcast WebSocket após handshake, evitando corrida do scheduler.

### Verificação

- Rust workspace: testes PASS — 49 integração API + crates; clippy e fmt PASS.
- Musician: typecheck, 32 testes e build PASS.
- Engineer: typecheck, 2 testes e build PASS.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --check`: PASS.

### Devil's Advocate / limitações

- CI remoto ainda bloqueado antes dos steps por infraestrutura/permissão; não declarar release `v0.3.1` pronta.
- PipeWire real, mídia WebRTC, ARM64 em Raspberry Pi 5, latência e estabilidade de 60 minutos continuam `SIMULATED`/`HARDWARE VALIDATION REQUIRED`.
- Testes locais de broadcast eram scheduler-dependent; correção só estabiliza o teste, não prova transporte de áudio real.

### Próximo

Desbloquear GitHub Actions com credencial/permissão válida; executar CI real; depois validar artefato ARM64 no Raspberry Pi 5 antes de release.

---

## 2026-09-09 — Phase 23 follow-up: observabilidade de ownership WebSocket

**Branch:** `fix/phase23-ownership-observability`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- Consultas SQLite de ownership usadas por mutações e broadcasts WebSocket agora passam por helper único.
- Falhas de banco geram `warn!` estruturado e permanecem fail-closed; erro nunca vira ownership ausente silencioso.
- README, CHANGELOG, TODO e review da Phase 23 atualizados.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS.
- `cargo test --manifest-path server/Cargo.toml --all`: PASS — 215 testes.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS.
- `scripts/validate-skills.sh`, `scripts/validate-docs.sh` e `git diff --check`: PASS.
- Reviews independentes de segurança e código: PASS; sem concerns de segurança ou erros lógicos.

### Limitações

CI remoto continua sem executar steps; GitHub API retorna 403 para permissões/logs com token atual. PipeWire, Opus, mídia WebRTC real e runtime ARM64 continuam SIMULATED/não validados no VPS.

### Próximo

Corrigir acesso do GitHub Actions ou executar com token autorizado; depois publicar e validar release `v0.3.1`. Avaliar contenção de `mix_assignment_lock` em leitura DB.

---

## 2026-09-09 — Phase 23 follow-up: documentação e gates locais

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- Guia do Músico atualizado de Phase 20 para Phase 23.
- Estado remoto consultado: `v0.3.1` falhou no gate de consistência; CI de `main` falhou antes de executar steps, sem logs acessíveis ao token atual.

### Verificação

- Rust fmt, 159 testes workspace e clippy `-D warnings`: PASS.
- Musician: `npm ci`, typecheck, 32 testes, build e audit high: PASS.
- Engineer: `npm ci`, typecheck, 2 testes, build e audit high: PASS.
- `scripts/validate-skills.sh` e `git diff --check`: PASS.

### Limitações

CI remoto segue BLOCKED por falha de infraestrutura/permissão antes dos steps; causa detalhada não está disponível via API autenticada atual. Release `v0.3.1` não deve ser declarada publicada. PipeWire, Opus, mídia WebRTC real e runtime ARM64 continuam SIMULATED/não validados no VPS.

### Próximo

Publicar mudança documental em branch/PR, corrigir acesso aos logs ou reexecutar CI com token autorizado, depois validar pipeline `v0.3.1`.

---

## 2026-09-09 — Release 0.3.1: correção do gate de versão

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- Detectado que a tag remota `v0.3.0` apontava para `74e8b91`, anterior ao commit `c41dccd` que sincronizou versões.
- Versão de workspace Rust, frontends e lockfiles avançada para `0.3.1`.
- CHANGELOG atualizado para preparar release coerente; não houve reescrita da tag `v0.3.0`.

### Verificação

- Testes e builds locais da rodada anterior: PASS.
- CI remoto da tag `v0.3.0`: FAIL no job `Validate version consistency`; jobs seguintes foram skipped.
- Causa real: conteúdo da tag ainda declarava versão `0.2.0`.

### Limitações

`v0.3.0` remoto segue apontando para commit inconsistente. Release `v0.3.1` depende de commit, tag e pipeline novos. ARM64, PipeWire, Opus e mídia WebRTC real seguem não validados no VPS.

### Próximo

Commitar e publicar `v0.3.1`; aguardar CI e Release completos antes de declarar artefatos publicados.

---

## 2026-09-09 — Phase 21: autenticação WebSocket sem token em URL

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- Middleware autentica `/ws/v1` usando `Sec-WebSocket-Protocol: openiem.bearer.<JWT>, openiem.v1`; endpoints HTTP continuam usando `Authorization: Bearer`.
- `WebSocketUpgrade::protocols` seleciona e ecoa somente `openiem.v1`; bearer não retorna no handshake.
- Cliente Musician usa URL fixa `/ws/v1` e passa token somente como subprotocolo.
- Testes cobrem extração de protocolo, ausência de credencial, integração WS e cliente.

### Verificação

- Rust fmt, 41 testes de integração + 19 unitários e clippy `-D warnings`: PASS.
- Musician: 32 testes, typecheck e build: PASS.
- Engineer: 2 testes, typecheck e build: PASS.
- `git diff --check`: PASS.

### Limitações

TLS de exposição externa, PipeWire/Opus, mídia WebRTC real, telemetria e runtime ARM64 continuam pendentes ou SIMULATED no VPS.

### Próximo

Validar TLS fail-closed e deployment no Raspberry Pi 5.

---

## 2026-09-09 — Phase 20: verificação da reconciliação Musician

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- Testes do hook cobrem snapshot REST autenticado, snapshot aninhado inválido, snapshot atrasado e aplicação de `SendAck`.
- Cliente usa header `Authorization: Bearer`, cancela fetch no cleanup e valida envelope WebSocket completo com versão e `request_id` limitado.
- ACK rejeita ganho e pan fora das faixas do servidor.

### Verificação

- Musician typecheck, 32 testes e build: PASS.
- Rust fmt, 189 testes e clippy `-D warnings`: PASS.
- `git diff --check`: PASS.

### Limitações

Token WebSocket em query string, TLS, PipeWire/Opus, mídia WebRTC e runtime ARM64 continuam pendentes ou SIMULATED no VPS.

### Próximo

Projetar autenticação WebSocket sem token em URL e avançar TLS fail-closed/hardware Raspberry Pi 5.

---

## 2026-09-09 — Phase 19: reconciliação completa do cliente Musician

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- Cliente Musician busca snapshot autenticado em `GET /api/v1/state` depois de abrir WebSocket.
- Validação runtime cobre estrutura aninhada, revisões, índices, tipos, finitude e limites de pan.
- Referência monotônica rejeita snapshot REST atrasado após ACK/State mais novo.
- `SendAck` valida alvo e atualiza send no snapshot local; mutations usam `SetSendGain`/`SetSendMuted` e não assumem mix 0.
- Tipos TypeScript refletem snapshot e comandos server-side.

### Verificação

- `npm run typecheck` PASS.
- `npm test -- --run` PASS: 28 testes.
- `npm run build` PASS.
- `git diff --check` PASS.
- Revisão independente detectou falhas iniciais; correções aplicadas. Nova rodada pendente após documentação.

### Limitações

Testes ainda não cobrem corrida assíncrona de fetch em browser real. PipeWire, Opus, mídia WebRTC real, telemetria e runtime ARM64 continuam SIMULATED/não validados no VPS.

### Próximo

Testes específicos de reconciliação; depois TLS fail-closed e validação Raspberry Pi 5.

---

---

## 2026-09-09 — Phase 18: reconciliação de revisão no cliente Musician

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- Tipos TypeScript agora representam `SendAck` do protocolo server.
- `useWebSocket` atualiza revisão para `State` e `SendAck` somente quando valor recebido não é mais antigo que revisão atual.
- Teste cobre ACK, mensagem `State` atrasada, payload inválido e disconnect manual.
- Validação runtime rejeita revisões e payloads `SendAck` inválidos; handlers de sockets antigos são ignorados, revisão reseta ao reconectar e disconnect atualiza status.

### Limitações

A UI ainda não reconstrói canais/sends a partir de deltas nem solicita snapshot quando detecta lacuna de revisão. PipeWire, Opus, mídia WebRTC real, telemetria e runtime ARM64 continuam SIMULATED/não validados.

---

## 2026-09-09 — Phase 17: broadcast de deltas WebSocket

**Branch:** `feat/phase17-ws-broadcast-delta`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- `AppState::event_tx` distribui deltas de gain, pan e mute entre sessões WebSocket.
- Engineer/Admin recebem `SendAck` não solicitado de outras sessões.
- Musician recebe apenas deltas do mix atribuído; originador recebe ACK direto sem duplicação.
- `mix_assignment_lock` coordena ownership, dispatch, publicação e leitura de assignment no filtro outbound.
- Revisões de mutações preservam ordem de publicação; filtro outbound constrói payload sob lock e libera lock antes de I/O.
- Teste de integração cobre mutator + observer simultâneos.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all` PASS.
- `cargo test --manifest-path server/Cargo.toml --all` PASS: 189 testes.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings` PASS.
- `git diff --check` PASS.
- Revisão independente encontrou e corrigiu ownership sem lock e reorder concorrente.

### Limitações

PipeWire, Opus, mídia WebRTC real, telemetria e runtime ARM64 permanecem SIMULATED/não validados.

---

---

## 2026-09-09 — Phase 15 follow-up: correção de configuração documentada

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- Musician Guide corrigido para usar `OPENIEM_BIND_ADDR` e `OPENIEM_ALLOW_INSECURE_HTTP`, nomes reais consumidos por `api-server`.
- README aponta para review vigente da Phase 15.
- ADR-008 e TODO refletem autenticação implementada e TLS ainda pendente para exposição externa.

### Verificação

- Testes Rust, clippy, fmt e validação documental repetidos antes do commit.
- Reviews independentes confirmaram nomes de configuração e ausência de alteração executável.
- Auditoria Python não executada: scanner prescrito não existe em `/root/scan_patterns.py` nem no repositório.

---

## 2026-09-09 — Phase 15 follow-up: documentação, versões e CI fail-closed

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- README e Musician Guide corrigidos para refletir Engineer Console, API Admin e ownership já entregues.
- Versão `0.2.0` sincronizada nos dois frontends e lockfiles.
- CI dos frontends Musician e Engineer deixou de converter falhas de instalação, typecheck e teste em sucesso silencioso.
- Quality gate de release agora executa typecheck, testes e `npm audit` dos dois frontends antes dos builds.

### Verificação executada

- Rust fmt, testes workspace e clippy `-D warnings` PASS.
- Musician: typecheck, 25 testes, build e `npm audit --audit-level=high` PASS.
- Engineer: typecheck, 2 testes, build e `npm audit --audit-level=high` PASS.
- `scripts/validate-docs.sh`, `scripts/validate-skills.sh` e `git diff --cached --check` PASS.
- PDF regenerado via pandoc/xelatex; `file` confirma PDF 1.5. Extração não validada: `pdftotext` ausente no VPS.

---

## 2026-09-09 — Phase 15: atomicidade de ownership

**Branch:** `main`
**Ambiente:** VPS Linux x86_64; áudio SIMULATED

### Implementado

- `mix_assignment_lock` cobre listagem de assignments, leitura de sends e mutações de gain, pan e mute.
- Ownership é validado sob o mesmo lock usado por assign/unassign, snapshot e signaling.
- Janela TOCTOU entre assignment persistido e alteração de send foi fechada.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check` PASS.
- `cargo test --manifest-path server/Cargo.toml --all` PASS: 141 testes.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings` PASS.

### Limitações

- PipeWire, Opus, mídia WebRTC e telemetria real permanecem SIMULATED no VPS.
- TLS de exposição externa continua pendente.

---

## 2026-09-09 — Phase 14: snapshot e telemetria

**Branch:** `main`
**Ambiente:** VPS Linux; áudio SIMULATED

### Implementado

- `GET /api/v1/state` agora retorna contrato `schema_version=1` com canais, mixes e sends configurados.
- Snapshot captura estado sob lock único e serializa a leitura de assignment com `mix_assignment_lock`; rotas de mutação existentes permanecem cobertas pelo backlog de atomicidade.
- `GET /api/v1/telemetry` exige `ENGINEER` ou `ADMIN`; métricas não conectadas retornam `null` e backend `simulated`.
- Contrato registrado em `docs/specifications/API-SNAPSHOT-TELEMETRY.md` e ADR-010.

### Verificação

- `cargo fmt --all` PASS.
- `cargo test --workspace` PASS: 33 testes HTTP, total workspace atualizado.
- `cargo clippy --workspace --all-targets -- -D warnings` PASS após correções.

### Limitações

- PipeWire, Opus, frames, XRUN e medidores reais permanecem pendentes no VPS.

---

## 2026-09-09 — Phase 13: signaling ownership hardening

**Branch:** `main`
**Ambiente:** VPS Linux; mídia SIMULATED

### Implementado

- `POST /api/v1/audio/offer` valida `mix_id` informado por `MUSICIAN` como índice numérico dentro da capacidade e compara com assignment persistido para `uid` autenticado.
- Músico sem assignment ou apontando para outro mix recebe `403 Forbidden` antes de parsing SDP/criação de sessão.
- Engineer/Admin preservam operação; ausência de `mix_id` permanece compatível.
- ADR-009 criado e backlog/status corrigidos.

### Verificação

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check` PASS.
- `cargo test --manifest-path server/Cargo.toml --all` PASS: 173 testes, 0 falhas.
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings` PASS.

### Limitações

- PipeWire, Opus e mídia WebRTC real continuam não validados no VPS.
- Rate limit HTTP, limite global de sessões e revogação imediata de access token continuam pendentes; assignment e signaling usam lock compartilhado para evitar TOCTOU.

---

## 2026-09-09 — Phase 6: Engineer Console dashboard

**Branch:** `main`
**Ambiente:** VPS Linux; áudio SIMULATED

### Implementado

- `web/engineer` ganhou login com token em memória, refresh por cookie HttpOnly após 401, dashboard autenticado, polling de 5 segundos, revision do estado, sessões WebRTC e gestão de assignment dos dois mixes.
- UI usa somente contratos API existentes. Catálogo de usuários permanece Admin-only; assignment recebe ID numérico.
- Erros HTTP/rede têm estado visível. Nenhuma métrica, canal ou mídia real é inventada.
- CSS responsivo próprio e indicador explícito de áudio SIMULATED.

### Verificação

- Frontend: typecheck, 2 testes e build PASS.
- Backend: fmt, 172 testes e clippy PASS.
- Documentação: `scripts/validate-docs.sh` PASS.

### Pendências

Snapshot completo, meters, telemetria e PipeWire/Opus dependem de contrato API e validação em Raspberry Pi 5.

---

## 2026-09-09 — Phase 12: release fixes, Dependabot, self-delete protection, SBOM

**Branch:** `feat/phase12-release-fixes`
**Tests:** 172 passed (↑ from 171), 0 failed

### Implemented

Fixed artifact naming bug in release pipeline (`validate-version` not in build job needs chain). Build jobs `build-server-x86`, `build-server-arm64`, `build-web` now have `needs: [quality-gate, validate-version]` so `${{ needs.validate-version.outputs.version }}` resolves correctly.

Added Dependabot for Cargo, npm (musician/engineer), and GitHub Actions in `.github/dependabot.yml`.

Admin self-delete protection: `DELETE /api/v1/admin/users/{id}` returns 403 Forbidden if caller's `uid` matches the path ID. Integration test `admin_cannot_delete_own_account` added.

SBOM generation added to release pipeline (best-effort, `continue-on-error: true`): installs `cargo-sbom` and generates `open-iem-server-<ver>-sbom.json`, uploaded as artifact and included in GitHub Release.

**Verification:** cargo fmt PASS; cargo clippy PASS; cargo test --all PASS: 172 tests, 0 failures.

---

## 2026-09-09 — Phase 8: DSP Chain Integration + Admin CLI + Musician Guide PDF

**Branch:** `feat/phase8-dsp-chain`
**Tests:** 157 passed (↑ from 152), 0 failed

### Implemented

#### EQ + Compressor wired into Mix::process (`mix-engine/src/mix.rs`)
- `Mix` struct gains `pub eq: ParametricEq` and `pub compressor: Compressor` fields.
- `Mix::new()` initializes both disabled (flat EQ, compressor off).
- `Mix::process()` chain: Sum → `eq.process()` → `compressor.process()` → Master Gain → Limiter.
- Realtime safety preserved: no allocation, no I/O in process path.
- 5 new tests: `test_mix_eq_boosts_at_freq`, `test_mix_compressor_reduces_loud`, `test_mix_eq_passthrough_when_disabled`, `test_mix_compressor_passthrough_when_disabled`, `test_mix_chain_order`.

#### Admin CLI (`server/admin-cli/`)
- Binary crate `open-iem-admin` added to workspace.
- Commands: `user list`, `user create --name --role`, `user delete --id`, `session list`, `session revoke --token`, `health`.
- Auth: `--token <JWT>` or `OPEN_IEM_ADMIN_TOKEN` env var.
- Output: human-readable table (default) or `--json`.
- Graceful 404 (Phase 9 server routes not yet implemented).
- `cargo clippy -D warnings` clean.

#### Musician Guide PDF (`docs/guides/MUSICIANS-GUIDE.pdf`)
- Generated via pandoc 3.1.11.1 + xelatex from `MUSICIANS-GUIDE.md`.
- 61 KB, PDF 1.5, table of contents.
- Emoji characters (⚠, ✅, ❌, 🔴) render as blank in lmroman font — cosmetic only, text content complete.

### Test Results

| Crate | Tests |
|-------|-------|
| mix-engine | 79 |
| api-server (integration) | 19 |
| audio-engine | 20 |
| control-server | 13 |
| control-protocol | 7 |
| doc-tests | 3 |
| **Total** | **157** |

### Not Yet Done (Phase 9 targets)
- Admin API server-side routes (`/api/v1/admin/users`, `/api/v1/admin/sessions`).
- Biquad coefficient validation vs scipy reference.
- ARM64 CI cross-build improvement.

---

## 2026-09-09 — Phase 7: Biquad EQ + RMS Compressor DSP

**Branch:** `feat/phase7-dsp` → squash-merge pending  
**Tests:** 152 passed (↑ from 135), 0 failed

### Implemented

#### Biquad Parametric EQ (`mix-engine/src/eq.rs`)
- Replaced Phase 2 passthrough stub with real Type-II Transposed Direct Form II (TDF2) peaking biquad filter.
- Coefficients follow Audio EQ Cookbook (RBJ): b0/b1/b2/a1/a2 for peaking filter at configured frequency/gain/Q.
- `SAMPLE_RATE = 48_000.0` constant; coefficients recomputed on `set_band`.
- `BiquadCoeffs { b0, b1, b2, a1, a2 }` — `identity()` and `peaking(frequency_hz, gain_db, q)`.
- `BiquadState { w1_l, w2_l, w1_r, w2_r }` — stereo delay lines inline, zero heap allocation.
- `ParametricEq::process(&mut self, l, r) -> (f32, f32)` — applies all enabled bands sequentially.
- API surface maintained: `bands: [EqBand; 4]`, `revision`, `set_band(index, band)`.
- Breaking change: `process` now `&mut self` (state mutation required for biquad).

#### RMS Compressor (`mix-engine/src/compressor.rs`)
- Replaced Phase 2 passthrough stub with stereo-linked RMS detector + smoothed gain reduction.
- Stereo link: detector uses `max(|L|, |R|)`.
- RMS: exp-moving-average of x² using configurable attack/release coefficients.
- Gain reduction: `(1 - 1/ratio) * (threshold_db - rms_db)` when RMS above threshold.
- Smoothed envelope: separate attack/release on gain_reduction_db.
- New mutators: `set_threshold`, `set_ratio`, `set_attack_ms`, `set_release_ms` — all bump revision.
- State inline: `rms_state: f32`, `gain_db: f32` — no heap allocation.
- Disabled: passthrough.

#### Documentation & Infrastructure
- `docker-compose.yml` — dev environment with api-server, musician-ui, engineer-ui.
- `docs/guides/MUSICIANS-GUIDE.md` — 14-section guide in pt-BR (server setup, login, mix control, WebRTC status, permissions, diagnostics, LAN, security, limitations).
- `docs/reviews/PHASE-6-REVIEW.md` — Phase 6 review (was missing).
- `docs/reviews/PHASE-7-REVIEW.md` — Phase 7 review.
- CHANGELOG, TODO, DEVELOPMENT-LOG updated.
- `cargo fmt --all` applied.

### Test Results

| Crate | Tests |
|-------|-------|
| mix-engine | 74 |
| api-server (integration) | 19 |
| audio-engine | 20 |
| control-server | 13 |
| control-protocol | 7 |
| doc-tests | 3 |
| **Total** | **152** |

### Not Yet Done (Phase 8 targets)
- Integrate EQ + Compressor into `Mix::process` audio chain.
- Admin CLI for user management.
- Musician Guide PDF generation.
- EQ coefficient validation vs reference implementation (scipy).

---

## 2026-09-08 — Phase 6: Real trickle-ICE injection + HTTP integration tests

**Branch:** `feat/phase6-trickle-ice`

### Implemented

- **streaming crate:** `add_ice_candidate` now performs real RFC 5245 candidate injection via `Candidate::from_sdp_string` + `Rtc::add_remote_candidate` (str0m). Previous stub only checked session existence. SIMULATED on VPS; no `poll_output` I/O loop runs until Raspberry Pi hardware.
- **streaming crate:** Named constants `MAX_SDP_BYTES`, `MAX_CANDIDATE_BYTES`, `MAX_USER_ID_BYTES` replace inline magic numbers.
- **streaming crate:** 4 new unit tests: `malformed_candidate_rejected_before_session_lookup`, `oversized_candidate_rejected`, `valid_candidate_injected_after_offer`, `candidate_rejected_when_no_session`.
- **api-server:** Full HTTP integration test suite added (`server/api-server/tests/integration.rs`) — 19 tests covering health, auth, RBAC (Musician/Engineer/Admin), CSRF/Origin, audio routes, channel controls.
- **api-server:** Test fixture role casing fixed (`"Musician"` → `"MUSICIAN"`, SCREAMING_SNAKE_CASE); previous fixtures caused 422 instead of testing actual role enforcement.
- **api-server:** `axum-test` pinned to `"21"` (was `"16"`, resolved to 21.1.0). `jsonwebtoken` gains `rust_crypto` feature.

**Verification:** `cargo test --workspace` — 135 tests passed, 0 failed. Independent reviewer: `passed=true`, no security concerns, no logic errors.

---

## 2026-09-08 — Phase 5: Audio transport signaling scaffold

**Branch:** `feat/phase5-audio-transport`

### Implemented

- Accepted ADR-004: WebRTC primary transport; RTP/UDP reserved for dedicated receivers.
- Added `server/streaming` Rust crate using `str0m 0.23`.
- Added bounded SDP offer validation and per-musician WebRTC session registry.
- Added bounded ICE candidate validation and session lifecycle/list operations.
- Defined SIMULATED 48 kHz stereo, 20 ms silence-frame contract for VPS development.
- Added Phase 5 specification and review.

**Verification:** `cargo fmt --all`, `cargo test --workspace` (122 passed), and `cargo clippy --workspace --all-targets -- -D warnings` pass. No Raspberry Pi hardware available on VPS; PipeWire and real Opus media remain pending.

---

## 2026-09-08 — Phase 5: Authenticated audio signaling routes

**Branch:** `main` (working tree delivery; commit pending)

### Implemented

- Wired `server/streaming` into `api-server` application state.
- Added authenticated `POST /api/v1/audio/offer` for SDP negotiation.
- Added authenticated `POST /api/v1/audio/ice-candidate` for bounded ICE signaling.
- Added engineer-only `GET /api/v1/audio/sessions`.
- Added `SessionInfo` JSON serialization and bounded `mix_id` validation.
- Mapped SDP parser failures to generic client errors.

**Verification:** `cargo fmt --all -- --check`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` pass. Independent review initially found input/error-boundary gaps; fixes applied and re-review passed. VPS remains **SIMULATED**: no Raspberry Pi, PipeWire, or real Opus path.

---

## 2026-09-08 — Phase 4: Musician PWA and Engineer scaffold

**Branch:** `feat/phase4-frontend-init` → squash-merged to `main` as PR #3
**Commit:** `b8ca865`

### Implemented

#### Musician PWA (`web/musician/`)
- Vite 5 + React 18 + TypeScript strict, mobile-first CSS modules
- `src/api/auth.ts`: login/refresh/logout via fetch with `credentials:'include'`
- `src/hooks/useWebSocket.ts`: WS hook connecting to `/ws/v1?token=<jwt>` (browser WS cannot set headers — known limitation, short TTL + LAN mitigation)
- `src/components/Login.tsx`: form, in-memory token storage (never localStorage)
- `src/components/Channel.tsx`: gain slider (−60 to +6 dBFS, step 0.5), mute toggle
- `src/components/MixControl.tsx`: 8 channels, master volume, logout
- `src/components/ConnectionStatus.tsx`: status dot + revision display
- `src/protocol/types.ts + envelope.ts`: matches server `control-protocol` wire format
- PWA manifest + SVG icons (192px, 512px)
- 25 vitest tests passing, typecheck clean, build: 150 KB JS (48 KB gzip)

#### Engineer Scaffold (`web/engineer/`)
- Phase 6 placeholder page
- 2 vitest tests passing, typecheck clean, build: 143 KB JS

### Security review
- Independent reviewer: **passed** (0 security, 0 logic errors)
- 3 non-blocking suggestions logged (master volume wire-up Phase 6, WS reconnect design, log scrubbing for Pi)

### Post-merge test results
- web/musician: 25/25 tests passing
- web/engineer: 2/2 tests passing

---

## 2026-09-08 — Phase 3: Security boundary hardening — follow-up

**Branch:** `fix/phase3-auth-boundaries`

- Added Origin validation for browser state-changing requests and rejected cookie-authenticated requests without Origin.
- Bounded login and admin user-creation username/password inputs before Argon2 work.
- Refresh rotation now revokes old token and inserts replacement in one SQLite transaction.
- API defaults to loopback HTTP; non-loopback insecure bind fails unless explicit isolated-development override is set.
- WebSocket enforces role checks, 16 KiB messages, 120 messages/minute, and closes at JWT expiry.
- Global HTTP body limit set to 16 KiB.
- TLS reverse-proxy deployment documented in `docs/deployment/TLS.md`.

**Test results:** `cargo fmt --all`: PASS; `cargo test --workspace`: 99 tests passed; `cargo clippy --workspace --all-targets -- -D warnings`: PASS.

---

## 2026-09-08 — Phase 3: Security boundary hardening

**Branch:** `fix/phase3-auth-boundaries`

- Independent security review found unresolved BLOCKER/HIGH gaps: plaintext HTTP, unauthorised WebSocket mutations, missing token expiry/revocation on sockets, non-atomic refresh rotation, missing CSRF/origin validation, and unbounded auth inputs.
- Fixed refresh expiry boundary (`now >= expires`), explicit `OsRng` refresh-token generation, bounded WebSocket message/request IDs, and REST gain range validation.
- Remaining BLOCKER/HIGH findings are tracked in `docs/TODO.md`; no merge is allowed until resolved.

**Test results:** `cargo fmt --all -- --check`: PASS; `cargo test --workspace`: 99 tests passed; `cargo clippy --workspace --all-targets -- -D warnings`: PASS.

---

## 2026-09-08 — Phase 3: Control Server State Dispatcher

**Branch:** `feat/phase3-control-protocol`

- Added `server/control-server/`, a tested control-plane consumer of `control-protocol` and `mix-engine`.
- Implemented revision-aware dispatch for `GetState`, `SetChannelGain`, and `SetChannelMute`.
- Invalid channel indexes and non-finite gains fail without state mutation.
- Accepted ADR-008: Ed25519 JWT access tokens, rotating opaque refresh tokens, Argon2id passwords, HTTPS-only cookies/WebSocket upgrades. Authentication implementation remains pending.

**Test results:** `cargo test --workspace`: 89 tests passed (86 unit + 3 doc tests); `cargo clippy --workspace --all-targets -- -D warnings`: PASS; `cargo fmt --all -- --check`: PASS.

---

## 2026-09-08 — Phase 3: Versioned Control Protocol Foundation

**Branch:** `feat/phase3-control-protocol`

- Added `server/control-protocol/` crate with JSON/Serde envelope, protocol version validation, role catalog, and typed client/server message catalog.
- Added round-trip and unsupported-version tests.
- Control protocol is control-plane only; it does not touch realtime audio path.

**Test results:** `cargo test --workspace`: 82 tests passed (20 audio-engine, 57 mix-engine, 2 control-protocol, 3 doc tests); `cargo clippy --workspace --all-targets -- -D warnings`: PASS.

---

## 2026-09-08 — Phase 0: Bootstrap & Specification Audit

### Status: IN PROGRESS

### Actions

- Read and analyzed `START.md` (2135 lines — full engineering master prompt)
- Verified GitHub remote: `https://github.com/rickslamaral/open-iem-platform`
- Pulled existing commit (initial commit with minimal README)
- Environment audit: Ubuntu 25.10, x86_64, 4 CPUs, 15 GiB RAM
- Installed Rust 1.98.1 via rustup
- Created full project directory structure
- Created 11 project-specific agent skills in `.agents/skills/`
- Created documentation structure (`docs/` with all subdirectories)
- Created ADR baseline (ADR-001 through ADR-008)
- Created `docs/SPEC-AUDIT.md`
- Created `docs/ARCHITECTURE-GAPS.md`
- Created `docs/SKILLS.md`
- Created `docs/TODO.md`
- Created `docs/DEVELOPMENT-ENVIRONMENT.md`
- Created CI/CD foundation (GitHub Actions)
- Created `scripts/validate-skills.sh`
- Created symlinks `.hermes/skills/` and `.claude/skills/` → `.agents/skills/`
- Created root files: README, CONTRIBUTING, SECURITY, LICENSE, CHANGELOG, .gitignore
- Bootstrap commit to main

### Environment Findings

- PipeWire not available on VPS (expected — audio hardware required for Phase 1 validation)
- Rust not pre-installed — installed via rustup
- Node 22 LTS available
- Docker available

### Next

Phase 0 Review → PASS → Phase 1 (Audio Engine POC)

---

## 2026-09-08 — Phase 1: Audio Engine POC (Research + Rust Workspace)

**Agent:** Autonomous Engineering Agent (cron)
**Branch:** main
**Commit:** (pending — see below)

### Completed

#### Research Documents Created

| Document | Gap Closed | Status |
|----------|-----------|--------|
| `docs/audio/LATENCY-BUDGET.md` | GAP-005 | DEFINED |
| `docs/audio/AUDIO-SLA.md` | GAP-006 | DEFINED |
| `docs/research/pipewire-integration.md` | GAP-003 | CLOSED — pipewire-jack selected for Phase 1 |
| `docs/research/realtime-scheduling.md` | GAP-008 | CLOSED — PipeWire+rtkit managed for Phase 1 |
| `docs/research/audio-transport/EVALUATION.md` | GAP-001 (partial) | PRELIMINARY — WebRTC selected as primary candidate |

#### Rust Workspace Initialized

- `server/Cargo.toml` — workspace root, resolver = "2", workspace lints
- `server/mix-engine/Cargo.toml` — crate definition
- `server/mix-engine/src/lib.rs` — public API, constants, `db_to_linear`, `linear_to_db`, `apply_pan`
- `server/mix-engine/src/channel.rs` — `Channel` struct
- `server/mix-engine/src/mix_send.rs` — `MixSend` struct  
- `server/mix-engine/src/limiter.rs` — `Limiter` stub
- `server/mix-engine/src/mix.rs` — `Mix` struct with `process()`
- `server/mix-engine/src/mix_engine.rs` — `MixEngine` top-level coordinator

#### Key Design Decisions

- No heap allocation in audio path (`core::array::from_fn`, fixed-size arrays)
- No I/O in `process_frame()` — enforced by code structure and `#[deny]` directives
- Revision counter (monotonic `u64`) on every mutable type
- Limiter is a stub (hard-clip) — proper lookahead planned for Phase 2
- `GAIN_DB_MIN = -144.0 dBFS` (practical –∞), `GAIN_DB_MAX = +12.0 dBFS`
- Equal-power (sine/cosine law) pan
- Solo logic evaluated at Mix level, not MixSend level

#### Test Results

```
cargo test: 50 unit tests + 3 doc tests = 53 total — ALL PASS
cargo clippy -- -D warnings: 0 warnings
cargo fmt: applied
skills validate: 11/11 PASS
```

### Gap Status After This Session

| Gap | Before | After |
|-----|--------|-------|
| GAP-001 Audio Transport | UNRESOLVED | PARTIALLY CLOSED |
| GAP-003 PipeWire Integration | UNRESOLVED | CLOSED |
| GAP-005 Latency Budget | UNDEFINED | DEFINED |
| GAP-006 XRUN SLA | UNDEFINED | DEFINED |
| GAP-008 RT Scheduling | UNRESOLVED | CLOSED |

### Not Yet Done (Phase 1 continuation)

- Hardware testing (Raspberry Pi 5 + USB audio) — requires hardware
- PipeWire filter node registration — requires PipeWire on target
- Audio I/O integration — Phase 1b
- ADR-004 final update with benchmark data — Phase 5

### 2026-09-08 — Phase 1 audio-engine integration validated

- Validated `server/audio-engine/` simulated backend and feature-gated JACK bridge.
- `cargo test --workspace`: 73 tests passed (20 audio-engine, 50 mix-engine, 3 doc tests).
- `cargo clippy -p audio-engine`: no warnings.
- Real PipeWire/JACK execution remains **SIMULATED** until Raspberry Pi 5 hardware is available.
- Phase 2 limiter replacement remains pending; current limiter is hard-clip by design.

### 2026-09-08 — Phase 2: Mix Engine complete

**Implemented:**

#### Lookahead Brick-wall Limiter (`mix-engine/src/limiter.rs`)
- Replaced Phase 1 hard-clip stub with proper lookahead limiter
- `LOOKAHEAD_FRAMES = 64` (1.33 ms @ 48 kHz — within IEM latency budget)
- Attack coef = `exp(-1/(48000×0.0005))` (0.5 ms), Release = `exp(-1/(48000×0.1))` (100 ms)
- Brick-wall guaranteed: `envelope_gain` never exceeds `target_gain`
- Zero heap allocation — fixed `[f32; LOOKAHEAD_FRAMES*2]` inline buffer
- `Limiter::process` now `&mut self` (state-mutating); `Mix::process` and `MixEngine::process_frame` updated to `&mut self` accordingly
- `Limiter::reset()` clears buffer and envelope (call on engine restart)

#### Parametric EQ stub (`mix-engine/src/eq.rs`)
- `ParametricEq` with `MAX_EQ_BANDS=4`, `EqBand { frequency_hz, gain_db, q, enabled }`
- `process(&self, l, r) → (l, r)` passthrough; biquad DSP deferred to Phase 7
- Revision counter on every mutation

#### Compressor stub (`mix-engine/src/compressor.rs`)
- `Compressor { threshold_db, ratio, attack_ms, release_ms, enabled }`
- `process(&self, l, r) → (l, r)` passthrough; dynamics DSP deferred to Phase 7
- Safe defaults: -18 dBFS threshold, 4:1 ratio, 10/100 ms attack/release

#### API propagation
- `Mix::process` → `&mut self` (required for mutable limiter)
- `MixEngine::process_frame` → `&mut self`
- All test fixtures updated accordingly

**Test results:** 80 tests, 0 failed, 0 clippy warnings
- audio-engine: 20 tests
- mix-engine: 57 tests (50 original + 7 new limiter tests)
- doc-tests: 3

### 2026-09-09 — Phase 9: Admin API server-side routes + biquad validation

**Implemented:**

#### Admin API Routes (`server/api-server/src/routes/admin.rs`)
- 4 new endpoints, all Admin-role-only via `require_min_role`:
  - `GET /api/v1/admin/users` — list all users (id, username, role)
  - `DELETE /api/v1/admin/users/{id}` — delete user (204) or not found (404)
  - `GET /api/v1/admin/sessions` — list active refresh-token sessions
  - `DELETE /api/v1/admin/sessions/{id}` — revoke session by ID (204/404)
- All routes in protected Router (behind JWT auth middleware)
- `ApiError::NotFound(String)` variant added for 404 responses

#### DB Layer (`server/api-server/src/db.rs`)
- `list_users()` — parameterized SELECT, returns Vec<(i64, String, Role)>
- `delete_user(user_id: i64)` — DELETE, NotFound if affected==0
- `list_active_sessions(now_unix: u64)` — non-revoked, non-expired refresh tokens
- `revoke_session_by_id(session_id: i64)` — UPDATE, NotFound if affected==0
- Sessions cascade-deleted on user delete (existing FK ON DELETE CASCADE)

#### Admin CLI Fix (`server/admin-cli/src/main.rs`)
- `user create --username --password --role` (was `--name`)
- `session revoke --id` (was `--token`, now numeric ID)

#### Testing
- 8 new HTTP integration tests in api-server/tests/integration.rs
- 3 new DB unit tests
- Total: 157 → 168 tests, all green

#### Biquad Validation
- Python RBJ coefficients vs Rust implementation: max delta 5×10⁻⁸ (f32 rounding only)
- Identity check (gain=0): b0=1.0, b1=a1, b2=a2 — PASS in both

#### CI
- Added npm-audit job (HIGH severity gate for musician/engineer frontends)

**Test results:** 168 passed, 0 failed, 0 clippy warnings, code formatted
- admin-cli: 0 tests (binary)
- api-server: 27 tests (was 19, +8 admin integration)
- api-server DB: 16 tests (was 13, +3 admin DB unit)
- mix-engine: 79 tests, audio-engine: 20 tests, control-server: 13 tests, streaming: 7 tests, doc-tests: 3


### 2026-09-09 — Phase 10: mix assignment and ownership enforcement

Implemented SQLite mix assignments, JWT `uid`, Engineer/Admin assignment routes, musician-owned send gain/pan/mute routes, and control-state mix accessors. VPS validation remains SIMULATED for PipeWire/Opus.

Added three HTTP integration tests for assignment, musician ownership denial, and gain validation. Assignment writes now reject slot conflicts without `INSERT OR REPLACE` data loss.

**Verification:** cargo fmt PASS; cargo clippy --all-targets -- -D warnings PASS; cargo test --all PASS: 171 tests, 0 failures.


### 2026-09-09 — Phase 11: versioned release pipeline

Implemented `.github/workflows/release.yml` with 6-stage pipeline: version validation gate, quality gate (fmt+clippy+test+audit), Linux x86_64 build, Linux ARM64 cross-compile (Raspberry Pi 5 target), web frontend builds, and GitHub Release with artefacts + SHA-256 checksums.

Bumped workspace version `0.1.0` → `0.2.0`. Aligned `admin-cli` Cargo manifest to workspace. Added `server/.cargo/config.toml` for ARM64 linker.

**Limitations:** ARM64 cross-compiled but SIMULATED — not tested on real Pi hardware. Windows/macOS UNSUPPORTED (toolchain). PipeWire/Opus remain SIMULATED on VPS.

**Verification:** cargo fmt PASS; cargo clippy PASS; cargo test --all PASS: 171 tests, 0 failures. Static scan clean. Independent reviewer: passed=true.

### 2026-09-09 — Documentation validation hardening

**Goal:** validate required documentation and release metadata on every CI/development round.

**Implemented:** Added `scripts/validate-docs.sh`, checking required Markdown/PDF files, SemVer workspace version, single CHANGELOG `[Unreleased]` section, and non-empty Musician Guide PDF. Removed duplicate `[Unreleased]` heading from CHANGELOG.

**Tests:** Documentation validator PASS (`version 0.2.0`); `git diff --check` PASS. Rust and frontend tests remain green from this run.

**Limitations:** PDF content extraction and rendering remain outside this lightweight validator. PipeWire/Opus and ARM64 runtime remain SIMULATED.

**Next Step:** Add validator to CI, then continue HTTP/WebSocket integration hardening.

### 2026-09-09 — Phase 16: WebSocket send mutations + musician ownership enforcement

**Implemented:**

#### Protocol (control-protocol)
- `SetSendGain { mix_index: u8, channel_index: u8, gain_db: f32 }` — musician/engineer can set per-channel gain on a mix send
- `SetSendPan { mix_index: u8, channel_index: u8, pan: f32 }` — pan ±1.0
- `SetSendMuted { mix_index: u8, channel_index: u8, muted: bool }` — mute toggle
- `SendAck { mix_index, channel_index, gain_db, pan, muted, revision }` — authoritative echo of post-mutation state
- `Eq` derive removed from `ServerMessage` (f32 fields incompatible)

#### Dispatch (control-server)
- `SetSendGain`: validates `gain_db.is_finite()` and `GAIN_DB_MIN..=GAIN_DB_MAX`, else `INVALID_GAIN`
- `SetSendPan`: validates `pan.is_finite()` and `-1.0..=1.0`, else `INVALID_PAN`
- `SetSendMuted`: dispatches directly (bool, no range check needed)
- Missing mix → `MIX_NOT_FOUND`; mix-engine error → `SEND_ERROR`
- `dispatch` doc: ownership check is caller responsibility

#### WebSocket handler (ws.rs)
- `check_permission()`: Musician may `GetState`, `SetSendGain`, `SetSendPan`, `SetSendMuted`; blocked from `SetChannelGain`/`SetChannelMute`
- `send_mix_index()`: extracts `mix_index` from send mutations; `None` for non-send messages
- Musician ownership gate: `db.get_user_assigned_mix(user_id)` → denied if != requested `mix_index`
- Lock scope: held only during `dispatch`, released before every `await`
- `#[allow(clippy::too_many_lines)]` on `handle_socket`

#### channels.rs
- Exhaustive match: added `SendAck => ApiError::Internal` arm in `set_channel_gain` and `set_channel_mute`

**Tests:** 17 new tests
- control-protocol: 4 round-trip serde tests
- control-server: 6 unit tests (ack on success, NaN gain, out-of-range pan, missing mix)
- api-server integration: 7 WS tests (engineer gain/pan/mute ack; musician denied unassigned; musician allowed assigned; musician denied channel mutation; engineer out-of-range gain)

**Verification:** cargo fmt PASS; cargo clippy --all-targets -D warnings PASS; cargo test --all PASS. Static scan clean. Independent reviewer: passed=true.
**CI:** PR #30; fmt failure on first push (method chain not collapsed) fixed on second push.

**Limitations:** PipeWire/Opus remains SIMULATED. ARM64 not hardware-validated.

**Next:** Broadcast state delta on send mutations to all connected clients; HTTPS/TLS gate; add NaN-specific WS test.


### 2026-09-09 — Phase 22: TLS deployment configuration + env var correction

**Goal:** Close HIGH security gate from Phase 3 (HTTPS/TLS fail-closed transport). Correct critical docker-compose.yml env var mismatch that broke dev compose.

**Implemented:**

#### docker-compose.yml
- Renamed all env vars from legacy `OPEN_IEM_*` prefix to `OPENIEM_*` to match api-server binary.
- Fixed healthcheck URL to `/api/v1/health`.
- Renamed volume mount from `./secrets` to `./keys`.
- Added explicit WARNING comment: file is dev-only, production requires Caddy TLS.

#### main.rs (api-server)
- Added startup `tracing::warn!` for each detected legacy `OPEN_IEM_*` env var, preventing silent misconfiguration.
- `#[allow(clippy::too_many_lines)]` added to keep `main` under clippy lint budget.

#### deployment/caddy/Caddyfile (new)
- LAN TLS via mkcert certificate (`auto_https off`).
- `reverse_proxy 127.0.0.1:8080` with IP forwarding headers.
- HTTP → HTTPS permanent redirect.

#### deployment/systemd/openiem-server.service (new)
- Dedicated `openiem` user, loopback bind, no `OPENIEM_ALLOW_INSECURE_HTTP` (fail-closed).
- Hardening: `NoNewPrivileges`, `PrivateTmp`, `ProtectSystem=strict`, `ProtectHome`, `LimitNOFILE=65536`.

#### deployment/raspberry-pi/README.md (new)
- Full RPi 5 deployment guide: binary, keys, systemd, Caddy+mkcert, CA trust per OS.
- Security-critical: mkcert CA key protection (chmod 600, no unencrypted backup, revocation procedure, 2yr 3mo expiry check).

#### docs/adr/ADR-011-tls-deployment.md (new)
- TLS-at-proxy decision, consequences, alternatives rejected.

#### docs/reviews/PHASE-22-REVIEW.md (new)
- Security findings addressed: HIGH-01 (rootCA.key protection), MED-01 (compose warning), MED-02 (legacy var warning in binary), MED-03 (cert expiry doc).

**Security review findings (all addressed):**
- HIGH-01: rootCA.key chmod 600 + revocation doc added to RPi README.
- MED-01: WARNING comment in docker-compose.yml.
- MED-02: startup tracing::warn! in main.rs for legacy OPEN_IEM_* vars.
- MED-03: mkcert cert expiry documented.

**Closes:** HIGH gate from Phase 3 security follow-up.

**Verification:** cargo fmt PASS; cargo clippy --all-targets -D warnings PASS; cargo test --workspace PASS: 199 tests, 0 failures. Static scan clean. Independent code review: passed=true.

**Limitations:** PipeWire/ALSA SIMULATED. ARM64 not hardware-validated. Caddy log sanitization for Sec-WebSocket-Protocol is operator-configurable (LOW finding, documented).

**Next:** Authorize every WebSocket message by role (Phase 3 HIGH remaining); add code coverage reporting; tag v0.3.0.

---

### 2026-09-09 — Release 0.3.0 version synchronization

**Goal:** Corrigir falha real do pipeline: tag `v0.3.0` não correspondia às versões do workspace Rust e dos dois frontends.

**Implemented:** Sincronizada versão `0.3.0` em `server/Cargo.toml`, `web/musician/package.json`, `web/musician/package-lock.json`, `web/engineer/package.json` e `web/engineer/package-lock.json`. README atualizado para Phase 23.

**Tests:** Skill validation PASS; `cargo fmt --all -- --check` PASS; `cargo clippy --all-targets -- -D warnings` PASS; `cargo test --all` PASS (215 testes); ambos frontends: `npm ci`, typecheck, testes (34 total) e build PASS.

**Problems:** GitHub CI e Release anteriores falharam no primeiro job por inconsistência `v0.3.0`/`0.2.0`. Logs detalhados indisponíveis via token GitHub, mas estado e validação local confirmam causa no gate de versão.

**Next:** Reapontar tag `v0.3.0` para commit sincronizado e verificar CI/Release reais.

---

### 2026-09-09 — Phase 23: WebSocket master gain/mute control with RBAC and broadcast

**Goal:** Complete WebSocket message authorization (Phase 3 HIGH); add SetMasterGain/SetMasterMute commands with MasterAck response and broadcast fan-out.

**Implemented:**

#### control-protocol/src/lib.rs
- `SetMasterGain { mix_index: u8, gain_db: f32 }` and `SetMasterMute { mix_index: u8, muted: bool }` added to `ClientMessage`.
- `MasterAck { mix_index, master_gain_db, master_muted, revision }` added to `ServerMessage`.
- Round-trip serde tests for both new messages and MasterAck.

#### control-server/src/lib.rs
- `SetMasterGain` dispatch: validates gain_db finite + in [GAIN_DB_MIN, GAIN_DB_MAX]; calls `mix.set_master_gain_db()`; returns `MasterAck`.
- `SetMasterMute` dispatch: calls `mix.set_master_muted()`; returns `MasterAck`.
- Unit tests: gain/mute ack on success, INVALID_GAIN for bad gain, MIX_NOT_FOUND for invalid index.

#### api-server/src/state.rs
- `MasterDelta` struct added (parallel to `SendDelta`).
- `master_event_tx: broadcast::Sender<MasterDelta>` added to `AppState` (capacity 256).

#### api-server/src/ws.rs
- `check_permission`: Musician role explicitly blocked from `SetMasterGain` and `SetMasterMute`.
- `master_mix_index()` helper extracts mix index from master mutation messages.
- `handle_socket`: subscribes `master_event_rx` before loop; 3-branch `tokio::select!` (recv + send delta + master delta).
- Master mutations broadcast `MasterDelta`; fan-out filters: Engineer/Admin see all mixes, Musician sees only assigned mix.
- Originator skipped in broadcast (consistent with send-delta model).

**Tests:**
- control-protocol: 3 round-trip serde tests (SetMasterGain, SetMasterMute, MasterAck).
- control-server: 4 unit tests (gain ack, mute ack, INVALID_GAIN, MIX_NOT_FOUND).
- api-server integration: 7 WS tests (engineer gain/mute ack, musician denied gain/mute, invalid gain, broadcast to other sessions, musician broadcast filter).

**Independent review:** passed=true. Suggestions: DB error logging in fan-out (LOW, documented in TODO); mix_assignment_lock contention in fan-out (LOW, documented in TODO); musician-filtered broadcast test (added).

**Security review findings:** None. All WebSocket messages now have explicit RBAC; closes Phase 3 HIGH item.

**Verification:** cargo fmt PASS; cargo clippy --all-targets -D warnings PASS; cargo test --workspace PASS: 215 tests, 0 failures. Static scan clean. Independent reviewer: passed=true.

**Limitations:** PipeWire/ALSA SIMULATED. ARM64 not hardware-validated.

**Next:** Tag v0.3.0; code coverage reporting; musician-filtered master broadcast test (added this phase); v0.3.0 release notes.

---

## Phase 93 — AGENTS.md e interface Hermes Agent (2026-09-14)

**Contexto:** Os cron jobs de desenvolvimento autônomo (`361e70c8e264` e `7aee82067e22`) usam `workdir=/workspace/open-iem-platform/`. O Hermes Agent carrega `AGENTS.md` automaticamente quando presente no `workdir`, mas o arquivo não existia no repositório — agentes rodavam sem bootstrapping dedicado.

**Implementado:**

- `AGENTS.md` criado na raiz do repositório: interface de bootstrapping para agentes autônomos Hermes. Define missão, ordem de leitura, autonomia vs confirmação, carregamento de credenciais, gates obrigatórios, workflow por fase, estado atual e restrições permanentes.
- `START.md` seção 146 adicionada: documenta propósito, estrutura esperada e contrato de manutenção do `AGENTS.md`.
- `CHANGELOG.md` atualizado: Phase 92 (WS EQ band control) e Phase 93 (AGENTS.md) registradas em `[Unreleased]`.

**Verificação:** Documentação only. Gates Rust e frontend não afetados.

**Pendências herdadas:** CI remoto, release `v0.3.1`, validação Raspberry Pi 5, frontend EQ controls (Phase 93+).

---

## Ciclo autônomo 2026-09-14 — sincronização de status TODO

**Ação:** TODO.md obsoleto — Phase 92 estava com todos os itens como `[ ]` apesar de PR #54 ter sido mergeado em main com implementação completa do backend WS EQ band control.

**Corrigido:**
- Estado atual atualizado: "Phase 83–92 implementadas"
- Phase 92: todos os itens marcados `[x]`; adicionado item pendente para CI remoto e frontend
- Phase 93 adicionada ao backlog: Frontend EQ Controls (Engineer Console)

**Gates:** cargo fmt PASS; cargo clippy PASS; 245 testes PASS (62 integration, 80 mix-engine, 20 control-server, 38 control-protocol, resto).

---

## 2026-09-14 — P0-003 Media Plane (feat/p0-003-media-plane)

**Branch:** feat/p0-003-media-plane
**Commit:** a0b2a49
**Status:** commit local; PR a abrir

### O que foi implementado

- `server/streaming/src/media_plane.rs` (374 linhas):
  - `StreamMetadata` — descriptor versionado por stream: stream_id ("mix_N"), mix_index, revision, sequence, sample_rate=48000, channels=2, frame_duration_ms=20
  - `MediaFrame` — par estéreo (f32, f32) + metadata versionado
  - `MEDIA_QUEUE_CAPACITY = 32` — fila bounded crossbeam por sessão
  - `MediaSession` — por usuário/mix: enfileira frames via try_send; overflow incrementa drop_count sem bloquear
  - `MediaPlane` — roteia FrameOutput para todas sessões ativas; dropped_total AtomicU64 agrega drops
  - `MediaPlaneError::InvalidMixIndex` — mix_index >= MAX_MIXES rejeitado no register
- `server/streaming/Cargo.toml` — deps adicionadas: crossbeam-channel="0.5", mix-engine={path="../mix-engine"}
- `server/streaming/src/lib.rs` — pub mod media_plane + re-exports

### Arquitetura (ADR-007)

- push_frame usa try_send exclusivamente — NUNCA bloqueia
- Fila bounded (32 frames ≈ 640 ms) por sessão; overflow rejeita mais novo
- mix_index validado em register_session; indexação em push_frame_output é segura

### Gates locais

- `cargo fmt --check`: OK
- `cargo clippy --all-targets -D warnings`: OK
- `cargo test -p streaming`: 23/23 OK (10 novos testes de media_plane + 13 existentes)
- `cargo test` (workspace completo): sem regressão
- Revisão independente: passed=true, sem security_concerns, sem logic_errors

### Nível de evidência

SIMULATED — sem hardware, sem PipeWire, sem Opus real. Fronteira de dados correta.

### Próximo

P0-004 — native/headless Opus receiver (depende de P0-003 ✓)


## 2026-09-14 — P0-004 receiver core

Implemented `streaming::opus_receiver`: pure-Rust Opus 48 kHz stereo decoder, bounded non-blocking RTP payload ingress, ordered bounded jitter buffer, headless `AudioOutput` boundary, fail-safe mute on underrun/output failure, and reconnect reset. Evidence remains **SIMULATED**; PipeWire/ALSA device output and physical receiver validation remain pending.
