# Open IEM Platform — Escopo de Fechamento 100% — Plano

> **For Hermes:** executar via `subagent-driven-development`, uma task por vez, com revisão independente e commits em `develop`. Nunca fazer merge sem autorização explícita.

**Goal:** transformar 16 pendências reais em trilhas verificáveis, separando CODE/SIMULATED, SOFTWARE_RELEASE_GATE, RUNTIME_VALIDATED e HARDWARE_CERTIFICATION, sem declarar 100% antes de toda evidência obrigatória existir.

**Architecture:** manter controle e mídia separados. Validar primeiro o caminho software Sans-IO e os gates portáveis; depois runtime em host Linux físico; depois hardware Raspberry Pi/USB; por fim release assinada. WebRTC/DTLS-SRTP real exige dois peers, rede real e captura de métricas; não aceitar teste unitário ou loopback artificial como substituto.

**Tech Stack:** Rust workspace (`server/Cargo.toml`), `str0m`, Opus, PipeWire/WirePlumber, ALSA/CPAL conforme backend existente, Docker/QEMU para gates portáveis, GitHub Actions, `.deb`, SBOM, Ed25519, Prometheus/diagnostics.

---

## Escopo e classificação

| ID | Pendência | Classe | Critério de fechamento |
|---|---|---|---|
| T01 | WebRTC/Opus E2E completo | CODE + RUNTIME_VALIDATED | CODE: dois peers Sans-IO/str0m com transporte controlado; RUNTIME: dois processos, sockets e áudio em host Linux, com SDP/ICE/DTLS-SRTP, PCM e métricas |
| T02 | DTLS-SRTP real | RUNTIME_VALIDATED | Fingerprint/cipher/handshake verificados em captura/log, sem downgrade, com teste negativo |
| T03 | PipeWire/WirePlumber físico | RUNTIME_VALIDATED | Host Linux físico enumera graph, captura/reprodução e teardown limpo |
| T04 | ALSA/USB físico | HARDWARE_CERTIFICATION | USB real, seleção por capability/ID, captura/reprodução, unplug/replug |
| T05 | Rede real | RUNTIME_VALIDATED | LAN isolada, perda/jitter/reorder controlados e tráfego real entre peers |
| T06 | Raspberry Pi 3/4/5 | HARDWARE_CERTIFICATION | Matriz de suporte declara baseline Pi 3 e compatibilidade Pi 4/5; cada claim tem ARM64 instalado, serviço ativo, áudio e WebRTC validados no hardware correspondente |
| T07 | Hot-plug | HARDWARE_CERTIFICATION | Inserção/remoção/reinserção sem crash, leak ou estado fantasma |
| T08 | XRUN/recovery | RUNTIME_VALIDATED + HARDWARE | XRUN injetado/observado, recuperação bounded, métricas e áudio posterior |
| T09 | Latência E2E/p99 | RUNTIME_VALIDATED + HARDWARE | Harness com timestamps monotônicos, amostra mínima definida, p50/p95/p99 e limite aprovado |
| T10 | Soak 5/15/60 min | SOFTWARE_RELEASE_GATE + HARDWARE | Software: soak bounded de 60 min com métricas e exit code; hardware: 5/15/60 min físicos separados, sem misturar evidência |
| T11 | Reboot/recovery | RUNTIME_VALIDATED + HARDWARE | reboot controlado, serviço retorna, estado seguro, áudio recupera ou falha explicitamente; evidência não bloqueia software release |
| T12 | Térmica/energia | HARDWARE_CERTIFICATION | temperatura, throttling, consumo e brownout medidos durante soak |
| T13 | Release v0.3.1 | SOFTWARE/PACKAGE_RELEASE_GATE | T00, T14, T15 e T16 aplicáveis PASS; artefatos, changelog, SBOM, assinatura, checksums, gates e aprovação explícita; hardware permanece separado |
| T14 | Install/upgrade/remove/rollback | PACKAGE_RELEASE_GATE | lifecycle repetível em amd64/arm64 aplicáveis, com artefatos reais ou cross-build documentado; preservação/remoção de dados documentada |
| T15 | Checksums/SBOM/Ed25519 | SOFTWARE_RELEASE_GATE | SHA-256, SBOM SPDX/CycloneDX, assinatura verificável e fingerprint publicado por canal independente, com cerimônia de assinatura auditável |
| T16 | Backend Windows | DECISION GATE | manter no escopo somente com decisão de produto; caso contrário mover para backlog futuro com ADR |

