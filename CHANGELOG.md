# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- Phase 130: cobertura headless determinística para composição bandwidth + duplicate no receiver Opus.
- Teste headless para composição determinística de bandwidth e outage no receiver Opus.
- Phase 129: combined bandwidth/loss fault pipeline drives encoded Opus receiver coverage; evidence remains CODE/CI, with runtime and hardware unvalidated.

### Fixed
- Added variable-payload coverage for deterministic bandwidth byte-budget enforcement.

### Added
- Phase 127: deterministic bandwidth fault profile now drives encoded Opus receiver coverage; evidence remains CODE/CI, with real network, WebRTC/DTLS-SRTP, PipeWire/ALSA and hardware unvalidated.

- Phase 126: `CombinedFaultProfile` accepts deterministic `BandwidthProfile` stages for byte-budget fault pipelines. Evidence remains CODE/CI; real network, WebRTC/DTLS-SRTP, PipeWire/ALSA and hardware remain unvalidated.

### Tests
- Phase 118 adiciona cobertura headless determinística de jitter: pacotes Opus reordenados chegam ao `OpusReceiver` sem perda, PLC, mute ou falhas de saída. Evidência limitada a CODE/CI; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

### Fixed
- Phase 111: `OpusReceiver` contabiliza falhas de decoder, validação de frame, PLC e saída como `output_failures`, mantendo mute fail-safe sem dupla contagem.

### Added
- Receiver fail-safe output failure counter in observability snapshots.
- Receiver `late_packets` counter separates stale/duplicate packets from other drops.

### Tested
- Phase 108 adiciona teste de integração do fluxo OpusReceiver com métricas de pacote descartado e reconnect; evidência permanece CODE/SIMULATED.

### Tests
- Added receiver observability snapshot coverage for populated counters and stable REST field names.

### Observability
- Count invalid receiver ingress payloads as dropped packets.
- Reconciled Phase 106 receiver metrics coverage: ingress, jitter overflow, reconnect and REST snapshot serialization remain CODE evidence only.

### Documentation
- Reconciliado status de observabilidade do receiver: métricas Phase 104/105 estão em CODE, sem claim de runtime enquanto não houver integração com binário headless.

### Fixed
- `OpusReceiver` agora contabiliza pacotes atrasados descartados no jitter buffer, evitando subcontagem em `ReceiverMetrics`.


### Fixed
- Preserve media frames while WebRTC Sans-IO sessions await negotiated audio media.

