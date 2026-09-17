# TODO

## Estado atual — 2026-09-17 (P2 preset catalog/application status reconciliation)

- **Architecture closure:** P0 technical decisions closed in ADR-001..010; implementation/validation follow `docs/DEVELOPMENT-HANDOFF.md`.
- **P1-008 — API/UI:** rotas de domínio `GET /api/v1/system` e `GET /api/v1/channels` implementadas na PR #75; EQ UI implementada na PR #74; cenas REST e catálogos Musician/Engineer concluídos em código/CI; aplicação de presets built-in no Engineer Console concluída nas PRs #144–#151. Runtime permanece pendente.
- **P1-002:** GET /api/v1/devices com RBAC, DeviceManager no AppState; mergeado em main (PR #80; CI 13/13).
- **P1-003:** Observability concluído (PR #68; CODE+CI/SIMULATED; runtime permanece pendente).
- **P1-004:** RecoveryRegistry integrado ao ciclo WebSocket de Musician; concluído em main via PR #81, CI run `35055191463` (13/13). CODE+CI; runtime/hardware permanecem pendentes.
- **P1-007:** backup/restore sem secrets implementado na PR #73; CODE+CI/SIMULATED. Restauração em ambiente limpo e uso operacional permanecem pendentes. P1-006 release permanece bloqueado por confirmação explícita e validação física.
- **Phase 83–95:** backend/control changes, musician channel names, observability metrics REST endpoint implementados e mesclados em `main`; runtime de áudio/media permanece não validado.
- **Release `v0.3.1`:** tag existe e CI remoto do HEAD passou 13/13; GitHub Release ainda não publicada por exigir confirmação explícita.
- **Validação física:** PipeWire/ALSA, WebRTC/Opus, instalação Linux dedicada e Raspberry Pi 5 continuam `SIMULATED`/pendentes.
- **Guias:** espelhos PT/EN/ES existentes; revisar e sincronizar após validação física.
- **P1-015 — Versioned SQLite migrations:** concluído em CODE; `M001` registrado em `migrations`, reaplicação evitada no reopen e schema legado compatível quando `must_change_password` já existe.
- **P0-007 first-access password:** login exposes `must_change_password`; authenticated `PUT /api/v1/auth/password` replaces Argon2id hash and clears bootstrap flag; ordinary users are denied. CODE evidence; runtime remains pending.
- **P0-003 media handoff:** PRs #119 and #125 merged; GAP-001 implementation is CODE/SIMULATED complete, while real WebRTC/network runtime validation remains pending. In-memory `TransportAdapter::send_from_registry` UDP delivery coverage now passes in CODE. `MediaBridge`, bounded `SessionRegistry::drive_once`, `MediaWriter` Opus encoding, negotiated `str0m::media::Writer` boundary and bounded `TransportAdapter` socket owner exist at CODE/SIMULATED; failed sends requeue within bounded capacity. Runtime and hardware remain pending.
- **P2 Engineer Console scenes UI:** listar, criar, recuperar, editar revisões e deletar cenas integrado às rotas REST em `web/engineer`; cobertura CODE permanece nos testes do console.
- **P2 Engineer Console scene revisions:** edição JSON e criação de nova revisão via PUT integradas; cobertura CODE+CI (PR #136, run `35189802675`).
- **P2 Musician scenes read-only:** catálogo autenticado de cenas e cena ativa integrado à Musician UI; músico não recebe permissão de recall/criação. Evidência CODE local; runtime permanece pendente.
- **P2 Musician scene catalog tests:** cobertura de renderização, estados loading/error/vazio e refresh adicionada; PR #140 mergeada com CI 13/13 (run `35194150143`). Evidência CODE+CI; runtime permanece pendente.
- **P2 Preset catalog/application:** `GET /api/v1/presets` fornece catálogo read-only para Musician/Engineer/Admin; `POST /api/v1/presets/{id}/apply` aplica presets built-in somente para Engineer/Admin em canal validado; Engineer Console oferece seleção de canal e aplicação autenticada em código/CI. Criação, edição, persistência e aplicação de presets de mix permanecem fora do escopo. Evidência CODE + CI (PRs #144–#151); execução e integração em runtime/hardware permanecem pendentes.
- **P2 Engineer preset validation:** entradas inválidas são descartadas no cliente; payload não-array cai em estado vazio sem quebrar renderização. Evidência CODE local; runtime permanece pendente.
- **P2 Engineer preset feedback:** aplicação exibe confirmação com preset/canal e mantém erro fail-closed; respostas obsoletas não alteram estado. Evidência CODE local; runtime permanece pendente.
- **P2 Preset application boundary:** `channel_index` fora do limite é rejeitado antes de mutação; payload JSON com campos desconhecidos falha fechado. Cobertura CODE local; runtime permanece pendente.
- **P2 Musician scene catalog refresh:** botão autenticado de atualização manual adicionado; respostas concorrentes e respostas após logout são descartadas. Cobertura CODE via typecheck/test/build; runtime permanece pendente.
- **P2 SceneStore revision invariant:** cenas restauradas agora exigem `revision >= 1`; cobertura unitária rejeita revisão zero antes de persistência. Evidência CODE; runtime/hardware permanecem pendentes.
- **P2 Scenes backup/restore:** PRs #94–#97 mergeadas; exportação durável, restore atômico, rollback de ponteiro ativo e cobertura RBAC concluídos em CODE+CI (run `35126301166`, 13/13). Restauração em ambiente limpo coberta por teste de reabertura file-backed; validação operacional implantada permanece pendente; seleção do caminho persistente agora é testável sem mutação global de ambiente.

## Phase 92 — Controle de EQ por WebSocket

- [x] Especificar `SetEqBand`/`EqBandAck` no control-protocol.
- [x] Validar band index, frequência, gain, Q e mix index no control-server.
- [x] Definir broadcast de `EqBandDelta` no api-server com RBAC para Musician.
- [x] Executar fmt, clippy, testes completos, revisão independente e scan de segurança.
- [x] Atualizar README, CHANGELOG, DEVELOPMENT-LOG e review da Phase 92.
- [x] Commitar e abrir PR #54 (mergeado em main via `chore/docs-cleanup-status-sync`).
- [x] Confirmar CI remoto 11/11 para mudanças mergeadas anteriores; cada novo commit exige evidência no HEAD.
- [x] Frontend EQ controls — Phase 93 implementada e mergeada em main via PR #74.
- [x] P0-001 — substituir Mutex no callback JACK por fronteira SPSC bounded e fila de controle non-blocking; callback processa período completo; testes locais passam.
- [x] P0-002 — Audio Lab L1/L2: harness SIMULATED com 8 testes CI implementado (PR #57).
- [x] P0-003 — Media Plane: bridge/session/Opus/negotiated writer plus bounded `TransportAdapter` socket owner implemented; bounded UDP delivery, failed-send requeue, dequeue-budget and suffix-order coverage pass in CODE/CI/SIMULATED. Runtime WebRTC/PipeWire/ALSA and hardware remain pending.
- [x] P0-004 — Native/headless Opus receiver core: bounded ingress/jitter, decode, fail-safe mute e reconnect (SIMULATED; OS output/hardware pendentes, PR #59).
- [x] P0-005 — Clock: sample timestamps, bounded drift estimator e adaptive resampling (SIMULATED; hardware clock validation pendente).
- [x] P0-006 — registry bounded de pairing, identidade, binding músico/mix, revogação, re-pair explícito e digest Argon2id salted (CODE; DTLS-SRTP/API integration pendente).
- [x] P0-007 — bootstrap idempotente `soundtech` e fronteira de migração versionada (CODE+CI; PR #62).
- [x] P0-008 — ALSA explicit fallback backend: abertura PCM, hw_params, fail-safe mute, XRUN recovery, stop_flag Release/Acquire (CODE+CI; PR #63; PipeWire/hardware validation pendente).
- [x] P1-004 — Recovery: AppState RecoveryRegistry, lifecycle WS de conexão/desconexão, restauração de mix atribuído, guarda de ownership duplicado e conflito de assignment DB; 5 testes de recovery. PR #81; CI run `35055191463` (13/13); gates locais fmt, clippy, testes e documentação PASS. Runtime PipeWire/ALSA, WebRTC/Opus e hardware permanecem pendentes.
- [x] P1-005 — Network Tests: 5 perfis determinísticos (loss/jitter/reorder/outage/reconnect) no crate `network-fault`. Integração com `recovery` e `observability`. 47 testes verdes (CODE; PR #72 mergeado em main; CI remoto 12/12).
- [x] P1-008 — API/UI: `GET /api/v1/system` pública e `GET /api/v1/channels` protegida por RBAC, com testes de contrato; PR #75. Status de áudio permanece `SIMULATED`.


## P1 backlog status

- [x] P1-003 — observability concluído; runtime/hardware pendentes.
- [x] P1-004 — recovery concluído: PR #81, CI run `35055191463` 13/13, gates locais PASS; sem validação física.
- [x] P1-005 — network concluído (PR #72).
- [ ] P1-006 — release bloqueado: confirmação explícita, instalação real e Raspberry Pi 5 ainda pendentes.
- [x] P1-007 — backup/restore de configuração sem secrets implementado na PR #73; 8 testes unitários; restauração em ambiente limpo ainda pendente.
- [x] P1-008 — concluído (PR #75).
- [x] P2 — SceneManager SQLite: `SceneStore` implementado com revisions imutáveis, recall transacional e detecção de payload corrompido; PR #86, CI 13/13 (run `35067216669`). CODE+CI; runtime/hardware pendentes.
- [x] P2 — Scenes REST API: 7 rotas REST (GET/POST /api/v1/scenes, GET /api/v1/scenes/active, GET/PUT/DELETE /api/v1/scenes/{id}, POST /api/v1/scenes/{id}/recall) com RBAC, SceneStore integrado ao AppState; 7 testes de integração; PR #87, CI 13/13. CODE+CI; runtime/hardware pendentes.
- [x] P2 — Scenes REST API PUT coverage: revisão Engineer e bloqueio RBAC Musician cobertos por testes de integração.
- [x] P2 — SceneStore file-backed: `SCENE_STORE_PATH` seleciona SQLite persistente; teste cobre criação, reopen e leitura do payload. CODE+CI; backup/restore operacional e runtime pendentes.
- [x] P2 — Backup/restore operacional: `GET/PUT /api/v1/config/backup` com RBAC Engineer/Admin, payload estrito e integração ao `ControlState`; CODE+CI/SIMULATED. Restore em ambiente limpo e runtime permanecem pendentes.
- [x] P2 — SceneStore export/restore: `GET /api/v1/scenes/backup` exporta e `PUT` restaura cenas duráveis e ponteiro ativo em JSON versionado; validação estrita e substituição atômica; estado transitório excluído; CODE+CI/SIMULATED.
- [x] P2 — Scenes REST backup/restore API integration coverage: Engineer-only atomic replacement, active-pointer restore and Musician denial tests.
- [x] P2 — AppState SceneStore restart coverage: fresh `AppState` reopens same file-backed path and preserves scenes, revision, payload and active pointer; deployed runtime validation remains pending.
- [x] P2 — Preset catalog read-only: `GET /api/v1/presets` Engineer/Admin e catálogo no Engineer Console; aplicação/mutação permanece fora do escopo. CODE+CI; runtime/hardware pendentes.
- [x] P1-009 / Phase 94 — concluído; validação contra servidor/runtime/hardware permanece pendente.

## Phase 95 — Observability Metrics REST endpoint

- [x] Adicionar rota `GET /api/v1/metrics` protegida por RBAC Engineer/Admin.
- [x] Integrar `observability::AllMetrics` ao `AppState`; snapshot via `state.metrics.snapshot()`.
- [x] Implementar 3 testes de integração: role guard, schema_version=1, contadores inicializados a zero.
- [x] Executar fmt, clippy, testes (71 testes de integração), revisão independente e scan de segurança.
- [x] Commitar, abrir PR #78 e confirmar CI remoto 13/13 (run `35046851737`).
- [ ] Validar contadores contra áudio real e Raspberry Pi 5.

## Phase 94 — nomes de canais na Musician UI

- [x] Consumir `GET /api/v1/channels` com Bearer em memória.
- [x] Exibir nomes retornados pelo servidor, com fallback seguro para nomes padrão.
- [x] Validar payload, limites de índice e comprimento de nome.
- [x] Executar typecheck, 50 testes e build do frontend musician.
- [x] Commitar, abrir PR #77 e confirmar CI remoto 13/13 (run `35044749428`).
- [ ] Validar UI contra servidor real e Raspberry Pi 5.

## Phase 93 — Frontend EQ Controls (Engineer Console)

- [x] Adicionar componente `EqBandControl` no Engineer Console com sliders de frequência, gain e Q por banda.
- [x] Conectar ao WebSocket existente via mensagem `SetEqBand`.
- [x] Exibir `EqBandAck` e sincronizar estado com snapshot REST.
- [x] Executar typecheck, testes, build, revisão independente e scan de segurança.
- [x] Commitar, abrir PR e confirmar CI remoto.

## Phase 89 — Engineer Console: Channel Strip de gain/mute

- [x] Renderizar canais do snapshot `/api/v1/state` no Engineer Console.
- [x] Adicionar slider de gain `-144..+12 dB` e botão mute por canal.
- [x] Preservar Bearer auth, debounce, optimistic UI e guards contra respostas obsoletas.
- [x] 26 testes, typecheck e build aprovados; CI remoto 11/11 no PR #49.
- [x] PR #49 mesclado em `main`.
- [ ] Validar servidor real e Raspberry Pi 5.

Priority labels: **BLOCKER** | **HIGH** | **MEDIUM** | **LOW** | **RESEARCH**

## Phase 91 — validação de coeficientes Biquad contra referência

- [x] Adicionar vetores determinísticos de referência para quatro combinações de frequência, ganho e Q.
- [x] Validar coeficientes RBJ normalizados (`b0`, `b1`, `b2`, `a1`, `a2`) com tolerância de `5e-6` em `f32`.
- [x] Executar testes locais do crate `mix-engine`.
- [ ] Validar processamento de áudio em Raspberry Pi 5 real.


---

## Phase 87 — Engineer Console: controles master gain/mute via WebSocket

- [x] Adicionar `protocol.ts` com tipos WebSocket do engineer (`SetMasterGain`, `SetMasterMute`, `MasterAck`).
- [x] Implementar hook `useEngineerWs` com reconexão, guard de cleanup e handlers MasterAck/State/Error.
- [x] Integrar `WsBadge` e `MixMasterControl` (slider gain + botão mute) na App do engineer.
- [x] 18 testes aprovados, typecheck limpo, build limpo.
- [x] CI remoto 11/11 jobs aprovados (PR #47).
- [ ] Validar controles em servidor real e Raspberry Pi 5.

## Phase 84 — Reprodutibilidade do pipeline de release

- [x] Adicionar `--locked` a clippy, testes e builds Rust do workflow de release.
- [x] Normalizar timestamps, ordenação e ownership dos archives server/web com `SOURCE_DATE_EPOCH` do commit.
- [x] Adicionar gate que executa dois empacotamentos consecutivos e compara checksums em cada job de artefato.
- [ ] Confirmar dois builds do mesmo tag com checksums idênticos em runner CI.
- [ ] Publicar release `v0.3.1` após secrets, instalação real e validação final.
- [ ] Validar Raspberry Pi 5 real.

## Phase 83 — Pan e mudo master na UI do músico

- [x] Adicionar slider de pan por canal (-1 a +1) com rótulo L/C/R.
- [x] Desabilitar pan quando canal mudo.
- [x] Exibir badge MASTER MUTED quando servidor reporta mudo master (somente leitura).
- [x] Sincronizar pan do snapshot e enviar SetSendPan via WebSocket.
- [x] 42 testes, typecheck e build locais aprovados.
- [x] Implementar SetMasterMuted no protocolo quando suporte ao servidor estiver disponível.
- [x] Confirmar CI remoto verde no run `34761731828` (9 jobs aprovados após correção do pin de `actions/cache`).
- [ ] Validar Raspberry Pi 5 real.

## Phase 82 — revisão de segurança do instalador

- [x] Auditar fluxo de instalação após Phase 81.
- [x] Corrigir cópia repetida de UI que criava `dist/dist` ou preservava conteúdo obsoleto.
- [x] Exigir SHA completo, verificar checkout exato e recusar fonte mutável antes de build.
- [x] Publicar release versionado via staging limpo e restaurar `current` em falha de instalação.
- [x] Validar Node.js >= 20 antes de mutar host ou instalar runtime suportado.
- [x] Corrigir SHA inválido de `actions/cache` que quebrava todos os 10 jobs de CI (ff0c085).
- [x] Confirmar CI remoto verde nos runs `34761731828` e `34763037883` (9 jobs aprovados).
- [ ] Confirmar instalação real em host Linux dedicado.
- [ ] Desbloquear release e validar Raspberry Pi 5 real.

## Phase 81 — hardening do instalador e concorrência CI/release

- [x] Fazer `--dry-run` descrever todas operações sem alterar host.
- [x] Aceitar `--ref` como SHA-1 completo com checkout detached.
- [x] Validar caminhos usados em substituições `sed`.
- [x] Restaurar CI automático em `main` e serializar release por tag.
- [ ] Confirmar instalação real em host Linux dedicado.
- [ ] Desbloquear release e validar Raspberry Pi 5 real.

## Phase 80 — compatibilidade TypeScript 7 no Engineer Console

- [x] Adicionar `web/engineer/src/vite-env.d.ts` para declarar imports de assets Vite.
- [x] Validar typecheck, testes e build do Engineer localmente.
- [x] Confirmar CI remoto verde e atualizar PR #41.
- [ ] Validar release, instalação ARM64 e Raspberry Pi 5 real.

---

---

## BLOCKER

- [x] P0 — Confirmar correção da suíte Python de assinatura no GitHub Actions — runs `34761731828` e `34763037883` passaram nos 9 jobs.
- [ ] P0 — Publicar release `v0.3.1` após validar artefatos e instalação real.

---

## Phase 79 — compatibilidade OpenSSL 3.5 na geração de assinaturas

- [x] Adicionar `-rawin` aos helpers de teste Ed25519.
- [x] Validar suíte combinada localmente: 56 testes aprovados.
- [x] Confirmar CI remoto verde no run `34760297250` e atualizar documentação.
- [ ] Validar release, instalação ARM64 e Raspberry Pi 5 real.


---

## BLOCKER

- [x] P0 — Confirmar correção da suíte Python de assinatura no GitHub Actions — runs `34761731828` e `34763037883` passaram nos 9 jobs.
- [ ] P0 — Publicar release `v0.3.1` após validar artefatos e instalação real.

## Phase 79 — compatibilidade OpenSSL 3.5 na geração de assinaturas

- [x] Adicionar `-rawin` aos helpers de teste Ed25519.
- [x] Validar suíte combinada localmente: 56 testes aprovados.
- [x] Confirmar CI remoto verde no run `34760297250` e atualizar documentação.
- [ ] Validar release, instalação ARM64 e Raspberry Pi 5 real.

## Phase 78 — rejeição de dados residuais em archives

- [x] Rejeitar bytes residuais e streams gzip concatenados após archive válido.
- [x] Adicionar testes offline para os dois casos.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar archives e instalação em Raspberry Pi 5 real.

## Phase 77 — validação estrutural antes do payload

- [x] Validar nomes, raiz, tipos, allowlist e duplicatas antes de consumir payloads.
- [x] Adicionar teste que garante rejeição de membro inesperado sem leitura de payload.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar archives e instalação em Raspberry Pi 5 real.

## Phase 76 — leitura integral de payloads de archive

- [x] Consumir payload completo de cada arquivo regular em chunks limitados.
- [x] Rejeitar payload truncado ou ausente durante leitura limitada pelo tamanho declarado.
- [x] Adicionar teste offline de payload truncado.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar archives e instalação em Raspberry Pi 5 real.

## Phase 75 — fonte única de versão e gate reproduzível

- [x] Criar `VERSION` canônico e validar manifests Rust/frontend e tags SemVer.
- [x] Integrar gate ao workflow de release e aos testes locais.
- [x] Atualizar README, CHANGELOG, DEVELOPMENT-LOG e review.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar build/runtime ARM64, mídia WebRTC, PipeWire/ALSA e Raspberry Pi 5 real.

## Phase 74 — documentação visual das interfaces

- [x] Documentar interfaces reais implementadas em `docs/INTERFACES.md`.
- [x] Gerar SVG/PNG documentais sem secrets e marcar como mockups, não screenshots de runtime.
- [x] Atualizar README, CHANGELOG, DEVELOPMENT-LOG e guias afetados.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar UI em runtime, mídia WebRTC, PipeWire/ALSA e Raspberry Pi 5 real.

## Phase 73 — Guia do Músico alinhado ao signaling HTTP

- [x] Atualizar guia com sequência real de oferta SDP e trickle ICE via HTTP.
- [x] Marcar control plane/Sans-IO como validado e mídia como `SIMULATED`.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar mídia WebRTC, PipeWire/ALSA e runtime em Raspberry Pi 5 real.

## Phase 72 — integração HTTP de signaling

- [x] Cobrir oferta SDP autenticada e resposta via rota HTTP.
- [x] Cobrir trickle ICE autenticado após negociação na mesma sessão.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar mídia WebRTC, PipeWire/ALSA e runtime em Raspberry Pi 5 real.

## Phase 71 — snapshot validado para publicação

- [x] Fixar bundle de entrada por descritor de diretório e abrir membros com `O_NOFOLLOW`.
- [x] Copiar em chunks limitados para staging privado e publicar output somente após validação.
- [x] Limitar bundle a 32 entradas e adicionar testes de atomicidade, cleanup e leitura limitada.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 70 — hardening do validador de bundle

- [x] Limitar tamanho de arquivo, bundle e manifesto antes de consumir conteúdo.
- [x] Validar SBOM opcional como JSON objeto e rejeitar manifesto symlink.
- [x] Adicionar testes offline para SBOM inválido, limite de arquivo e manifesto symlink.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 69 — validação do bundle final de release

- [x] Validar conjunto final, versão, checksums, assinaturas server e arquivos inesperados antes da publicação.
- [x] Fixar fingerprint SHA-256 da chave pública e falhar fechado em ausência, formato inválido ou divergência.
- [x] Gerar manifesto de nomes/digests validados e publicar somente arquivos listados nele.
- [x] Integrar teste do validador de bundle ao job Python de segurança do CI.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Provisionar e distribuir chave pública Ed25519 por canal independente.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 68 — nomes Windows e limites de assinatura

- [x] Rejeitar caracteres inválidos, pontos/espaços finais e nomes reservados Windows em cada componente de archive.
- [x] Limitar assinatura e chave pública detached a 64 KiB antes de executar OpenSSL.
- [x] Adicionar testes offline determinísticos.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 67 — nomes seguros entre plataformas

- [x] Rejeitar separador `\\` e caracteres de controle em nomes de archive.
- [x] Adicionar testes offline para separador Windows e caractere de controle.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 66 — raiz canônica em archives

- [x] Rejeitar raiz `.`/`..` ou não canônica no validador.
- [x] Adicionar teste offline para raiz não canônica.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 65 — limites incrementais durante leitura de archives

- [x] Aplicar limites por membro e total descomprimido antes de consumir membros posteriores.
- [x] Adicionar teste offline de rejeição imediata para membro oversized.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 64 — tratamento fail-closed da portabilidade do validador

- [x] Converter ausência de `O_NOFOLLOW` em erro CLI controlado, sem traceback.
- [x] Validação local concluída.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 63 — verificação fail-closed de arquivos assinados

- [x] Exigir `O_NOFOLLOW` sem fallback permissivo, abrir com `O_NONBLOCK` e usar `/usr/bin/openssl` sem resolução por `PATH`.
- [x] Abrir artifact, assinatura e chave com `O_NOFOLLOW` e manter descritores estáveis durante OpenSSL.
- [x] Rejeitar symlink e entradas não regulares para todos os arquivos verificados.
- [x] Adicionar testes offline para symlink em artifact e assinatura.
- [x] Cobrir ausência de `O_NOFOLLOW` no validador de archives.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 62 — testes de segurança Python no CI

- [x] Adicionar job CI dedicado para testes dos validadores de archive e assinatura.
- [x] Fixar `actions/setup-python` por SHA completo.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 61 — abertura fail-closed de archives

- [x] Abrir archive com `O_NOFOLLOW` e medir tamanho no descritor validado.
- [x] Rejeitar entradas que não sejam arquivos regulares.
- [x] Adicionar testes offline para symlink e diretório.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 60 — validação incremental de archives

- [x] Interromper leitura ao exceder 32 membros, sem materializar headers além do limite.
- [x] Adicionar testes offline para limites comprimido e total descomprimido.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 59 — limites de recursos no validador de archives

- [x] Rejeitar archive acima de 512 MiB comprimidos, mais de 32 membros, membro acima de 256 MiB e total descomprimido acima de 512 MiB.
- [x] Adicionar testes offline para limites de contagem e tamanho.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

---

## Phase 58 — verificação local de assinatura Ed25519

- [x] Criar `scripts/verify-release-signature.py` com verificação detached Ed25519 fail-closed.
- [x] Adicionar testes offline para assinatura válida, artefato alterado, assinatura ausente e chave symlink.
- [x] Integrar geração/publicação de assinatura no workflow usando secret externo `OPENIEM_RELEASE_SIGNING_KEY_PEM`; chave nunca entra no repositório.
- [ ] Provisionar chave Ed25519 no GitHub Actions e distribuir fingerprint/chave pública por canal independente.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 57 — validação de archive no instalador e origem HTTPS

- [x] Usar `scripts/validate-release-archive.py` antes de extrair archive ARM64.
- [x] Definir `OPENIEM_ALLOWED_ORIGINS` no unit systemd e documentar ajuste para LAN IP.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 56 — correção do lifetime do diretório temporário do Caddyfile

- [x] Manter `CERT_WORK_DIR` até concluir cópia e instalação do Caddyfile.
- [ ] Validar `caddy validate` e instalação em Raspberry Pi 5 real.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 55 — hardening de caminho no deployment

- [x] Resolver Caddyfile a partir da raiz confiável do clone.
- [x] Rejeitar Caddyfile symlink antes de instalação privilegiada.
- [ ] Validar `caddy validate` e instalação em Raspberry Pi 5 real.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 54 — provenance de artefatos de release

- [x] Gerar attestation de provenance para archives de servidor x86_64 e ARM64 antes do upload.
- [ ] Validar publicação e verificação da attestation após desbloquear GitHub Actions.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 53 — validação de archives de release

- [x] Validar archives de servidor x86_64/ARM64 antes de checksum/upload; rejeitar traversal, links, membros inesperados e binários ausentes.
- [x] Adicionar testes offline determinísticos, casos maliciosos e contrato CLI do validador.
- [x] Fazer instalador ARM64 exigir e instalar `api-server` e `open-iem-admin`.
- [x] Adotar assinatura independente para autenticar artefatos de release — integração Ed25519 no workflow concluída na Phase 58.
- [ ] Provisionar secret Ed25519 e distribuir fingerprint/chave pública por canal independente.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 52 — follow-up de verificação

- [x] Impedir redirects no Admin CLI para preservar política HTTPS/loopback em cada requisição.
- [x] Fixar `cargo-audit` em `0.22.2` nos workflows CI e release.
- [x] Gerar certificados LAN em diretório temporário e instalar via `sudo install` com ownership/modos explícitos.
- [ ] Adicionar validação automatizada de archive malicioso e testar instalação em Raspberry Pi 5 real.
- [x] Adotar assinatura independente para autenticar artefatos de release — integração Ed25519 no workflow concluída na Phase 58.
- [ ] Provisionar secret Ed25519 e distribuir fingerprint/chave pública por canal independente.
- [ ] Desbloquear CI remoto antes de merge/release.

---

## Phase 51 — hardening do fluxo de deployment ARM64

- [x] Restringir redirects de download a HTTPS.
- [x] Criar diretórios de serviço com ownership e modos explícitos.
- [x] Gerar chaves JWT em diretório temporário e instalar com permissões restritas.
- [x] Resolver e validar unit systemd a partir de clone confiável antes da instalação.
- [ ] Validar instalação em Raspberry Pi 5 real após publicação de release.

## Phase 50 — hardening do instalador ARM64

- [x] Fazer download em diretório temporário com `set -euo pipefail` e limpeza automática.
- [x] Validar tag SemVer antes de construir URL/caminhos.
- [x] Rejeitar traversal, caminhos absolutos, symlinks e hard links antes da extração.
- [ ] Validar instalação em Raspberry Pi 5 real após publicação de release.

## Phase 49 — correção do guia de instalação ARM64

- [x] Alinhar nome e caminho do artefato Raspberry Pi ao workflow de release.
- [ ] Executar instalação em Raspberry Pi 5 real após publicação de release; runtime ARM64 continua não validado.

## Phase 48 — pinning imutável das actions

- [x] Fixar actions de terceiros de CI e release em commits SHA completos, com comentários de versão.
- [ ] Executar CI/release após desbloqueio do runner; não publicar artefato sem gates reais.

## Phase 47 — hardening dos workflows

- [x] Corrigir filtro de tags semver do workflow de release e reforçar validação exata no job `validate-version`.
- [x] Restringir permissões GitHub Actions; escrita limitada ao job `github-release`.
- [x] Pin de actions de terceiros por SHA completo; avaliação de atualizações permanece com Dependabot.
- [ ] Executar CI/release após desbloqueio do runner; não publicar artefato sem gates reais.

## Phase 46 — alinhamento do runner de release

- [x] Corrigir follow-up de revisão: documentar `make test-audio` e restringir validação PDF a `docs/guides`.
- [x] Trocar `ubuntu-24.04` por `ubuntu-latest` nos seis jobs de `.github/workflows/release.yml`.
- [ ] Executar workflow de release após desbloqueio do runner; validar gates, artefatos e checksums sem publicar artefato não testado.

## Phase 45 — correção do label de runner

- [x] Trocar `ubuntu-24.04` por `ubuntu-latest` em todos os jobs CI.
- [x] Confirmar resultado após push: run `34607886129` falhou antes dos steps em todos os jobs com `runner_id=0` e `steps=[]`.
- [ ] Desbloquear GitHub Actions/runner; runs `34614028392` e `34614025997` também falharam pré-steps (`runner_id=0`, `steps=[]`); API de permissões e runners retorna HTTP 403 para token atual; sem CI remoto verde não há release `v0.3.1`.

## Phase 43 — verification gates

- [x] Adicionar `server/Cargo.lock` e remover exclusão global do lockfile.
- [x] Adicionar job CI para documentação, PDF, skills e whitespace.
- [x] Renumerar ADRs duplicados 012 para 013/014.

## READY / CURRENT PHASE

- [x] P1 — Tornar `make up` reconstruível por padrão — Phase 40 — `docker compose up -d --build`; evita imagens obsoletas no desenvolvimento.
- [x] P1 — Validar Musician Guide PDF de forma reproduzível — Phase 42 — `make docs` extrai texto e renderiza PDF; `mutool` cobre ambiente sem `pdftotext`.
- [x] P1 — JWT pós-emissão revogável — Phase 31 — access-session mappings checked by middleware and WebSocket message/keepalive loops; local verification passes, CI remains blocked.

- [x] P1 — Corrigir Docker Compose dev — Phase 30 — Dockerfiles de desenvolvimento adicionados para API e UIs; `docker compose config` passa sem aviso de `version` obsoleto. Build completo depende de chaves JWT locais e daemon Docker disponível.
- [x] P1 — Implementar CLI `iem` com paridade documentada ao Makefile — Phase 35 — dispatcher tipado para help/status/diagnostics/docs/test/build/up/down; versão `0.3.1`; sem instalação global e sem `iem run`.
- [x] P1 — Corrigir saída do `open-iem-admin` — Phase 34 — HTTPS obrigatório fora de localhost, tabelas preservam união de colunas e 404 não afirma recurso planejado.
- [x] P1 — Criar harness de áudio determinístico — Phase 32 — cobre determinismo, sinais sintéticos, isolamento, ganho, pan, mute, limiter e finitude; não substitui hardware. Recuperação stop/start permanece follow-up.
- [x] P1 — Criar matriz formal de validação Docker/Linux/Raspberry Pi — Phase 37 — matriz publicada; execução em Raspberry Pi 5 continua pendente.
- [x] P1 — Adicionar sincronização de observabilidade ao desenvolvimento — Phase 36 — Engineer Console consulta telemetria e mantém métricas desconhecidas como `UNKNOWN`; áudio real segue pendente.

## BLOCKED / VALIDATION REQUIRED

- [ ] P0 — Validar PipeWire/ALSA e áudio real no Raspberry Pi 5 — status HARDWARE VALIDATION REQUIRED.
- [ ] P0 — Validar mídia WebRTC, latência, jitter, perda e recuperação — status HARDWARE VALIDATION REQUIRED.

---

## HIGH

- [x] Add API snapshot/telemetry contract before expanding Engineer Console (Phase 14; audio metrics remain SIMULATED)
- [ ] Install PipeWire on target hardware (Raspberry Pi 5) for Phase 1 validation
- [x] Evaluate audio transport options; WebRTC selected in ADR-004 (Phase 5 signaling scaffold complete)
- [x] Wire authenticated audio signaling routes and engineer session listing (Phase 5)
- [x] Establish authentication mechanism decision (ADR-008 — Accepted and implemented; TLS deployment validation pending)
- [x] HTTP integration tests for api-server (Phase 6 — 19 tests green)
- [x] Trickle ICE real injection in Sans-IO loop (Phase 6 — str0m Candidate::from_sdp_string)
- [x] WebSocket send mutations: SetSendGain/SetSendPan/SetSendMuted with musician ownership enforcement (Phase 16)
- [x] Biquad EQ real DSP (Phase 7 — TDF2 peaking biquad, RBJ coefficients, no heap)
- [x] RMS Compressor real DSP (Phase 7 — stereo-linked, exp-RMS, smoothed gain reduction)
- [x] Add ARM64 cross-compilation to CI (`rust-build-arm64` installs `gcc-aarch64-linux-gnu` and builds `aarch64-unknown-linux-gnu`; CI evidence: run `34792164989`)
- [x] Musician mix ownership enforcement (mix sends and audio signaling; Phase 13)
- [x] Admin API server-side routes (Phase 9 — complete)

---

## MEDIUM

- [x] Initialize Rust workspace (`server/Cargo.toml`)
- [x] Initialize frontend projects (`web/musician/`, `web/engineer/`) — Phase 4 complete
- [x] Create Docker Compose for local development
- [x] Define PipeWire filter node architecture for Mix Engine (pipewire-jack — see docs/research/pipewire-integration.md)
- [x] Design channel/mix state machine (revision control — implemented in mix-engine)
- [x] Define WebSocket message type catalog (`server/control-protocol/`; Phase 3 foundation)

---

## LOW

- [x] Set up `cargo audit` in CI (Phase 8 — existing job)
- [x] Set up `npm audit` in CI (Phase 9 — npm-audit job added)
- [x] Create `examples/` with minimal mix scenario
- [x] Include deterministic `audio-engine` integration harness in `make test` — Phase 39
- [x] Configure Dependabot for dependency updates
- [x] Set up code coverage reporting
- [x] LOW: Log DB errors in master broadcast fan-out (fail-closed and observable)
- [x] LOW: mix_assignment_lock held during DB read in broadcast fan-out — lock removed from receiver-side fan-out; read-only ownership check now lock-free (Phase 86)
- [x] LOW: Test DB failure during musician WebSocket ownership lookup — fail-closed coverage added Phase 24
- [x] WebSocket keepalive: server Ping/Pong timeout and bounded socket sends — Phase 24 (policy timing covered; transport timing integration remains pending)
- [x] LOW: WebSocket keepalive: add deterministic state-machine coverage without waiting 30/60 seconds — tracker e loop cobrem timeout pendente, Pong incorreto, Pong correlacionado, ausência de falso timeout após Pong válido e fronteiras do intervalo 30 s/timeout 60 s
- [x] LOW: Aplicar limite de 16 KiB em mensagem/frame no `WebSocketUpgrade`, antes da alocação do payload — Phase 26 follow-up
- [x] LOW: Testar rejeição de frame/mensagem acima de 16 KiB via integração WebSocket — teste de transporte confirma encerramento para mensagem Text acima do limite (Phase 26 follow-up)
- [x] LOW: WebSocket: revogação pós-emissão de JWT — Phase 31; middleware, keepalive e mensagens revalidam access-session mapping persistido
- [x] LOW: WebSocket: quotas agregadas por usuário/IP — 4 por usuário, 16 por IP, 64 global; reserva atômica e liberação RAII (Phase 28)
- [x] HIGH: Limitar falhas de autenticação `/ws/v1` por IP — 5 falhas por janela de 60 s; estado bounded em memória, peer via `ConnectInfo<SocketAddr>` (Phase 29)
- [x] LOW: WebSocket: ressincronização após broadcast lag — servidor emite `State` com revisão autoritativa; Musician refaz snapshot REST (Phase 27); revisão monotônica cobre ACK antes do primeiro snapshot
- [x] LOW: WebSocket protocol control coverage: client Ping/Pong payload preservation and binary-frame rejection (Phase 24 follow-up)
- [x] LOW: WebSocket rate quota counts every inbound frame, including control frames, preventing Ping/Pong flood bypass (Phase 24 follow-up)
- [x] MEDIUM: Enforce process-wide WebSocket connection quota (64 permits); per-user/IP quotas and post-issuance JWT revocation checks implemented in Phases 28/31
- [x] LOW: Add explicit WebSocket logging redaction policy and generic client-facing protocol errors — Phase 25; parser/transport details redacted, normal close classified separately
- [x] LOW: Preserve validated WebSocket `request_id` in authorization errors — Phase 26; invalid envelopes use synthetic `server`

---

## RESEARCH

- [x] **Audio Transport** — Preliminary evaluation complete (docs/research/audio-transport/EVALUATION.md); WebRTC selected as primary; final benchmarks deferred to Phase 5
- [x] **Browser Audio Constraint** — Phase 90 verified WebRTC browser media versus native receiver constraints; real media validation remains pending
- [x] **PipeWire filter node API** — pipewire-jack selected for Phase 1 (docs/research/pipewire-integration.md)
- [ ] **Raspberry Pi 5 realtime tuning** — PREEMPT_RT kernel, PipeWire latency config, USB audio device selection
- [ ] **JPMixer architecture** — Study WebSocket/scene/mix model as UX reference (verify license before using code)
- [ ] **Windows x64 audio backend** — Implement and validate WASAPI/ASIO backend; current server backend is not native Windows audio.
- [ ] **Platform validation matrix** — Keep Windows x64, Linux x64 and Raspberry Pi 5 ARM64 evidence separate; macOS, Android and iPadOS remain future/backlog.
- [x] **Linux realtime scheduling** — PipeWire+rtkit for Phase 1; hybrid in Phase 2 (docs/research/realtime-scheduling.md)

---

## PHASE 3 SECURITY FOLLOW-UP

- [x] HIGH: Add HTTPS/TLS listener and fail-closed transport configuration — **DONE Phase 22** (Caddy config, systemd, RPi5 guide, ADR-011)
- [x] HIGH: Authorize every WebSocket message by role and musician mix ownership (Phase 23 — SetMasterGain/SetMasterMute RBAC complete; all WS messages now have explicit role checks)
- [x] HIGH: Enforce WebSocket expiry, rate limits and post-issuance revocation through persistent access-session mappings (Phase 31); revocation is checked on handshake, messages and keepalive ticks
- [x] HIGH: Make refresh rotation atomic in one SQLite transaction
- [x] MEDIUM: Add Origin/CSRF validation and bounded auth request inputs
- [x] MEDIUM: Bound HTTP request body to 16 KiB; route payload validation remains pending
- [x] MEDIUM: Parse bind address as SocketAddr and fail closed for non-loopback HTTP without explicit dev override
- [x] MEDIUM: Preflight refresh token owner and JWT before atomic rotation to avoid token loss on issuance failure
- [x] MEDIUM: Make musician mix ownership checks and send mutations atomic under `mix_assignment_lock` (Phase 15)

## PHASE 7 STATUS

- [x] Biquad peaking EQ real processing (TDF2, RBJ coefficients, stereo biquad state, 48kHz)
- [x] Stereo-linked RMS compressor (exp-RMS detector, smoothed gain reduction, mutators)
- [x] Docker Compose dev environment (api-server, musician-ui, engineer-ui)
- [x] Musician Guide pt-BR (docs/guides/MUSICIANS-GUIDE.md)
- [x] PHASE-6-REVIEW.md and PHASE-7-REVIEW.md
- [x] Integrate EQ + Compressor into Mix::process audio chain (Phase 8)
- [x] Create admin CLI for user management (Phase 8)

## PHASE 9 STATUS

- [x] Admin API server-side routes: GET/DELETE /api/v1/admin/users, GET/DELETE /api/v1/admin/sessions
- [x] DB methods: list_users, delete_user, list_active_sessions, revoke_session_by_id
- [x] ApiError::NotFound(String) variant
- [x] 8 HTTP integration tests for admin endpoints
- [x] 3 DB unit tests for new methods
- [x] Admin CLI fix: --username/--password for user create, --id for session revoke
- [x] Biquad coefficient validation vs Python/scipy (delta ≤ 5×10⁻⁸)
- [x] npm audit CI job (HIGH severity gate)
- [x] Admin self-delete protection (policy gate — deferred Phase 10)
- [x] Musician mix ownership enforcement (mix sends and audio signaling; Phase 13)

## PHASE 8 STATUS

- [x] EQ + Compressor wired into Mix::process chain (Sum → EQ → Comp → Master Gain → Limiter)
- [x] Mix struct exposes `eq: ParametricEq` and `compressor: Compressor` fields
- [x] 5 new mix-engine integration tests (EQ boost, compressor reduction, passthrough, chain order)
- [x] Admin CLI binary (`server/admin-cli/`) — user/session/health commands, JSON/table output
- [x] Musician Guide PDF (`docs/guides/MUSICIANS-GUIDE.pdf`) — pandoc+xelatex, 61 KB
- [x] PHASE-8-REVIEW.md
- [x] Admin API server-side routes (Phase 9) — complete and covered by `server/api-server/src/routes/admin.rs` plus integration tests: GET/POST/DELETE `/api/v1/admin/users` and GET/DELETE `/api/v1/admin/sessions`.
- [ ] Biquad coefficient validation vs reference implementation (scipy)
- [ ] Generate Musician Guide PDF (Phase 8 documentation) — DONE Phase 8
- [ ] Biquad coefficient validation vs reference implementation (Phase 8) — deferred Phase 9

## PHASE 1 STATUS

- [x] Audio engine crate with simulated backend and feature-gated JACK/PipeWire bridge
- [x] Phase 1 review completed: PASS WITH CONDITIONS
- [ ] Validate real PipeWire graph on Raspberry Pi 5 (hardware blocker)

## PHASE 21 STATUS

- [x] Replace WebSocket query-token authentication with negotiated subprotocol authentication
- [x] Reject missing, empty and query-only WebSocket credentials
- [x] Preserve server-selected subprotocol during HTTP upgrade
- [x] Update Musician client and integration coverage

## PHASE 20 STATUS

- [x] Add independent hook tests for REST snapshot, malformed nested state, delayed snapshot and `SendAck`
- [x] Validate WebSocket envelope version/request ID and ACK ranges client-side
- [x] Abort snapshot fetches when WebSocket connection is cleaned up
- [x] Replace WebSocket query-token authentication with cookie or subprotocol authentication — DONE Phase 21

## PHASE 18 STATUS

- [x] Musician client accepts `SendAck` revisions
- [x] Musician client ignores stale `State`/`SendAck` revisions
- [x] Add client reconciliation of full channel/send state after revision gap (Phase 19; REST snapshot + SendAck application)

## PHASE 17 STATUS

- [x] Broadcast WebSocket deltas for send gain/pan/mute
- [x] Engineer/Admin cross-session `SendAck` delivery
- [x] Musician mix ownership filtering for broadcasts
- [x] Assignment lock coordination and integration coverage
- [x] Independent security/code review; findings fixed
- [x] Add client-side revision ordering/reconciliation for missed or lagged broadcasts — Phase 27; stale revisions ignored and lagged snapshots reconciled

## COMPLETED

- [x] Repository initialized with correct GitHub remote
- [x] Environment audited and documented
- [x] Project directory structure created
- [x] 11 project skills created in `.agents/skills/`
- [x] Documentation structure created (`docs/`)
- [x] ADR baseline created (ADR-001 through ADR-008)
- [x] CI foundation created (GitHub Actions)
- [x] `scripts/validate-skills.sh` created
- [x] Root project files created (README, CONTRIBUTING, SECURITY, LICENSE, CHANGELOG, .gitignore)
- [x] Latency budget defined (`docs/audio/LATENCY-BUDGET.md`) — GAP-005 CLOSED
- [x] XRUN SLA defined (`docs/audio/AUDIO-SLA.md`) — GAP-006 CLOSED
- [x] Mix Engine Rust crate implemented (Channel, MixSend, Mix, Limiter, MixEngine)
- [x] 53 unit + doc tests passing — 0 clippy warnings

- [x] Lookahead brick-wall limiter (LOOKAHEAD_FRAMES=64 @ 48kHz = 1.33ms)
- [x] ParametricEq stub (passthrough — biquad DSP Phase 7)
- [x] Compressor stub (passthrough — dynamics DSP Phase 7)
- [x] 80 tests passing — 0 clippy warnings


## P1 TOPOLOGY STATUS

- [x] P1-001 — topology capability model and Channel Mode validation: `server/topology`, explicit source mapping, typed invalid-config errors, 11 tests (PR #65; CODE + CI local e remoto aprovados).
- [x] P1-002 — Device Manager: capability discovery, hot-plug and recovery state machine; integrado ao AppState, GET /api/v1/devices com RBAC; 3 testes de integração; CI remoto 13/13 (run `35052291469`; PR #80).

## PHASE 10 STATUS

- [x] SQLite mix assignment model and Engineer/Admin API
- [x] JWT numeric user identity (`uid`)
- [x] Musician-owned send gain/pan/mute routes
- [x] Ownership and assignment validation
- [x] Add dedicated Phase 10 integration coverage for assignment lifecycle and musician ownership routes


## PHASE 11 STATUS

- [x] Versioned release pipeline `.github/workflows/release.yml`
- [x] Version consistency gate (tag == workspace version)
- [x] Quality gate (fmt + clippy + tests + cargo-audit) required before builds
- [x] Linux x86_64 server artefact with SHA-256
- [x] Linux ARM64 cross-compile for Raspberry Pi 5 (SIMULATED — not hardware-validated)
- [x] Musician PWA + Engineer UI artefacts
- [x] GitHub Release with CHANGELOG excerpt
- [x] Workspace version bumped to 0.2.0; admin-cli aligned to workspace
- [ ] Tag v0.2.0 and verify full pipeline on GitHub Actions
- [x] Investigate `v0.3.0` release gate failure; tag points to pre-sync commit
- [ ] Publish `v0.3.1` and verify full pipeline on GitHub Actions — código local validado; CI remoto falha antes dos steps; logs bloqueados por permissão do token atual
- [ ] Validate ARM64 binary on real Raspberry Pi 5 hardware
- [x] Dependabot for Cargo + npm

## PHASE 12 STATUS

- [x] Fix release artifact naming (validate-version chain in build jobs)
- [x] Dependabot: Cargo + npm (musician/engineer) + GitHub Actions, weekly
- [x] Admin self-delete protection: 403 when caller deletes own UID
- [x] SBOM: cargo-sbom in release pipeline, best-effort non-blocking

- [x] SBOM generation (cargo-sbom in release pipeline)
- [x] Sincronizar versões com tag v0.3.0 após falha do gate de consistência
- [ ] Reapontar tag v0.3.0 e verificar pipeline completo no GitHub Actions