## Dependências críticas

```text
T15 -> T13
T14 -> T13
T03 -> T01 -> T02/T05
T04 -> T06/T07/T08/T12
T05 -> T01/T08/T09
T06 -> T07/T08/T12
T08 -> T10/T11
T09 -> T10
T14/T15/T16 -> T13 software release
T10 software soak -> T13; T10 physical soak remains certification-only
T03/T05 -> T01 runtime
T03/T04/T06/T07/T08/T09/T10 physical/T11/T12 remain hardware/runtime certification and never block software release
T16 -> ADR antes de qualquer implementação Windows
```

---

## Tasks executáveis

### T00 — Baseline e matriz de evidência

**Arquivos:** `docs/TODO.md`, `docs/DEVELOPMENT-HANDOFF.md`, `docs/ARCHITECTURE-GAPS.md`, novo ADR em `docs/adr/`.

- Congelar versões, hardware disponível, host de teste, topologia LAN e critérios de sucesso.
- Criar matriz com `NOT_STARTED`, `IN_PROGRESS`, `PASS`, `FAIL`, `BLOCKED`, `NOT_APPLICABLE`.
- Separar evidência `CODE`, `CI`, `DOCKER/QEMU/SIMULATED`, `RUNTIME_VALIDATED`, `HARDWARE_CERTIFICATION`.
- Não fechar nenhum item sem comando, log, timestamp, commit e artefato.

**Gate:** validador documental PASS; nenhum claim físico sem evidência física.

### T01 — Harness de WebRTC/Opus em dois peers

- Escrever teste RED com dois peers `str0m` e transporte controlado.
- Negociar SDP/ICE; não fabricar `media_mid`.
- Medir frames codificados, transmitidos, recebidos, decodificados, drops e reconnect.
- Separar teste Sans-IO de teste com socket/LAN real.

**Gate CODE:** teste reproduzível sem `sleep` e sem hardware, com dois peers Sans-IO. **Gate runtime:** processos/sockets e áudio reais, com host e evidência identificados; T05 cobre LAN real.

### T02 — DTLS-SRTP e segurança de sessão

- Verificar fingerprint esperado; rejeitar fingerprint errado/revogado.
- Registrar cipher/profile negociado e falha de downgrade.
- Testar replay, peer desconhecido e renegociação inválida.
- Fazer security review independente.

**Gate:** handshake real observado; testes positivos e negativos PASS.

### T03 — PipeWire/WirePlumber em host Linux físico

- Reusar graph virtual somente como pré-gate.
- Em host físico, coletar `uname`, `pw-cli`, nodes, links, formato PCM, taxa e canais.
- Executar captura → mix → reprodução; validar teardown e ausência de processos órfãos.

**Gate:** log bruto preservado; classificar host, dispositivo e versão.

### T04 — ALSA/USB físico

- Selecionar dispositivo por nome/ID/capability, nunca número fixo de card.
- Validar sample rate, channels, period/buffer, captura/reprodução.
- Separar teste USB físico de `snd-dummy`, Docker e loopback.

**Gate:** evidência física; simuladores não fecham item.

### T05 — Rede LAN real

- Usar dois hosts/processos reais em LAN isolada.
- Registrar RTT, perda, reorder, jitter, throughput e relógio monotônico.
- Executar cenário nominal e fault injection controlado em LAN isolada, com autorização por teste, limites de tráfego e blast radius, abort trigger e restauração de shaping/firewall.

**Gate:** pcap/log/telemetria, configuração da rede e validação de conectividade pós-teste preservados.

### T06 — Matriz Raspberry Pi 3/4/5 ARM64

- Validar, por dispositivo declarado como suportado, arquitetura, kernel, toolchain, instalação e serviço.
- Executar T01–T05 no Pi correspondente quando aplicável.
- Registrar temperatura, throttling, CPU, memória, energia e estabilidade.
- Se algum modelo não for testado, marcar esse claim `PENDING` ou estreitar suporte no ADR; não inferir Pi 3/4 a partir de Pi 5.

**Gate:** só hardware físico fecha cada claim de dispositivo.

### T07 — Hot-plug USB