### Added
- Phase reviews added: PHASE-97, PHASE-98, PHASE-99 and PHASE-100 documenting PipeWire smoke CI, CI reconciliation, virtual graph integration and multi-packet Opus reorder coverage; evidence levels remain CODE/CI or SOFTWARE/SIMULATED as documented per phase.
- Pair and revoke device pairing routes (`POST /api/v1/audio/pairing`, `DELETE /api/v1/audio/pairing/:device_id`) integrated into api-server. CODE+CI evidence (PR #189, CI 16/16 SUCCESS); DTLS-SRTP session binding and runtime remain pending.
- CI audio-lab now runs the PipeWire/WirePlumber userspace smoke gate; evidence remains SOFTWARE/SIMULATED.
- Added multi-packet Opus round-trip coverage with out-of-order arrival, jitter-buffer ordering and per-frame content assertions.
- Deterministic Opus writer/receiver round-trip test and PipeWire/WirePlumber userspace smoke script, both explicitly classified SOFTWARE/SIMULATED.

### Tests
- Adicionado round-trip de snapshot de configuração em estado novo, cobrindo serialização, desserialização e restauração de canal, mix e send.


### Changed
- Stability gate now runs bounded deterministic audio/media regression loops instead of unconditionally reporting blocked.
- Reconciled Phase 97 package status: amd64 and arm64 `.deb` lifecycle gates pass in CI; local arm64 cross-linker availability remains irrelevant to CI artifact evidence.

### Added
- Documentado Phase 96: snapshots locais de configuração via `iem config backup` e `iem config restore`.

### Security
- Limita leitura de snapshots locais `iem config restore` a 1 MiB antes da desserialização.

### Documentation
- Reconciliado o status da Phase 98: smoke PipeWire/WirePlumber agora roda no CI; sink/source virtual, WebRTC E2E e hardware continuam pendentes.
- Add canonical Phase 95 review for observability metrics REST endpoint and clarify CODE/CI versus runtime evidence.

### Security
- Revalida estado de pairing sob lock após Argon2 para bloquear revogação e rotação concorrentes durante autenticação.

### Changed
- Reconciled scene duplication documentation with merged CODE+CI evidence; runtime validation remains pending.

### Documentation
- Atualizado handoff/TODO: P1-002 Device Manager concluído em CODE/CI; hot-plug/runtime e hardware continuam pendentes.
- Sincronizado status público do README e handoff com Audio Lab L1/L2 e fases P0 de mídia já implementadas em CODE/CI; runtime e hardware continuam pendentes.
- Reconciliado `GAP-013` com o Device Manager e a rota de dispositivos já implementados; hot-plug e runtime continuam pendentes.

### Fixed
- SceneStore rejeita ponteiros de revisão ativos inválidos ou sem histórico persistido antes de listar, duplicar ou salvar cenas.


### Fixed
- Fail closed on invalid or overflowing SceneStore revision counters.

### Added
- Engineer Console now blocks built-in preset application on locked channels.
- Scene duplication endpoint and Engineer Console action.


### Changed
- Preset catalog and built-in application now share one server-side allowlist.
- Reconciliado status documental de rotas de cenas e presets com evidência CODE+CI; runtime/hardware continua pendente.
- Engineer Console agora confirma preset aplicado, canal alvo e falhas sem aceitar respostas obsoletas.

### Fixed
- Rejeição antecipada de canal inexistente e cobertura de payload desconhecido na aplicação de presets built-in.
- Aplicação de preset agora rejeita canais bloqueados antes de qualquer mutação no servidor.


### Documentation
- Reconciled GAP-001, GAP-003 and GAP-004 with implemented media, receiver and clock code; runtime and hardware validation remain pending.

### Added
- Engineer Console agora aplica presets built-in em canal selecionado e recarrega estado após sucesso.
- Engineer Console preset catalog now supports authenticated manual refresh with stale-response protection.
- Musician UI now shows authenticated read-only preset catalog.

### Fixed
- Engineer Console now ignores malformed preset entries and safely renders empty state for invalid catalog payloads.

### Changed
- Preset catalog endpoint is available to authenticated Musician, Engineer and Admin roles.
### Security
- Enforced first-access password change for bootstrapped `soundtech` accounts through authenticated `PUT /api/v1/auth/password`.



### Added
- Catálogo somente leitura de presets no Engineer Console via `GET /api/v1/presets`, restrito a Engineer/Admin.
- Engineer Console agora edita configuração de cenas e cria revisões via `PUT /api/v1/scenes/{id}`.

### Tests
- API de cenas agora cobre atualização de revisão e bloqueio de mutação por Musician.

### Added
- Musician UI now lists authenticated scenes and marks active scene read-only.

### Added
- Engineer Console: gerenciamento de cenas via API REST (listar, criar, recuperar e deletar).


### Changed
- P0-003 media-plane status records bounded transport delivery and retry-budget coverage as CODE + CI/SIMULATED; runtime and hardware validation remain pending.

### Fixed
- P0-003 transport adapter now clamps registry dequeue to per-pass budget while preserving bounded retry requeue and drop errors.

### Fixed
- Engineer EQ controls now cancel pending debounce timers on unmount, preventing stale WebSocket mutations after view teardown.
- Bound streaming session metadata and ICE lookup identifiers; preserve bounded transport output and expose Sans-IO poll errors in `DriveReport`; retry-capacity drops fail closed through transport errors.

### Changed
- P0-003 now retains bounded Sans-IO `str0m::Transmit` datagrams for an external socket adapter; no network I/O claim.

### Changed
- P0-003 media session drive coverage merged in PR #119; transport output remains CODE/SIMULATED and deferred to bounded transport ownership.

### Tests
- Cobertura do limite por estágio no drive de mídia WebRTC Sans-IO.

### Added
- Bounded Opus media writer and negotiated Sans-IO WebRTC writer handoff (CODE/SIMULATED).

- Bounded Opus media writer for simulated MixEngine frame handoff.
- Bounded per-session media-frame drain API for the future WebRTC writer (CODE/SIMULATED).

### Added
- P0-003: added bounded simulated WebRTC Sans-IO drive pass; drains MediaBridge with frame/output budgets and reports deferred transmit bytes without network I/O.
- Added bounded `streaming::MediaBridge` to hand off processed `FrameOutput` values to media sessions without blocking; network/WebRTC drive remains pending.
- SceneStore API integration test now reopens durable scenes through a fresh `AppState`, covering runtime state construction and active-pointer persistence.


### Fixed
- P1-015: SQLite schema migration `M001` now records completion in `migrations`, applies once, and upgrades legacy databases that already contain `must_change_password`.
- SceneStore rejeita snapshots com `revision == 0`, preservando invariantes de revisão monotônica.
- Tornada testável a seleção do `SCENE_STORE_PATH`, isolando abertura do `SceneStore` durável sem mutação global de ambiente.

### Tests
- Added focused fresh, reopen, and legacy-schema migration coverage.
- Added file-backed scenes clean-state reopen coverage for durable restore and active pointer.


- SceneStore file-backed persistence selected by `SCENE_STORE_PATH`, with reopen persistence coverage.

### Added
- Added REST integration coverage for atomic scene backup restore, active pointer replacement and Engineer-only authorization.
- P2 SceneStore backup restore: `PUT /api/v1/scenes/backup` validates strict versioned snapshots and atomically replaces durable scenes plus active pointer.
- P2 Scenes REST API: 7 rotas REST com RBAC (GET/POST /api/v1/scenes, GET /api/v1/scenes/active, GET/PUT/DELETE /api/v1/scenes/{id}, POST /api/v1/scenes/{id}/recall); `SceneStore` integrado ao `AppState`; 7 testes de integração (PR #87, CI 13/13)
- P2 SceneManager: `SceneStore` SQLite-backed persistence with immutable revision history, transactional save/recall, and corrupt-payload detection (PR #86, CI 13/13)

### Added
- Strict, versioned `scene-manager` schema foundation for durable Scenes/state-store; SQLite lifecycle remains pending.

### Added — Phase 94: nomes de canais na Musician UI
- Musician UI consome `GET /api/v1/channels` com Bearer token mantido somente em memória.
- Nomes válidos do servidor substituem rótulos padrão; payload inválido mantém fallback local.
- 50 testes frontend musician, typecheck e build aprovados.

### Added — P1-008 domain API routes
- `GET /api/v1/system` publica versão, capacidades e status honesto `SIMULATED`.
- `GET /api/v1/channels` autenticada para `Musician` ou papel superior, com canais configurados e revisão do estado.


### Added — P1-005 network fault profiles
- Novo crate `server/network-fault`: 5 perfis de falha determinísticos L1 SIMULATED: `LossProfile`, `JitterProfile`, `ReorderProfile`, `OutageProfile`, `ReconnectProfile`.
- `ReconnectProfile` integra `recovery::RecoveryRegistry` para verificar restauração de mix ID após reconexão.
- Teste de integração valida que resultados de falha alimentam corretamente `observability::{NetworkMetrics, ReceiverMetrics}`.
- 47 testes verdes; sem I/O, sem async, sem hardware dependency.


### Added — P1-001 topology capability model
- Novo crate `server/topology` com limites de capacidade MVP, `ChannelMode`, source mapping explícito e validação tipada de configuração. PR #65, CI remoto aprovado no run `34894139285`.

### Added — P0-008 ALSA fallback backend
- ALSA explicit fallback backend (`AlsaBackend`, feature `alsa`): PCM open, hw_params, fail-safe mute on device error, XRUN recovery, bounded `stop_flag` with `Release`/`Acquire` ordering for ARM correctness (P0-008, PR #63).

### Added — P0-005 clock drift control
- Capture sample timestamps, bounded drift estimator (±500 ppm) and adaptive resampling ratio (0.9995–1.0005) in `streaming::clock`. Simulation only; hardware clock validation pending.

### Added — Phase 93: AGENTS.md e interface Hermes Agent
- `AGENTS.md` criado na raiz: interface de bootstrapping para agentes Hermes rodando via cron job com `workdir=/workspace/open-iem-platform/`.
- `START.md` seção 146 adicionada: documenta propósito, estrutura e contrato de manutenção do `AGENTS.md`.

### Added — Phase 92: WS EQ band control (mergeado #54)
- `SetEqBand` em `ClientMessage` e `EqBandAck` em `ServerMessage` no control-protocol.
- Validação de band index, frequência 20–20000 Hz, gain −24/+24 dB, Q 0.1–10.0 no control-server.
- `EqBandDelta` struct e broadcast channel (256) no api-server state.
- Fan-out para Engineer/Admin via WebSocket; RBAC bloqueia Musician de receber EqBandAck.
- 4 testes de integração: ack Engineer, deny Musician RBAC, broadcast peer, sem broadcast Musician.
- 204 testes verdes locais; revisão independente aprovada.

### Added — Phase 91: Biquad reference validation + ARM64 cross-CI
- 4 vetores determinísticos RBJ em `server/mix-engine/src/eq.rs` para validação de coeficientes com tolerância `5e-6` em `f32`.
- `scripts/validate_biquad_reference.py` para validação independente fora do Rust.
- CI job `Rust Build (ARM64 cross)` adicionado ao `ci.yml`; cross-compile x86_64→ARM64 validado em CI remoto.

### Added — Phase 90: Browser audio constraint
- ADR-005 atualizado para `Accepted`: WebRTC como caminho de mídia para browser/PWA; WebSocket restrito a controle.
- GAP-001 e GAP-002 marcados como `RESOLVED` (validação runtime ainda pendente).
- `docs/research/browser-audio-constraint/EVALUATION.md`: análise de WebRTC, SDP/ICE over HTTP, WebTransport (adiado) e RTP/UDP puro (rejeitado para PWA).

### Added — Phase 89: Engineer Console channel strip
- Engineer Console renderiza canais de `/api/v1/state` com slider de gain (−144..+12 dB) e botão mute por canal.
- Writes usam endpoints autenticados existentes com debounce 300 ms, optimistic UI e guards contra respostas obsoletas.
- Canais bloqueados desabilitam controles; áudio explicitamente `SIMULATED`.

### Added — Phase 88: Musician UI master gain/mute read-only
- Slider de master gain e botão mute da Musician UI passam a ser `disabled` + `aria-readonly="true"`.
- CSS: `cursor: not-allowed`, `opacity: 0.5`, hint `.readOnlyHint`.
- RBAC Rust já bloqueava `SetMasterGain`/`SetMasterMute` para role Musician desde Phase 23; UX agora consistente.

### Added — Phase 87: Engineer Console WebSocket master gain/mute
- `web/engineer/src/protocol.ts`: tipos WS `SetMasterGain`, `SetMasterMute`, `MasterAck`, `State`, `Error`.
- `web/engineer/src/useEngineerWs.ts`: hook React com reconexão 3 s, guard mounted, handlers MasterAck/State/Error.
- `WsBadge` no header; `MixMasterControl` (slider gain −40..+10 dB + botão mute) por mix.

### Performance — Phase 86: lock-free broadcast fan-out
- Remove `mix_assignment_lock` dos dois caminhos de fan-out somente leitura no WebSocket handler (`ws.rs`).
- Stale-read aceito na janela de transição de assignment; cliente re-sincroniza via REST snapshot.

### Added — Phase 85: Rust code coverage
- `make coverage` target: `cargo-llvm-cov` → `coverage/lcov.info`; cria diretório em checkout limpo.
- CI job `Rust Code Coverage`: instala `cargo-llvm-cov 0.9.1 --locked`, gera LCOV, faz upload do artifact (30 dias).
- Baseline: **81.21% lines, 73.85% functions** (workspace, backend simulado).

### Added — examples/minimal-mix
- `server/mix-engine/examples/minimal_mix.rs`: dois mixes independentes de dois canais com gain/pan/mute/master; assertions self-check.
- `examples/minimal-mix/README.md`: guia com grafo de sinal e uso.

### Changed — Phase 84: release reproducibility
- Cargo `--locked` em todos os checks/builds do workflow de release.
- Archives normalizam ordenação, timestamps e ownership via `SOURCE_DATE_EPOCH`; gate executa dois empacotamentos e compara checksums.

### Fixed — Phase 79: CI OpenSSL 3.5
- Workflow de release usa `openssl pkeyutl -sign -rawin`, compatível com OpenSSL 3.5.

### Fixed — Phase 83: Musician UI pan + master mute indicator
- Slider de pan por canal (−1 a +1) com rótulo L/C/R; desabilitado quando canal mudo.
- Badge `MASTER MUTED` somente leitura quando servidor reporta mudo master.
- `SetSendPan` enviado a cada mudança; sincronização via snapshot WebSocket.

### Security — Phase 82 auditoria do instalador (2026-09-13)
- Auditoria independente encontrou HIGH no build a partir de checkout remoto sem verificação criptográfica de fonte, além de MEDIUM em compatibilidade Node.js e instalação parcial/reexecução de assets.
- Merge e release permanecem bloqueados até pin/verificação de fonte, instalação atômica e validação de runtime.

### Security — Phase 81 hardening do instalador e concorrência (2026-09-13)
- Instalador aceita `--ref` como SHA-1 completo com checkout detached e valida caminhos antes de gerar unit file via `sed`.
- `--dry-run` agora descreve operações de dependências, clone, build, instalação e systemd sem alterar host.
- CI valida pushes em `main`; release serializa execuções concorrentes por tag.
- Testes locais e CI do PR #41 passaram; instalação real, release e Raspberry Pi 5 continuam não validados.

### Fixed — Phase 80 compatibilidade TypeScript 7 (2026-09-13)
- Adicionado `web/engineer/src/vite-env.d.ts` com referência `vite/client`, corrigindo `TS2882` para import lateral de `style.css` no TypeScript 7.
- Typecheck, testes e build locais do Engineer aprovados; CI remoto precisa confirmar.

### Fixed — Phase 79 compatibilidade OpenSSL 3.5 (2026-09-13)
- Helpers de teste Ed25519 usam `-rawin`, exigido por OpenSSL 3.5 para operações de assinatura sem digest.
- Suíte combinada local: 56 testes aprovados; CI remoto precisa confirmar correção.

### Security — Phase 78 rejeição de dados residuais em archives (2026-09-13)
- O validador agora verifica o stream gzip completo após validar e consumir o TAR, rejeitando bytes residuais e streams gzip concatenados não autenticados.
- Testes offline cobrem ambos os casos; CI remoto, release e hardware continuam não validados.

### Security — Phase 77 archive validation order (2026-09-12)
- O validador rejeita nomes, raiz, tipos, duplicatas e membros inesperados antes de consumir payloads, reduzindo custo de CPU/IO em archives inválidos.
- Teste confirma que membro inesperado falha sem chamar consumidor de payload; CI remoto e hardware continuam não validados.

### Changed — Phase 76 CI status refresh (2026-09-12)
- Runs `34724845040` (PR) e `34724842446` (push) falharam antes dos steps; jobs consultados retornaram `runner_id=0` e `steps=[]`. Nenhum teste ou build remoto executou; merge e release continuam bloqueados.

### Security — Phase 76 archive payload validation (2026-09-12)
- O validador consome payload completo de arquivos regulares em chunks limitados e rejeita membros truncados; `tarfile` limita leitura ao tamanho declarado pelo header.
- Teste offline cobre payload de membro truncado; CI remoto, release e hardware continuam não validados.

### Changed — Phase 75 version gate (2026-09-12)
- Adicionado `VERSION` como fonte canônica e `scripts/validate-version.py` para validar SemVer, manifests Rust/frontend e tag de release.
- Release workflow e `make test` usam gate reproduzível; CI remoto continua bloqueado antes dos steps.

### Added — Phase 74 documentação visual das interfaces (2026-09-12)
- Adicionadas imagens documentais geradas do código-fonte para Login, Musician PWA, Engineer Console, Admin CLI/API e controles de mix.
- Adicionado `docs/INTERFACES.md`; imagens não são screenshots de runtime e não alteram claims de suporte.

### Changed — Phase 73 Musician Guide (2026-09-12)
- Guia e PDF alinhados ao fluxo HTTP real de signaling; mídia continua `SIMULATED`.

### Added — Phase 72 HTTP signaling integration (2026-09-12)
- Teste autenticado cobre negociação SDP seguida de trickle ICE nas rotas HTTP reais; áudio permanece `SIMULATED`.

### Security — Phase 71 validated release snapshot (2026-09-12)
- Validação de bundle agora abre diretório e entradas por descritores `O_NOFOLLOW`, copia em chunks limitados e publica staging somente após validação completa.
- O workflow de release consome somente o snapshot validado; limite de 32 entradas reduz pressão de recursos. CI remoto, hardware e release continuam não validados.

### Security — Phase 70 bundle validator hardening (2026-09-12)
- Validador limita arquivo individual a 512 MiB, bundle a 2 GiB e manifesto a 64 KiB antes de leituras sem limite.
- SBOM opcional precisa ser JSON objeto válido; escrita de manifesto usa `O_NOFOLLOW` e descritor regular.
- Testes offline: 45 aprovados. CI remoto, hardware e release continuam não validados.

### Changed — Phase 69 CI status refresh (2026-09-12)
- Runs `34711659175` (PR) e `34711656770` (push) falharam antes da execução dos jobs; no PR, os 10 jobs retornaram `runner_id=0` e `steps=[]`. Nenhum teste ou build remoto executou; merge e release continuam bloqueados.

### Changed — Phase 69 CI status refresh (2026-09-12)
- Runs `34711083274` (PR) e `34711080316` (push) falharam antes dos steps; os 10 jobs do run PR retornaram `runner_id=0` e `steps=[]`. Nenhum teste ou build remoto executou; merge e release continuam bloqueados.

### Security — Phase 69 final release bundle validation (2026-09-12)
- O workflow valida o bundle consolidado antes da publicação: archives server x86_64/ARM64 e web musician/engineer devem corresponder à versão da tag.
- Checksums são recalculados com abertura protegida contra symlink; assinaturas server ausentes/vazias e arquivos inesperados bloqueiam a publicação.
- Jobs de build e validação final exigem `OPENIEM_RELEASE_SIGNING_PUBLIC_KEY_FINGERPRINT`, aceitam somente fingerprint SHA-256 hexadecimal de 64 caracteres e comparam contra chave pública derivada antes de verificar assinaturas.
- Validação gera manifesto imutável com nomes e digests exatos; publicação usa somente caminhos listados no manifesto, sem glob amplo.
- Testes offline: 42 aprovados; CI agora inclui `tests/test_validate_release_bundle.py`. CI remoto, hardware e release continuam não validados.

### Security — Phase 68 Windows archive names and signature input limits (2026-09-12)
- O validador rejeita caracteres inválidos, pontos/espaços finais e nomes reservados Windows em todos os componentes do caminho.
- O verificador rejeita assinatura ou chave pública acima de 64 KiB antes de executar OpenSSL.
- Testes offline: 30 aprovados. CI remoto, hardware e release continuam não validados.

### Security — Phase 67 cross-platform safe archive names (2026-09-12)
- O validador rejeita separadores `\\` e caracteres de controle em nomes de membros antes de qualquer validação estrutural, evitando divergência de interpretação entre consumidores POSIX e Windows.
- Testes offline cobrem separador Windows e caractere de controle.
- CI remoto e hardware continuam não validados.

### Security — Phase 66 canonical archive root validation (2026-09-12)
- O validador rejeita nomes de diretório raiz `.`/`..` ou não canônicos antes de validar conteúdo, mantendo caminhos de release determinísticos.
- Adicionado teste local para raiz não canônica.
- CI remoto e hardware continuam não validados.

### Security — Phase 65 incremental archive resource validation (2026-09-12)
- Limites de tamanho por membro e total descomprimido agora são aplicados durante a iteração do archive; entradas abusivas falham antes do consumo de membros posteriores.
- Adicionado teste offline para rejeição imediata de membro oversized.
- CI remoto e hardware continuam não validados.

### Security — Phase 64 archive validator portability (2026-09-12)
- CLI do validador agora captura ausência de `O_NOFOLLOW` e retorna falha controlada, sem traceback nem comportamento permissivo.
- CI remoto e hardware continuam não validados.


### Changed — CI status after Phase 63 archive verifier portability (2026-09-12)
- Push run `34695497246` e PR run `34695498591` falharam antes dos steps em todos os 10 jobs; nenhum teste remoto executou. Merge e release continuam bloqueados.

### Security — Phase 63 archive verifier portability (2026-09-12)
- Validador de archives agora falha explicitamente quando o sistema não oferece `O_NOFOLLOW`, removendo fallback permissivo que poderia aceitar symlinks.
- Teste offline cobre ausência de `O_NOFOLLOW`.
- CI remoto e hardware continuam não validados.

### Security — Phase 63 follow-up: fail-closed verifier portability and executable resolution (2026-09-12)
- Verificador exige `O_NOFOLLOW` no sistema alvo, usa `O_NONBLOCK` antes de validar arquivo regular e chama `/usr/bin/openssl` sem depender de `PATH` mutável.
- Review independente bloqueou fallback permissivo de symlink e resolução de OpenSSL via `PATH`; correções aplicadas.
- Execução continua suportada/validada somente em Linux com `/usr/bin/openssl`; CI remoto e hardware continuam não validados.

### Security — Phase 63 fail-closed signed-file verification (2026-09-12)
- Verificador Ed25519 abre artifact, assinatura e chave com `O_NOFOLLOW`, valida descritor regular e mantém o mesmo arquivo aberto durante OpenSSL, evitando troca TOCTOU.
- Testes cobrem symlink em artifact e assinatura; erro de execução do OpenSSL falha de forma explícita.
- CI remoto e hardware continuam não validados.

### Security — Phase 62 Python security tests in CI (2026-09-12)
- Workflow CI ganhou job dedicado para executar testes offline dos validadores de archive e assinatura com Python/pytest.
- Execução remota e hardware continuam não validados.

### Security — Phase 61 fail-closed archive input (2026-09-12)
- Validador abre archive com `O_NOFOLLOW`, exige arquivo regular e aplica limite comprimido via `fstat()` no descritor validado.
- Testes cobrem symlink e diretório como entradas rejeitadas.
- CI remoto e hardware continuam não validados.

### Security — Phase 60 incremental archive validation (2026-09-12)
- Validador interrompe leitura ao exceder 32 membros, evitando materialização ilimitada de headers em archives comprimidos.
- Testes cobrem limites de archive comprimido e total descomprimido, além dos limites existentes por membro e contagem.
- CI remoto e hardware continuam não validados.

### Security — Phase 59 archive resource limits (2026-09-12)
- Validador rejeita archives acima de 512 MiB comprimidos, mais de 32 membros, membros acima de 256 MiB ou total descomprimido acima de 512 MiB, limitando consumo durante validação.
- Testes determinísticos cobrem rejeição de contagem e tamanho excessivos; CI remoto e hardware continuam não validados.
- Após push, runs `34674856987` e `34674858746` falharam antes dos steps; nenhum teste remoto executou.

### Security — Phase 58 detached release signature verification (2026-09-12)
- Adicionado verificador local Ed25519 para assinatura detached de artefatos.
- Instalador Raspberry Pi exige `.sig` e chave pública regular instalada por canal independente antes da extração.
- Workflow gera assinaturas detached Ed25519 para archives x86_64 e ARM64 usando secret externo `OPENIEM_RELEASE_SIGNING_KEY_PEM`; arquivos `.sig` são publicados com archives e checksums.
- Secret ausente, chave não-Ed25519 ou assinatura vazia interrompem job; verificador também rejeita chave pública não-Ed25519.
- Guia exige chave pública root-owned com modo 0600/0644 e fingerprint SHA-256 autenticado independentemente; CI remoto e hardware continuam não validados.

### Security — Phase 57 deployment archive validation (2026-09-11)
- O instalador Raspberry Pi agora usa `scripts/validate-release-archive.py` antes da extração, rejeitando caminhos não canônicos, links, arquivos especiais, duplicatas e membros inesperados.
- O serviço systemd define `OPENIEM_ALLOWED_ORIGINS=https://iem.local`; o guia documenta atualização dessa origem quando mDNS não estiver disponível.
- CI remoto continua bloqueado antes dos steps; nenhum hardware Raspberry Pi ou release foi validado.

### Changed — CI status refresh (2026-09-11)
- Runs `34656659602` (PR) e `34656657286` (push) falharam antes dos steps; os jobs terminaram sem `runner_id` executável. Nenhum claim de CI verde, merge ou release foi feito.

### Fixed — Phase 56 Caddyfile temporary-directory lifetime (2026-09-11)
- Mantida área temporária do certificado até concluir cópia e instalação do Caddyfile; limpeza antecipada fazia o bloco documentado falhar antes de `sudo install`.

### Security — Phase 55 deployment path hardening (2026-09-11)
- O guia Raspberry Pi resolve Caddyfile a partir da raiz confiável do clone, rejeita symlink e instala com `install` e modo explícito.
- README agora registra Phase 54 na tabela incremental e marca Phase 53 como validação local concluída; CI remoto continua bloqueado.

### Security — Phase 54 artifact provenance attestation (2026-09-11)
- O workflow de release agora gera attestations Sigstore/GitHub para archives de servidor x86_64 e ARM64 antes do upload.
- Jobs de build recebem somente `id-token: write` e `attestations: write` necessários ao provenance; CI remoto e hardware continuam não validados.

### Changed — Phase 53 CI status refresh (2026-09-11)
- Registrados runs `34645508776` (PR) e `34645504292` (push), ambos falhando antes dos steps; os 9 jobs do PR retornaram `runner_id=0` e `steps=[]`.
- PR #40 e release permanecem bloqueados; nenhum claim de CI verde ou artefato publicado foi adicionado.

### Security — Phase 53 release archive validation
- O workflow de release valida archives de servidor x86_64 e ARM64 antes do checksum/upload, rejeitando traversal, links, arquivos inesperados e binários ausentes.
- Empacotamento web falha quando qualquer `dist/` esperado está ausente; uploads falham quando não encontram arquivos.
- Validador CLI rejeita basenames obrigatórios fora da allowlist; testes determinísticos agora cobrem esse contrato.
- Assinatura independente continua pendente.

### Security — Phase 52 verification follow-up
- Admin CLI não segue redirects HTTP(S), impedindo downgrade TLS e vazamento de Bearer token para destino redirecionado.
- `cargo-audit` foi fixado na versão `0.22.2` nos workflows CI e release.
- Geração de certificado LAN usa diretório temporário e instala arquivos com ownership/modos explícitos; áudio, CI remoto e hardware continuam não validados.

### Security — Phase 51 ARM64 deployment hardening
- Downloads do instalador restringem redirects a HTTPS.
- Diretórios de serviço recebem ownership e modos explícitos; chaves JWT são geradas em diretório temporário e instaladas com permissões restritas.
- Unit systemd é resolvida a partir de clone confiável, validada e instalada com `install`; nenhum claim novo de execução em Raspberry Pi foi feito.

### Security — Phase 50 Raspberry Pi installer hardening
- O fluxo de instalação ARM64 agora falha fechado com `set -euo pipefail`, valida tag SemVer, usa diretório temporário e remove artefatos ao sair.
- Download verifica HTTP, checksum do arquivo exato e rejeita membros de archive com traversal, caminhos absolutos, symlinks ou hard links antes da extração.
- Nenhum claim novo de autenticidade do release, execução ARM64 ou hardware Raspberry Pi foi adicionado.

### Fixed — Phase 49 Raspberry Pi ARM64 installation guide
- Corrigidos nome e caminho do artefato ARM64 no guia de deployment para coincidir com `.github/workflows/release.yml`.
- O comando de instalação agora baixa e valida checksum com `sha256sum --check` antes de extrair e instalar `api-server` dentro do diretório versionado.
- `curl` falha explicitamente em erros HTTP antes da instalação.
- Nenhum artefato foi publicado; release e CI remoto seguem bloqueados por falha pré-steps do runner.

### Security — Phase 48 immutable GitHub Actions pinning
- Fixadas actions de CI/release em commits SHA completos, reduzindo risco de retagging upstream.
- CI remoto continua não validado por falha pré-steps do runner.

### Security — Phase 47 workflow permission and tag validation hardening
- Restringidas permissões padrão de CI e jobs de build/release a `contents: read`.
- Mantida escrita GitHub Release somente no job publicador.
- Corrigido filtro candidato de tags e adicionada validação exata `vX.Y.Z` sem zeros à esquerda antes de qualquer build.
- CI remoto continua não validado por falha pré-steps do runner.

### Fixed — Phase 46 verification follow-up
- Documentado `make test-audio` em `docs/CLI.md`, mantendo contrato CLI alinhado ao Makefile.
- `scripts/validate-pdf.sh` agora aceita somente PDFs existentes sob `docs/guides`, evitando validação fora do escopo do repositório.

### Changed — Phase 46 release runner alignment
- Alinhados seis jobs de `.github/workflows/release.yml` ao `ubuntu-latest` já usado pelo CI.
- A alteração não prova disponibilidade do runner: os runs remotos anteriores falharam antes dos steps com `runner_id=0`; release continua bloqueada até execução real.

### Fixed — Phase 45 GitHub Actions runner label
- Trocado `runs-on: ubuntu-24.04` por `runs-on: ubuntu-latest` em todos os jobs de CI; runs anteriores falharam antes dos steps com `runner_id=0`.
- Os runs `34614028392` e `34614025997` também falharam antes dos steps em todos os jobs com `runner_id=0` e `steps=[]`; CI remoto e release continuam bloqueados.

### Fixed — Phase 44 documentation whitespace gate
- Removido trailing whitespace introduzido em reviews das Phases 37–39 e na matriz de validação de plataforma.
- `git diff --check origin/main...HEAD` passa localmente; CI remoto continua bloqueado antes dos steps por ausência de runner executável.

### CI / Security — Phase 43 verification gates
- Adicionado `server/Cargo.lock` para permitir auditoria reproduzível de dependências Rust.
- CI agora valida documentação, PDF, skills e whitespace.
- ADRs duplicados foram renumerados para preservar identificadores únicos.
- CI remoto continua sem runner executável; nenhuma release foi publicada.

### Tooling — Phase 42 reproducible Musician Guide PDF validation
- `make docs` agora valida extração textual e renderização do Musician Guide PDF.
- `scripts/validate-pdf.sh` usa `pdftotext` ou fallback `mutool`, sem depender de pacote específico no VPS.
- Nenhum claim novo de suporte para PipeWire, WebRTC media ou Raspberry Pi 5.

### Documentation — Phase 41 Musician Guide package refresh
- Atualizada versão/data de `docs/guides/MUSICIANS-GUIDE.md` para refletir estado atual do projeto.
- Regenerado `docs/guides/MUSICIANS-GUIDE.pdf` a partir do guia Markdown; PDF continua limitado ao comportamento real e marca áudio como `SIMULATED`.
- Validação de texto/renderização do PDF ficou bloqueada neste VPS: `pdftotext` não está instalado; geração via Pandoc/XeLaTeX foi executada.

### Changed — Phase 40 Compose rebuild safety
- `make up` agora usa `docker compose up -d --build`, evitando iniciar imagens locais obsoletas.
- Compose continua restrito a desenvolvimento; HTTP e áudio `SIMULATED` não mudaram.

### Changed — Phase 39 Makefile audio harness coverage
- `make test` now runs the deterministic `SIMULATED` `audio-engine` integration harness.
- Added standalone `make test-audio` target for focused audio verification.
- No hardware, PipeWire, WebRTC media, Raspberry Pi or remote CI support claim changed.

### Changed — Phase 38 CLI documentation consistency
- Corrected `docs/CLI.md` and development history to reflect implemented `iem` behavior, version `0.3.1`, fixed command set and current limits.

### Added — Phase 37 platform validation matrix
- Added `docs/validation/PLATFORM-VALIDATION-MATRIX.md` with evidence boundaries for VPS Linux, Docker Compose, Raspberry Pi 5 ARM64, Windows Docker Desktop, native Windows audio and future macOS work.
- Hardware, realtime audio, WebRTC media and ARM64 runtime remain `PENDING` or `SIMULATED` until executed on target hardware.

### Changed — Phase 36 Engineer Console telemetry
- Engineer Console now reads `/api/v1/telemetry` and displays backend plus XRUN count.
- Unknown metrics remain `UNKNOWN`; VPS audio remains `SIMULATED`.
- Local frontend tests pass; remote CI remains blocked before workflow steps.

### Added — Phase 35 developer CLI `iem`
- Added `iem` binary dispatching fixed `help`, `status`, `diagnostics`, `docs`, `test`, `build`, `up` and `down` commands to existing Makefile targets.
- Preserves child exit codes and rejects arbitrary shell arguments by construction.
- No global installation, release package or `iem run` support is provided.
- Local verification: admin-cli tests, clippy, formatting, docs/skills validation and `iem --help` pass; remote CI remains blocked before steps.

### Changed — Phase 34 Admin CLI output correctness
- `open-iem-admin` accepts HTTP only for localhost and requires HTTPS for remote server URLs, preventing Bearer token transmission over cleartext networks.
- JSON table output now includes union of fields across all object rows instead of dropping fields absent from first row.
- HTTP 404 errors now report generic resource-not-found status without stale implementation claims.
- Local verification: admin-cli tests and clippy pass; remote CI remains blocked before workflow steps.

### Changed — Phase 33 CI branch trigger
- Added `feat/**` to CI push triggers. Current development branch is `feat/phase24-ws-resilience`; prior workflow matched `feature/**` but not `feat/**`.
- Remote runner remains blocked before workflow steps (`steps=[]`); this change does not claim CI success.

### Tests — Phase 32 deterministic audio harness
- Added `server/audio-engine/tests/deterministic_harness.rs` as a deterministic `SIMULATED` audio-engine integration harness.
- Harness covers determinism, mix isolation, gain, pan, mute, limiter bounds and finite samples.
- Local verification passes: `cargo fmt --all -- --check` and `cargo test -p audio-engine` — 20 unit tests, 4 integration tests and doc-tests.
- Harness does not cover hardware, realtime performance or stop/start. Remote CI remains blocked before steps; PR #40 is open; no merge or release.

### Changed — Compose compatibility cleanup
- Removed obsolete top-level `version` field from `docker-compose.yml`; modern Compose no longer emits the compatibility warning.
- Compose remains development-only and runtime validation remains pending.

### Security — Phase 31 JWT post-issuance revocation (PR #40)
- Added persistent access-session mappings tied to refresh sessions, with middleware validation of JWT ID, user, session, expiry, user existence and revocation state.
- Fixed refresh failure rollback so it discards replacement state without resurrecting a concurrently revoked refresh session.
- Resolved refresh owner before rotation, preventing a missing user lookup from consuming a valid refresh token.
- Redacted internal error details from HTTP response bodies.
- Musician client now retains delayed REST snapshot as baseline when WebSocket revision already advanced before initial snapshot.
- Refresh rotation revokes old access mappings; logout, admin session revoke, replay detection and user deletion invalidate related access mappings.
- Established WebSocket connections re-check access-session state on inbound messages and keepalive ticks, returning `SESSION_REVOKED` before closing.
- Local verification: `cargo fmt --all -- --check`, `cargo test -p api-server --test integration` (57 passed), and `cargo clippy -p api-server --all-targets -- -D warnings` pass. Workspace test/clippy are blocked by missing `jack.pc`; CI remains blocked and changes are not merged or released.

### Changed — Phase 30 Docker Compose development
- Added development Dockerfiles for `api-server`, Musician PWA and Engineer UI.
- Removed stale Compose build target that did not exist in the development image.
- Compose remains development-only: HTTP is explicitly insecure, JWT keys are runtime-mounted, and audio is SIMULATED.

### Documentation
- Added `docs/guides/WINDOWS-DOCKER-GUIDE.md` with Windows 10/11 + Docker Desktop prerequisites, JWT key generation, planned local smoke test, cleanup, troubleshooting and explicit audio validation limits.
- Documented Compose as development-only; Docker runtime and Windows support remain unvalidated, with no runtime support claim.

### Security — Failed WebSocket authentication limiting
- Added bounded in-process per-peer-IP limiting for failed `/ws/v1` authentication: 5 failures per 60-second window and at most 4,096 retained IP entries.
- Successful authenticated WebSocket upgrades do not consume failure budget; peer IP comes only from `ConnectInfo<SocketAddr>`.
- Blocked attempts return HTTP 429 with `Retry-After: 5`.

### Security — WebSocket admission quotas
- Added atomic in-process quotas of 4 upgraded connections per authenticated user and 16 per peer IP, alongside the global limit of 64.
- Quota reservations use RAII release when WebSocket handlers end; quota rejections return HTTP 429 with `Retry-After: 5`.
- Peer identity comes from `ConnectInfo<SocketAddr>`; forwarded headers are not trusted.

### Tests — WebSocket keepalive loop
- Added deterministic paused-clock coverage for the 30-second Ping interval and 60-second Pong timeout boundary.

### Fixed — WebSocket state recovery race
- Musician now records accepted revisions before applying ACK state, preventing a delayed initial REST snapshot from regressing the revision observed over WebSocket.

### Fixed — WebSocket state recovery
- Broadcast lag now emits an authoritative `State` revision notice; Musician refetches authenticated REST snapshot to recover missed deltas.
- Musician client now validates and applies `MasterAck` broadcasts to its local mix snapshot.

### Tests — WebSocket frame limits
- Added integration coverage proving oversized Text messages terminate transport before application parsing.

### Security — WebSocket frame limits
- Applied the 16 KiB protocol limit during WebSocket upgrade for both messages and frames, preventing oversized payload allocation before application validation.

### Fixed — WebSocket keepalive state machine
- Keepalive timeout now applies only while a Pong challenge is outstanding; a valid Pong cannot cause a later false timeout before the next Ping.

### Security — WebSocket keepalive challenge correlation
- Server now accepts Pong for liveness only when payload matches outstanding server Ping; unsolicited or stale Pong frames cannot bypass timeout.

### Changed — WebSocket error correlation
- Errors returned after successful envelope validation preserve originating `request_id`; parser failures continue using `server`.

### Security — WebSocket error redaction
- WebSocket protocol failures now expose stable generic codes/messages; parser and transport details stay server-side.
- Normal peer closes and receive failures no longer masquerade as JWT expiration.

### Security — WebSocket connection cap
- Added a process-wide semaphore limiting upgraded WebSocket connections to 64; excess upgrades return HTTP 503 with `Retry-After: 5`.

### Changed — WebSocket keepalive
- Extracted keepalive intervals and timeout into named policy constants; added boundary unit tests for timeout evaluation.

### Added — Local developer interface
- Added root `Makefile` with documented build, test, lint, docs, validation, diagnostics and lifecycle targets.
- Added `scripts/validate-environment.sh` with explicit `OK`, `OPTIONAL`, `SIMULATED` and `HARDWARE VALIDATION REQUIRED` states.

### Changed — Engineering contract
- Extended `START.md` with CI diagnostics, release blocking, recovery, hardware validation, traceability, audio test harness and architecture fitness requirements.
- Clarified `make run-local` as local API execution with simulated audio; it does not bypass server configuration.
- Docker Compose diagnostics now verify the Compose plugin or standalone executable instead of inferring support from Docker alone.

### Security — WebSocket protocol
- Binary WebSocket frames now receive `INVALID_MESSAGE` and close the connection instead of being silently discarded.

### Added — Phase 24 WebSocket resilience
- Added integration coverage for client Ping/Pong payload preservation and binary-frame rejection.
- WebSocket rate limiting counts every inbound frame, including control frames, preventing Ping/Pong floods from bypassing the per-connection quota.
- Added fail-closed integration coverage when the musician ownership database lookup fails.
- Added server-initiated WebSocket Ping every 30 seconds and `CONNECTION_TIMEOUT` after 60 seconds without Pong.
- Bounded every WebSocket send operation to 10 seconds, preventing stalled clients from retaining handler tasks indefinitely.

### Fixed — Phase 24 audit follow-up
- Stabilized WebSocket broadcast integration tests by yielding after observer handshake, preventing scheduler-dependent false failures.
- Confirmed local Rust workspace and both frontend quality gates pass; real PipeWire, WebRTC media and Raspberry Pi ARM64 runtime remain unvalidated.

### CI — Phase 24
- ARM64 cross-compilation configuration already contains the Rust target, cross-linker package and linker environment; no workflow change required.
- GitHub-hosted CI remains blocked before steps execute because runners are not allocated; local gates are the verified result.

### Fixed — Phase 23 follow-up
- WebSocket ownership lookup now logs database failures and denies forwarding/mutation instead of silently collapsing errors to `None`.

### Changed — Phase 23 documentation
- Guia do Músico atualizado para refletir estado e controles implementados até Phase 23.

### Fixed — Release 0.3.1 preparation
- Preparada consistência de versão entre manifests Rust, frontends e lockfiles após `v0.3.0` ter apontado para commit anterior à sincronização.
- Próximo tag de release deve ser `v0.3.1`; `v0.3.0` remoto permanece imutável e inválido para o gate atual.

### Changed — Release 0.3.0 preparation
- Sincronizadas versões Rust e frontend com tag `v0.3.0`; pipeline de release agora pode validar consistência.

### Added — Phase 23
- `SetMasterGain { mix_index, gain_db }` WebSocket client message: sets master gain for a mix (Engineer/Admin only).
- `SetMasterMute { mix_index, muted }` WebSocket client message: sets master mute for a mix (Engineer/Admin only).
- `MasterAck { mix_index, master_gain_db, master_muted, revision }` server response for master mutations.
- `MasterDelta` broadcast event: all sessions receive unsolicited `MasterAck` after any master mutation; Musician sessions receive only for their assigned mix.
- `master_event_tx` broadcast channel (capacity 256) in `AppState` for `MasterDelta` events.
- Integration tests: `ws_engineer_set_master_gain_returns_master_ack`, `ws_engineer_set_master_mute_returns_master_ack`, `ws_musician_denied_set_master_gain`, `ws_musician_denied_set_master_mute`, `ws_engineer_set_master_gain_invalid_gain_returns_error`, `ws_master_mutation_broadcasts_to_other_sessions`, `ws_master_broadcast_filtered_by_musician_assignment`.

### Security — Phase 23
- Musician role cannot send `SetMasterGain` or `SetMasterMute`; `check_permission` returns `false` for both with explicit match arm.
- Master mutation broadcast filtered by role and mix assignment, consistent with send-delta RBAC model.

### Added — Phase 22
- `deployment/caddy/Caddyfile`: LAN TLS configuration using mkcert certificate; `reverse_proxy` to `127.0.0.1:8080`; HTTP → HTTPS redirect; `X-Real-IP`, `X-Forwarded-For`, `X-Forwarded-Proto` headers forwarded.
- `deployment/systemd/openiem-server.service`: production systemd service with dedicated `openiem` user, loopback bind, `NoNewPrivileges`, `PrivateTmp`, `ProtectSystem=strict`, `ProtectHome`, `LimitNOFILE=65536`, `Restart=on-failure`.
- `deployment/raspberry-pi/README.md`: step-by-step RPi 5 deployment guide (binary install, key generation, systemd, Caddy+mkcert, CA trust per OS), mkcert CA private key protection instructions (chmod 600, no unencrypted backups, revocation procedure, certificate expiry check).
- `docs/adr/ADR-011-tls-deployment.md`: TLS-at-proxy strategy decision; consequences and alternatives rejected.

### Changed — Phase 22
- `docker-compose.yml`: corrected env var prefix from `OPEN_IEM_*` to `OPENIEM_*` to match server binary (`OPENIEM_BIND_ADDR`, `OPENIEM_ALLOW_INSECURE_HTTP`, `OPENIEM_DB_PATH`, `OPENIEM_JWT_PRIVATE_PEM`, `OPENIEM_JWT_PUBLIC_PEM`); healthcheck URL fixed to `/api/v1/health`; volume mount renamed from `./secrets` to `./keys`; added explicit WARNING that this file is dev-only.
- `server/api-server/src/main.rs`: startup `tracing::warn!` emitted for each detected legacy `OPEN_IEM_*` environment variable, preventing silent misconfiguration in environments that have not migrated.

### Security — Phase 22
- `OPENIEM_ALLOW_INSECURE_HTTP` is absent from the production systemd service, enforcing fail-closed HTTP-on-non-loopback behavior.
- Documented mkcert CA private key protection requirements: `rootCA.key` must be `chmod 600`, excluded from unencrypted backups, with revocation instructions.
- Closes HIGH gate from Phase 3 security follow-up: HTTPS/TLS listener and fail-closed transport configuration.

### Changed — Phase 21
- WebSocket authentication moved from URL query parameters to negotiated `Sec-WebSocket-Protocol: openiem.bearer.<JWT>`.
- Server selects and echoes authenticated subprotocol during `/ws/v1` upgrade; missing or empty credentials are rejected.

### Security — Phase 21
- Access tokens no longer appear in WebSocket URLs, reducing exposure through proxy/access logs and browser history.

### Added — Phase 20
- Musician hook tests cover authenticated REST snapshot, malformed nested state, delayed snapshot protection and `SendAck` reconciliation.
- WebSocket client validates protocol version, bounded request ID, ACK ranges and aborts snapshot fetch during connection cleanup.

### Security — Phase 20
- REST snapshot uses `Authorization: Bearer <token>`.
- Invalid ACK gain/pan values and malformed envelopes are rejected before local state mutation.
- WebSocket query-string token remains a documented limitation until cookie/subprotocol authentication is implemented.

### Added — Phase 19
- Musician client reconciles assigned mix snapshot through authenticated `GET /api/v1/state` after WebSocket connection.
- Nested snapshot validation rejects malformed or non-finite channel, mix and send values.
- `SendAck` updates local send state; stale REST snapshots are rejected by monotonic revision tracking.

### Changed — Phase 18
- Musician WebSocket client consumes `SendAck` revisions and ignores `State`/ACK messages older than current revision, preventing stale UI state after delayed broadcasts.

### Added — Phase 17
- WebSocket send mutation broadcasts: connected Engineer/Admin sessions receive unsolicited `SendAck` deltas after another session changes gain, pan or mute.
- Musician sessions receive broadcast deltas only for their currently assigned mix; originator receives only direct acknowledgement.
- Broadcast integration coverage added for cross-session delivery.

### Security — Phase 17
- WebSocket send ownership check and dispatch now share `mix_assignment_lock`, closing the assignment TOCTOU window.
- Broadcast ordering is serialized with assignment/mutation operations; ownership filtering builds payload under lock and releases lock before outbound socket I/O.

### Added — Phase 16
- WebSocket send mutations: `SetSendGain`, `SetSendPan`, `SetSendMuted` client messages via `/ws/v1`.
- `SendAck` server message: echoes current gain_db, pan, muted and state revision after each send mutation.
- Musician ownership enforcement: ws_handler rejects send mutations targeting a mix not assigned to the authenticated musician.
- Input validation in dispatch: `gain_db` must be finite and within `GAIN_DB_MIN..=GAIN_DB_MAX`; `pan` must be finite and within ±1.0.
- 17 new tests (7 WS integration, 6 control-server unit, 4 protocol round-trip).
- `axum-test` ws feature enabled in api-server dev-dependencies.

### Fixed — Phase 15 follow-up
- Musician Guide LAN example now uses actual server environment variable names (`OPENIEM_BIND_ADDR` and `OPENIEM_ALLOW_INSECURE_HTTP`).

### Changed — Phase 15 follow-up
- README, Musician Guide e pacotes web agora refletem Engineer Console operacional, ownership atômico e versão `0.2.0`.
- CI deixou de mascarar falhas de instalação, typecheck e teste nos frontends Musician e Engineer; scripts existentes agora são gates obrigatórios.
- Quality gate de release executa typecheck, testes e `npm audit` dos dois frontends.

### Security — Phase 15
- `mix_assignment_lock` agora cobre ownership e leitura/mutação de sends, fechando janela TOCTOU entre assignment persistido e alteração de estado.

### Added — Phase 14
- Contrato versionado de snapshot em `GET /api/v1/state`, com canais, mixes e sends filtrados por ownership de músico.
- `GET /api/v1/telemetry` para Engineer/Admin, reportando backend `simulated` e métricas `null` quando áudio real não está conectado.

### Security — Phase 13
- `POST /api/v1/audio/offer` agora valida `mix_id` de músicos contra assignment persistido antes de criar sessão WebRTC.

### Added — Phase 6 Engineer Console
- Engineer Console funcional em `web/engineer`: login, refresh por cookie HttpOnly após 401, dashboard autenticado, polling limitado, revisão do estado, sessões WebRTC e atribuição/remoção de mixes.
- Indicador explícito `SIMULATED` para áudio sem PipeWire no VPS.

### Changed — Phase 6
- Assignment usa ID numérico porque catálogo de usuários continua restrito à API Admin.

### Added — Phase 12
- Admin self-delete protection: `DELETE /api/v1/admin/users/{id}` returns 403 if caller matches target user ID.
- Dependabot configuration (`.github/dependabot.yml`): Cargo (weekly, limit 5), npm musician/engineer (weekly, limit 3), GitHub Actions (weekly).
- SBOM generation in release pipeline (`cargo-sbom`, best-effort, non-blocking); output `open-iem-server-<ver>-sbom.json` attached to GitHub Release.

### Fixed — Phase 12
- Release artifact naming bug: `build-server-x86`, `build-server-arm64`, `build-web` jobs now have `needs: [quality-gate, validate-version]` so version string is populated in artifact filenames.

---

### Added — P2 operational configuration backup/restore
- Added authenticated `GET/PUT /api/v1/config/backup` for Engineer/Admin roles.
- Enforced strict JSON snapshot fields; credentials and transient DSP state remain excluded.
- Restore remains CODE+CI/SIMULATED; clean-environment and runtime validation remain pending.

## [0.2.0] — 2026-09-09

### Added — Phase 11
- Versioned release pipeline (`release.yml`): semver tag triggers quality gate + multi-target builds + GitHub Release.
- Linux x86_64 server artefact: `api-server` + `open-iem-admin` + README + LICENSE + CHANGELOG, SHA-256 checksum.
- Linux ARM64 (aarch64) server artefact: cross-compiled for Raspberry Pi 5 target; SIMULATED on VPS until real Pi validates PipeWire/Opus.
- Musician PWA and Engineer UI artefacts bundled from `web/musician/dist` and `web/engineer/dist`.
- Version consistency gate: release workflow rejects if tag version ≠ `[workspace.package].version` in `server/Cargo.toml`.
- `server/.cargo/config.toml` linker config for ARM64 cross-compilation.
- `admin-cli` Cargo manifest aligned to workspace semver.

### Changed
- Workspace `version` bumped `0.1.0` → `0.2.0`.

---

### Added — Phase 10
- Persistent SQLite mix assignments with one mix per user.
- Engineer/Admin mix assignment API.
- Musician-owned mix send API for gain, pan, mute, and state reads.
- Numeric user ID (`uid`) in access JWT claims.

### Security
- Musician send mutations require assigned mix ownership; channel master controls remain Engineer+.


### Added — Phase 9
- **Admin API server-side routes** (`server/api-server/src/routes/admin.rs`): 4 endpoints, all require Admin role:
  - `GET /api/v1/admin/users` — list all users (id, username, role)
  - `DELETE /api/v1/admin/users/{id}` — delete user by numeric ID (204 / 404)
  - `GET /api/v1/admin/sessions` — list active (non-revoked, non-expired) refresh-token sessions
  - `DELETE /api/v1/admin/sessions/{id}` — revoke session by numeric ID (204 / 404)
- **DB methods** added to `Db`: `list_users`, `delete_user`, `list_active_sessions`, `revoke_session_by_id` — all parameterized, no format-string SQL.
- **`ApiError::NotFound(String)`** variant added (was `&'static str`); HTTP 404 response.
- **8 new HTTP integration tests** for all admin endpoints (RBAC enforcement, success paths, 404 paths). api-server: 19 → 27 tests.
- **3 new DB unit tests**: `list_users_empty_and_populated`, `delete_user_ok_and_not_found`, `list_and_revoke_sessions`.
- **npm audit job** added to CI (HIGH severity gate, non-blocking until dependencies present).
- **Biquad coefficient validation** vs Python/scipy: max delta ≈ 5×10⁻⁸ (f32 rounding only), algorithm confirmed identical.

### Fixed — Phase 9
- **Admin CLI `user create`** command: `--name` → `--username`, added required `--password` arg. Server body now `{username, password, role}` matching `CreateUserRequest`.
- **Admin CLI `session revoke`** command: `--token: String` → `--id: u64`, dispatches `DELETE /api/v1/admin/sessions/{id}`.

### Changed — Phase 9
- Total workspace tests: 157 → 168 (+11: 8 integration + 3 DB unit).
- `GET /api/v1/admin/users` added alongside existing `POST` (combined route).


- **EQ + Compressor integrated into `Mix::process`** audio chain: Sum → EQ → Compressor → Master Gain → Limiter. Both are disabled by default (passthrough when disabled).
- **`Mix` struct fields** `eq: ParametricEq` and `compressor: Compressor` exposed for per-mix DSP configuration.
- **5 new mix-engine tests** covering EQ boost at frequency, compressor reduction on loud signal, EQ passthrough when disabled, compressor passthrough when disabled, and EQ→Compressor chain order verification.
- **Admin CLI binary** (`server/admin-cli/`): `open-iem-admin` — commands: `user list/create/delete`, `session list/revoke`, `health`. Global `--server`, `--token` (or `OPEN_IEM_ADMIN_TOKEN` env), `--json` flags. Graceful 404 handling (Phase 9 server-side routes).
- **Musician Guide PDF** (`docs/guides/MUSICIANS-GUIDE.pdf`): generated via pandoc+xelatex from `MUSICIANS-GUIDE.md`. 61 KB, PDF 1.5. Emoji glyphs (⚠, ✅, ❌) render as blank in lmroman — cosmetic only.
- **`PHASE-8-REVIEW.md`** in `docs/reviews/`.

### Changed — Phase 8
- mix-engine test count: 74 → 79 (+5 new chain integration tests).
- Total workspace tests: 152 → 157.
- `server/Cargo.toml` workspace members: added `admin-cli`.

### Added — Phase 7
- **Biquad Parametric EQ** (`mix-engine/src/eq.rs`): real Type-II Transposed DF2 peaking filter replacing Phase 2 passthrough stub. RBJ Audio EQ Cookbook coefficients, SAMPLE_RATE=48000, stereo biquad state inline (no heap), all 4 bands independent.
- **BiquadCoeffs** struct: `identity()` + `peaking(frequency_hz, gain_db, q)` — coefficients recomputed on `set_band`.
- **BiquadState** struct: stereo delay lines `w1_l/w2_l/w1_r/w2_r`, `tick()` for per-sample processing.
- **RMS Compressor** (`mix-engine/src/compressor.rs`): stereo-linked RMS detector + smoothed gain reduction replacing Phase 2 passthrough stub.
- Compressor mutators: `set_threshold`, `set_ratio`, `set_attack_ms`, `set_release_ms` — all bump revision and recompute coefficients.
- **Docker Compose** dev environment (`docker-compose.yml`): api-server, musician-ui, engineer-ui services with health checks.
- **Musician Guide** (`docs/guides/MUSICIANS-GUIDE.md`): 14-section guide in pt-BR covering Linux requirements, server setup, JWT key generation, login, mix controls, WebRTC status, permissions, diagnostics, LAN, security, known limitations.
- **PHASE-6-REVIEW.md** and **PHASE-7-REVIEW.md** added to `docs/reviews/`.
- `cargo fmt --all` applied across workspace.

### Changed — Phase 7
- `ParametricEq::process` signature changed from `&self` to `&mut self` (required for biquad state mutation).
- mix-engine test count: 57 → 74 (+17 new DSP tests for biquad EQ and compressor).
- Total workspace tests: 135 → 152.

### Added — Phase 6
- Real trickle-ICE candidate injection via `Candidate::from_sdp_string` + `Rtc::add_remote_candidate` in streaming crate (SIMULATED on VPS)
- Named constants replace magic numbers in streaming crate (`MAX_SDP_BYTES`, `MAX_CANDIDATE_BYTES`, `MAX_USER_ID_BYTES`)
- HTTP integration test suite for api-server (19 tests: auth, RBAC, CSRF, audio routes, channel controls)
- Test fixture role casing corrected (`"MUSICIAN"`, `"ADMIN"`, `"ENGINEER"` — SCREAMING_SNAKE_CASE)
- `axum-test` pinned to `"21"`, `jsonwebtoken` gains `rust_crypto` feature
- Project directory structure (all planned directories)
- 11 project-specific agent skills in `.agents/skills/`
- Documentation structure (`docs/` with all subdirectories)
- ADR baseline (ADR-001 through ADR-008)
- `docs/SPEC-AUDIT.md` — specification audit with 14 FRs, 10 NFRs, 10 missing requirements, 10 architecture gaps
- `docs/ARCHITECTURE-GAPS.md` — 10 identified gaps with phase dependencies
- `docs/SKILLS.md` — skills registry and orchestration guide
- `docs/TODO.md` — prioritized backlog
- `docs/DEVELOPMENT-LOG.md` — development history
- `docs/DEVELOPMENT-ENVIRONMENT.md` — environment audit
- GitHub Actions CI foundation (Rust lint/test/build, ARM64 cross-build, frontend, skill validation)
- `scripts/validate-skills.sh` — skill validation script
- Symlinks: `.hermes/skills/` and `.claude/skills/` → `.agents/skills/`
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `LICENSE` (Apache 2.0), `.gitignore`

## [0.0.1] - 2026-09-07

### Added
- Initial repository creation
- Minimal README