- Executar inserção, remoção e reinserção de dispositivo USB real.
- Confirmar recuperação, limpeza de handles e ausência de estado fantasma.
- Executar somente em laboratório isolado, nunca em host de produção; preservar backup e procedimento de recuperação antes do teste.

**Gate:** evidência física; Docker, QEMU, `snd-dummy` e loopback não contam.

### T08 — XRUN e recuperação

- Criar cenário de underrun/overrun observável.
- Confirmar contadores, mute seguro, flush/reconnect e retorno de áudio.
- Testar repetição e falha persistente com stop conditions e abortamento seguro.

**Gate:** recuperação limitada, sem deadlock, sem crash, sem falso PASS por silêncio.

### T09 — Latência E2E e p99

- Definir origem/destino de timestamp e sincronização de relógios.
- Executar amostra mínima documentada em condições nominal e fault.
- Publicar p50/p95/p99, jitter, drops e intervalo de confiança.

**Gate:** não confundir latência de controle com áudio E2E.

### T10 — Soak 5/15/60 minutos

- Software release: rodar `OPENIEM_SOAK_SECONDS=3600` em ambiente reproduzível; não extrapolar smoke curto.
- Hardware certification: rodar exatamente 300, 900 e 3600 segundos em cada duração; não misturar logs físicos com software gate.
- Monitorar RSS, FD, threads, CPU, XRUN, drops, reconnect, temperatura e crash.
- Fazer teardown e verificar processos/portas/arquivos residuais.

**Gate:** software tem log/exit code de 3600 segundos; cada duração física tem log/exit code próprios.

### T11 — Reboot/recovery

- Testar reboot limpo e interrupção abrupta controlada em laboratório isolado.
- Confirmar serviço habilitado, estado seguro, rebind de dispositivos e retorno de áudio.
- Validar idempotência de recovery; definir stop conditions e recuperação manual antes de interromper host.

**Gate:** evidência de boot, journal, healthcheck e áudio posterior; runtime/hardware only, não requisito de software release.

### T12 — Térmica e energia

- Medir temperatura, throttling, consumo, tensão e eventos de brownout durante T10/T11.
- Identificar equipamento de medição e limites de segurança.
- Abortar em condição perigosa; não executar em host ou fonte de produção.

**Gate:** medição física identificada; sem estimar consumo por CPU.

### T13 — Release v0.3.1

- Fechar todos os itens `SOFTWARE_RELEASE_GATE`: Rust/frontend/security, DSP/PCM/virtual ALSA, Opus/WebRTC software, amd64/arm64 package validators, clean-container lifecycle, systemd ownership/permissions, ARM64 userspace smoke, T10 software soak de 3600 segundos, documentation/checksum/provenance.
- Fechar T00, T14, T15 e decisão T16 antes de release; T16 pode ser `NOT_APPLICABLE` para Windows. Hardware T03/T04/T06–T12 permanece separado e não bloqueia software release.
- Verificar tag, artefatos, checksums, SBOM, assinatura, notas e rollback.
- Exigir confirmação explícita antes de publicar release; definir owner, artefato de rollback e aprovação da cerimônia de assinatura.

**Gate:** release só após aprovação; sem publicar durante desenvolvimento autônomo.

### T14 — Lifecycle de pacote

- Testar install, reinstall, upgrade, downgrade/rollback, uninstall e purge.
- Verificar preservação de config/data e remoção do que é declarado removível.
- Executar amd64 e arm64; distinguir container de host físico.

**Gate:** artefato real validado com `dpkg-deb --control` e logs por operação.

### T15 — Checksums, SBOM e Ed25519

- Gerar SHA-256 por artefato e SBOM SPDX ou CycloneDX com versão/pinning.
- Assinar manifest com Ed25519 em ambiente de assinatura protegido, sem chave privada no repo; usar operador autorizado, armazenamento não exportável quando possível, dual control, rotação, revogação e auditoria.
- Publicar chave pública/fingerprint por canal independente.
- Testar assinatura válida, arquivo alterado, assinatura errada e chave ausente.

**Gate:** verifier independente reproduz PASS/FAIL; publicação exige autorização explícita.

### T16 — Decisão de escopo Windows

- Decidir se WASAPI/ASIO continua no MVP.
- Se sim: especificar backend, APIs, CI Windows, testes físicos e critérios.
- Se não: mover para backlog futuro e remover qualquer claim de suporte.

**Gate:** ADR aprovado; não implementar backend por suposição.

## Definition of Done — 100%

Desenvolvimento só está concluído quando TODOS os critérios abaixo forem verdadeiros:

### Produto e arquitetura

- [ ] T00 matriz de evidência atualizada e sem claims stale.
- [ ] Controle, mídia, áudio físico, runtime e hardware continuam separados.
- [ ] ADR Windows fecha decisão de escopo.
- [ ] Todos os GAPs aplicáveis têm owner, dependência, critério e evidência.

### Software e testes

- [ ] `cargo fmt --manifest-path server/Cargo.toml --all -- --check` PASS.
- [ ] `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings` PASS.
- [ ] `cargo test --manifest-path server/Cargo.toml` PASS.
- [ ] Frontends Musician/Engineer typecheck, test e build PASS.
- [ ] Security scan e revisão independente PASS.
- [ ] WebRTC/Opus E2E em dois peers PASS sem fixture artificial.
- [ ] DTLS-SRTP real PASS com fingerprint/cipher/replay/negative tests.

### Runtime e rede

- [ ] PipeWire/WirePlumber físico PASS.
- [ ] ALSA/USB físico PASS.
- [ ] Rede LAN real PASS.
- [ ] XRUN/recovery PASS.
- [ ] Latência E2E p99 medida e dentro do limite aprovado.
- [ ] Soak real de 5, 15 e 60 minutos executado exatamente nesses tempos.
- [ ] Reboot/recovery PASS.

### Hardware

- [ ] Cada claim Pi 3 baseline e Pi 4/5 compatibilidade tem instalação e validação física próprias; claim não testado permanece PENDING.
- [ ] Hot-plug USB PASS.
- [ ] Térmica, energia, throttling e brownout medidos.
- [ ] Nenhuma afirmação física baseada somente em Docker, QEMU, CI ou loopback.

### Release e distribuição

- [ ] install/reinstall/upgrade/rollback/uninstall/purge PASS em amd64 e arm64 aplicáveis.
- [ ] Checksums SHA-256 publicados e verificados.
- [ ] SBOM gerado, válido e associado ao artefato.
- [ ] Manifest assinado com Ed25519 e verificado por chave pública independente.
- [ ] Tag `v0.3.1` e release publicados somente após confirmação explícita.
- [ ] Runbook de rollback executado ou marcado explicitamente como não executado.

### Estado final

- [ ] Todos os itens T00–T16 aplicáveis têm evidência anexada para declarar conclusão geral 100%; itens não aplicáveis têm ADR.
- [ ] `SOFTWARE_RELEASE_GATE` pode PASSAR sem hardware; `PLATFORM_100_COMPLETE` exige runtime/hardware aplicáveis e não pode ser confundido com release.
- [ ] CI validado no SHA exato da PR.
- [ ] Nenhuma PR duplicada.
- [ ] Nenhum merge executado sem autorização explícita.
- [ ] Workspace limpo.
- [ ] Relatório final lista comandos, artefatos, limitações e owner de cada evidência.

## Critério de bloqueio

Se qualquer item exigir hardware, rede física, secret de assinatura, confirmação de release ou acesso externo indisponível, marcar `BLOCKED/PENDING`; itens de hardware não bloqueiam `SOFTWARE_RELEASE_GATE` com evidência concreta. Não marcar 100%, não fabricar resultado e não substituir por simulação. Testes destrutivos exigem laboratório isolado, backup restaurável previamente testado, limites de energia/corrente, blast radius documentado, stop conditions, procedimento de recuperação, owner de abortamento e autorização operacional explícita. Fault injection exige restaurar shaping/firewall e validar conectividade pós-teste. Cerimônia de assinatura exige geração offline, armazenamento/backup criptografado, operador autorizado, quorum, logs sem segredos, revogação e resposta a comprometimento.

## Ordem de execução recomendada

1. T00 baseline.
2. T01–T02 WebRTC/DTLS-SRTP software/runtime.
3. T03–T05 host Linux e LAN.
4. T07–T11 hot-plug, recovery, latência, soak e reboot.
5. T06 hardware Pi 3 baseline e Pi 4/5; T12 térmica e energia, quando aplicáveis.
6. T14–T15 lifecycle, checksums e supply chain.
7. T16 decisão Windows.
8. T13 release software, somente com confirmação; T03/T04/T06–T12 físicos não bloqueiam esse gate.
