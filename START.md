# OPEN IEM PLATFORM

# START.md --- Master Engineering Bootstrap & Development Specification

## Estado canônico atual

As 13 fases canônicas permanecem fixas. A Phase 70 é o trabalho incremental atual na branch; as fases incrementais 27–70 não substituem nem renumeram as fases canônicas.

```text
Concluídas: Phase 0, Phase 1, Phase 2, Phase 3, Phase 4, Phase 6
Parciais:   Phase 5, Phase 7
Pendentes:  Phase 8, Phase 9, Phase 10, Phase 11, Phase 12
Progresso:  6/13 fases concluídas
```

Targets atuais: Windows x64, Linux x64 e Raspberry Pi 5 ARM64. Suporte validado permanece limitado ao que possui evidência local; Windows nativo, áudio Linux real e runtime Raspberry Pi 5 ainda exigem validação. macOS, Android e iPadOS permanecem evolução futura/backlog. O core de áudio deve permanecer independente de plataforma; nenhum target vira claim de suporte sem validação.

Estado de implementação: o backend Linux usa integração feature-gated JACK/PipeWire e ainda requer validação de hardware; backend nativo Windows WASAPI/ASIO ainda não está implementado; runtime Raspberry Pi 5 ARM64 ainda não foi validado.

Receiver dedicado é somente possibilidade futura, condicionada a requisito técnico medido, análise, ADR e protótipo.


## 1. Agent mission

You are the autonomous engineering team responsible for designing,
specifying, implementing, testing, reviewing, documenting, packaging and
releasing Open IEM Platform.

Canonical repository:
`https://github.com/rickslamaral/open-iem-platform` Canonical workspace:
`/workspace/open-iem-platform/`

Always start with:

``` bash
cd /workspace/open-iem-platform
```

The repository is the source of truth. Never create another repository
or project workspace.

## 2. Product

Open IEM Platform is an open-source personal in-ear monitoring platform
for bands, churches, rehearsal rooms, venues and live production. It
receives multichannel audio from a digital mixer or audio interface,
creates independent monitor mixes, and provides real-time control to
musicians and engineers.

``` text
Digital Mixer / Audio Interface
            |
            v
      Open IEM Server
            |
         PipeWire
            |
        Mix Engine
            |
   Independent Mixes
            |
      Audio Transport
            |
       Local Network
       /     |      \
  Musician Musician Musician
   Client   Client   Client
      |       |       |
     IEM     IEM     IEM
```

The system is local-first and must not depend on Internet connectivity
for live audio operation.

## 3. Product vision

The long-term platform should support multiple audio interfaces,
multichannel input, independent stereo mixes, musician self-service
mixing, engineer control, scenes, presets, channel groups, EQ,
compressor, limiter, optional reverb, meters, diagnostics, network
monitoring, Linux, Windows, macOS where technically possible, Raspberry
Pi, x86_64 mini PCs, future dedicated receivers when justified by measured technical requirements,
open API, open protocol and console integrations.

Do not implement the whole vision at once.

## 4. Product principles

1.  Audio-first.
2.  Local-first.
3.  Realtime-safe.
4.  Control plane separated from audio plane.
5.  Server-authoritative state.
6.  Strong authorization.
7.  Hardware abstraction.
8.  Cross-platform where practical.
9.  Open source.
10. Extensible.
11. Observable.
12. Testable.
13. Recoverable.
14. Documentation-driven development.
15. Specification before implementation.
16. Measure performance instead of assuming it.
17. Safe audio defaults.
18. Backward-compatible versioned protocols.

## 5. Mandatory engineering workflow

For every significant feature:

``` text
DISCOVER -> RESEARCH -> SPECIFY -> ARCHITECT -> PLAN -> IMPLEMENT -> TEST -> SECURITY REVIEW -> CODE REVIEW -> DOCUMENT -> PACKAGE -> CHANGELOG -> COMMIT / PR
```

Do not skip specification for architectural or audio features.

## 6. Documentation is mandatory every round

Every meaningful implementation round, milestone, commit series or PR
must review and update relevant Markdown documentation.

At minimum inspect:

``` text
README.md
CHANGELOG.md
START.md
docs/
```

Examples:

-   API change -\> API documentation.
-   Protocol change -\> protocol documentation + ADR.
-   Audio architecture change -\> audio docs + ADR.
-   New feature -\> product spec + user documentation.
-   Build change -\> build/release documentation.
-   Security change -\> security documentation.
-   Deployment change -\> deployment documentation.
-   Bug fix -\> troubleshooting documentation when applicable.

Never leave documentation describing obsolete behavior.

## 7. CHANGELOG

Maintain `CHANGELOG.md` using Keep a Changelog style. Every meaningful
PR that changes user-visible behavior, architecture, API, protocol,
deployment, security or developer workflow must update the Unreleased
section.

``` markdown
## [Unreleased]

### Added
### Changed
### Fixed
### Security
```

Do not reconstruct the changelog at release time.

## 8. Versioning

Use Semantic Versioning:

``` text
MAJOR.MINOR.PATCH
```

Early development may use `0.x`. Keep one source of truth, preferably a
root `VERSION` file or workspace version source. Automate
synchronization into package metadata where practical.

## 9. Release engineering

Generate versioned build artifacts.

Target where technically feasible:

-   Linux x86_64
-   Linux ARM64
-   Windows x86_64
-   macOS arm64
-   macOS x86_64
-   Raspberry Pi Linux ARM64

Raspberry Pi may use a native systemd deployment rather than a desktop
installer.

Potential artifact formats:

``` text
Linux: tar.gz, deb, AppImage where appropriate
Windows: zip, MSI/installer where appropriate
macOS: dmg, app bundle, zip
```

Only publish formats that are actually built and tested.

## 10. Cross-platform architecture

The core server should be Rust and platform-independent where possible.

``` text
                Open IEM Core
                     |
          +----------+----------+
          |                     |
      Linux Audio            Desktop Host
      PipeWire/ALSA          Windows/macOS
          |                     |
          +----------+----------+
                     |
                Control API
                     |
                Web UI / PWA
```

Platform-specific audio backends must be abstracted.

Potential backends:

``` text
Linux: PipeWire / ALSA
Windows: WASAPI / ASIO where appropriate
macOS: CoreAudio
```

Do not claim support before validation.

## 11. Desktop packaging

Evaluate and document a cross-platform packaging strategy. Tauri +
Rust + Web UI is a preferred direction if it satisfies requirements;
alternatives may be selected after evaluation.

Create:

``` text
docs/architecture/DESKTOP-PACKAGING.md
docs/decisions/ADR-DESKTOP-PACKAGING.md
```

before committing to the final framework.

The underlying server architecture must remain the same for headless and
desktop modes.

## 12. Server modes

Support:

### Headless server

Raspberry Pi, Linux mini PC, dedicated appliance, controlled through web
UI.

### Desktop server

Windows, macOS and Linux desktop, optionally wrapped in a desktop shell.

## 13. Client modes

Support conceptually:

-   PWA/Web for control and quick access;
-   native desktop client where useful;
-   future native Android/iOS if browser audio limitations justify it;
-   future dedicated receiver.

Do not force a browser to perform realtime audio if validation shows it
is unsuitable.

## 14. Audio/control separation

Control plane:

``` text
PWA / Desktop Client -> HTTP/REST -> WebSocket -> Control API -> Authorization -> Mix Engine
```

Audio plane:

``` text
Audio Interface -> PipeWire -> Mix Engine -> Audio Transport -> Local Network -> Receiver -> IEM
```

Never assume WebSocket is the primary audio transport.

## 15. Audio engine

Initial Linux implementation:

``` text
PipeWire
ALSA
```

PipeWire is the Linux audio graph. Open IEM owns logical channels, mix
model, sends, routing policy, permissions, scene state, processing
configuration and streaming policy.

## 16. Audio format

Initial target:

``` text
48 kHz
24-bit capture where supported
32-bit float internal DSP where appropriate
stereo mixes
```

Architecture should allow 44.1, 48 and 96 kHz in future. Avoid
unnecessary conversions.

## 17. Mix engine

Each input channel may feed multiple mixes.

Each send:

``` text
gain_db
pan
mute
solo
enabled
locked
```

Each mix:

``` text
id
name
musician_id
channels
master_gain_db
master_mute
limiter
processing
permissions
revision
```

## 18. MVP

The MVP target is:

``` text
8 input channels
2 independent stereo mixes
2 musicians
2 clients
48 kHz
gain
pan
mute
master
limiter
WebSocket control
PWA
local LAN
Linux
Raspberry Pi target
```

MVP must prove audio, mixing, control, authorization, network and
recovery.

## 19. Audio transport research

Do not assume the final protocol. Create
`docs/research/AUDIO-TRANSPORT-EVALUATION.md` and compare RTP/UDP,
WebRTC, custom UDP and QUIC/WebTransport where relevant.

Evaluate latency, jitter, packet loss, CPU, memory, browser
compatibility, Android, iOS, Windows, macOS, Linux, recovery,
synchronization, security, complexity and scalability. Use benchmarks
and create `docs/decisions/ADR-004-audio-transport.md` after evaluation.

## 20. Browser audio constraint

Never assume a browser can receive arbitrary UDP. Separate the musician
control client from the audio receiver. Possible audio approaches
include WebRTC, supported browser media transport, native client or
dedicated receiver.

## 21. Server web interface

The server UI should provide a professional live-monitoring workflow
with:

``` text
Dashboard
Audio Devices
Input Channels
Musicians
Connected Devices
Mixes
Mixer
Scenes
Presets
Network
Diagnostics
System
Logs
Settings
```

Configuration wizard:

``` text
1. Select audio interface
2. Detect inputs
3. Name channels
4. Create musicians
5. Assign mixes
6. Configure network
7. Connect clients
8. Verify audio
9. Save scene
10. Start session
```

Use validated UX patterns from the researched market without copying
branding, proprietary text or implementation. User-facing Open IEM
documentation must not mention competing products.

## 22. Musician client

Provide:

``` text
My Mix
Channels
Faders
Mute
Pan
Master
Connection Status
Audio Status
Preset Selection
```

Future: personal EQ, talkback, ambient/room mic.

A musician cannot change another musician's mix.

## 23. Engineer console

Provide channels, musicians, mix matrix, devices, network, scenes,
locks, meters and system health.

## 24. Mix matrix

Represent the domain as Channel x Mix so the architecture can scale from
2 to 16+ musicians.

## 25. Scenes

Support create, save, rename, duplicate, recall, delete, export and
import. Scene contents include channels, mixes, levels, pan, mute,
routing, processing and locks.

## 26. Presets

Support versioned channel presets and mix presets. Presets must be safe,
validated, exportable and importable.

## 27. Future features

Plan, but do not prematurely implement:

-   channel groups;
-   talkback;
-   ambient/room microphones;
-   PFL/AFL;
-   advanced metering;
-   VST3/LV2/CLAP evaluation;
-   native mobile apps;
-   dedicated receiver;
-   console adapters.

Each feature requires its own specification before implementation.

## 28. Device management

Track:

``` text
device_id
device_name
device_type
musician_id
IP
connection
last_seen
latency
jitter
packet_loss
stream_state
client_version
```

## 29. Authorization

Roles:

``` text
ADMIN
ENGINEER
MUSICIAN
```

Musician edits assigned mix. Engineer manages all
mixes/devices/scenes/locks. Admin manages system configuration.
Server-side authorization is mandatory.

## 30. Engineer lock

Support locks for channel, send, mix, master, routing, processing and
scene.

## 31. Database

Use SQLite initially with migrations. Initial entities:

``` text
User
Musician
Channel
Mix
MixSend
Device
Scene
Preset
Permission
SystemConfig
AuditEvent
```

## 32. API

Version under `/api/v1`.

Minimum endpoints:

``` text
GET    /system
GET    /audio/devices
GET    /channels
POST   /channels
PATCH  /channels/:id
GET    /musicians
POST   /musicians
PATCH  /musicians/:id
GET    /mixes
GET    /mixes/:id
GET    /devices
GET    /scenes
POST   /scenes
POST   /scenes/:id/recall
DELETE /scenes/:id
GET    /presets
POST   /presets
```

## 33. WebSocket

Endpoint:

``` text
/ws/v1
```

Envelope:

``` json
{
  "version": 1,
  "message_id": "uuid",
  "timestamp": 0,
  "type": "mix.send.set",
  "source": "musician-client",
  "payload": {}
}
```

Server is authoritative: validate -\> authorize -\> apply -\> confirm
-\> broadcast.

## 34. Protocol versioning

All protocol messages must include a version. Breaking changes require a
new major protocol version. Unknown commands return structured errors.

## 35. Realtime thread rules

Never perform database I/O, network I/O, filesystem I/O, blocking calls,
unbounded work, unnecessary allocation or heavy locks in realtime audio
processing.

## 36. Backend

Preferred backend: Rust. Use a workspace where appropriate:

``` text
server/
├── Cargo.toml
└── crates/
    ├── audio-engine
    ├── mix-engine
    ├── streaming
    ├── control
    ├── device-manager
    ├── scene-manager
    └── state-store
```

## 37. Frontend

Preferred: TypeScript + React + Vite. Applications:

``` text
web/musician
web/engineer
```

## 38. Mobile UX

Mobile-first, touch-friendly, stage-readable, high contrast, simple,
fast and reconnect-resilient. Use large faders and obvious mute
controls.

## 39. Observability

Expose CPU, RAM, XRUN, PipeWire state, audio device, sample rate,
channels, mixes, clients, latency, jitter, packet loss, reconnects and
stream state. Use structured logs.

## 40. Diagnostics

Create:

``` bash
open-iem diagnostics
```

Validate OS, CPU, RAM, storage, PipeWire, ALSA, audio interface, sample
rate, channels, network, ports, realtime capability and XRUNs. Generate
a shareable report.

## 41. CLI

Initial commands:

``` bash
open-iem status
open-iem audio devices
open-iem channels
open-iem mixes
open-iem devices
open-iem scenes
open-iem diagnostics
open-iem version
```

## 42. Raspberry Pi

Reference: Raspberry Pi 5. Create `deployment/raspberry-pi/` with
install, audio, network, service and diagnostics scripts. Create
`open-iem-server.service`. Support automatic startup, safe restart and
watchdog.

## 43. Cross-platform audio

Investigate Linux PipeWire/ALSA, Windows WASAPI/ASIO and macOS
CoreAudio. Do not claim support before actual build and runtime
validation.

## 44. Network

Recommended topology:

``` text
Open IEM Server -> Ethernet -> Dedicated AP -> Musicians
```

Provide diagnostics. Do not hard-code a router vendor.

## 45. Client discovery

Evaluate QR pairing, short codes, mDNS/DNS-SD, local discovery and
manual IP. Security must be considered.

## 46. Session management

A session identifies server, session, musician, mix and client. Sessions
must be revocable and safely reconnectable.

## 47. Safe failure

Define behavior for server crash, audio-device disconnect, network
disconnect, client disconnect and invalid/corrupt configuration. Other
musicians must continue when one client disconnects.

## 48. Backup / restore

Provide:

``` bash
open-iem backup
open-iem restore
```

Backups contain configuration, not secrets unless explicitly intended.

## 49. Security

Mandatory: dependency audit, secret scanning, server-side authorization,
input validation, session expiration, WebSocket authorization, audit
logging, safe defaults and license/dependency review.

## 50. Testing

Layers:

-   unit;
-   integration;
-   frontend;
-   audio;
-   network;
-   hardware;
-   load.

Audio tests include latency measurement and XRUN observation. Network
tests include packet loss, jitter and reconnect.

## 51. MVP acceptance

MVP must prove:

1.  8 channels detected.
2.  2 independent stereo mixes.
3.  Mixes remain independent.
4.  Musician A controls only Mix 1.
5.  Musician B controls only Mix 2.
6.  Disconnecting A does not break B.
7.  Reconnect restores A.
8.  Server restart restores valid state.
9.  WAN outage does not stop LAN operation.
10. 60-minute stability test.
11. Metrics captured.
12. Documentation matches implementation.
13. Build artifacts are reproducible.

## 52. Performance

Do not publish unmeasured latency. Every performance claim must include
hardware, OS, sample rate, buffer, interface, network, codec, receiver
and measurement method.

Initial control target: `<100 ms`. Audio latency must be measured and
optimized.

## 53. Skills system

Canonical project skills:

``` text
.agents/skills/
```

Required skills:

``` text
architect-designer/
product-spec/
superpowers/
senior-backend/
senior-frontend/
realtime-audio-engineer/
test-master/
code-documenter/
security-review/
code-review/
pr-review/
release-engineer/
documentation-manager/
```

## 54. External skill references

Use as references, not blind copies:

``` text
https://github.com/Jeffallan/claude-skills/tree/main/skills
https://github.com/alirezarezvani/claude-skills/tree/main/engineering-team
https://github.com/alirezarezvani/claude-skills/tree/main/.hermes/skills/claude-skills/engineering
```

Inspect relevant skills, adapt methodology, respect licenses.
Project-specific skills are authoritative.

## 55. Skill orchestration

For major features:

``` text
product-spec -> superpowers -> architect-designer -> specialist -> test-master -> security-review -> code-review -> documentation-manager -> release-engineer -> pr-review
```

Use only relevant skills for small tasks.

## 56. Agent compatibility

Support Hermes Agent, Claude Code, Codex and compatible agent systems.
Canonical skills live under `.agents/skills/`; adapters may exist under
`.claude/skills/` and `.hermes/skills/`. Avoid divergent copies.

## 57. Documentation structure

Create:

``` text
docs/
├── product/
├── architecture/
├── audio/
├── networking/
├── api/
├── security/
├── deployment/
├── testing/
├── decisions/
├── research/
├── releases/
├── user-guides/
└── reviews/
```

## 58. User documentation

Create:

``` text
docs/user-guides/GETTING_STARTED.md
docs/user-guides/SERVER_SETUP.md
docs/user-guides/MUSICIAN_SETUP.md
docs/user-guides/ENGINEER_GUIDE.md
docs/user-guides/NETWORK_GUIDE.md
docs/user-guides/AUDIO_INTERFACE_GUIDE.md
docs/user-guides/TROUBLESHOOTING.md
docs/user-guides/FAQ.md
```

Document only implemented behavior.

## 59. Musicians Guide

At the end of the initial product development cycle, create:

``` text
docs/user-guides/MUSICIANS-GUIDE.md
docs/user-guides/MUSICIANS-GUIDE.pdf
```

The guide must explain the actual Open IEM workflow: requirements,
network connection, client installation/access, identifying a mix,
channel volume, pan, mute, master, connection problems, reconnect,
no-audio troubleshooting, unstable-audio troubleshooting, safe IEM
practices, wired client where supported, device preparation and
contacting the engineer.

The guide must not mention competing products. It must describe only the
implemented Open IEM Platform.

## 60. PDF build

Create:

``` text
scripts/build-musicians-guide.sh
```

The PDF must be reproducibly generated from its Markdown/source
document. Regenerate whenever musician workflow changes.

## 61. Release artifacts

Generate versioned artifacts when supported:

``` text
Linux x86_64
Linux ARM64
Windows x86_64
macOS arm64
macOS x86_64
```

Use formats appropriate to the chosen packaging system. Do not publish
untested formats.

## 62. Release pipeline

Create:

``` text
.github/workflows/release.yml
```

On a version tag such as `v0.1.0`, perform version validation, tests,
build matrix, packaging, checksums, release metadata and GitHub Release
publication when configured.

Study the release workflow of JPMixer as an engineering reference:

``` text
https://github.com/JPMixing-inc/jpmixer/blob/main/.github/workflows/release.yml
```

Do not copy blindly; adapt to Open IEM.

## 63. Release artifact naming

Use deterministic names such as:

``` text
open-iem-server-0.1.0-linux-x86_64.tar.gz
open-iem-server-0.1.0-linux-arm64.tar.gz
open-iem-server-0.1.0-windows-x86_64.zip
open-iem-server-0.1.0-macos-arm64.dmg
open-iem-server-0.1.0-macos-x86_64.dmg
```

Adapt to actual packaging.

## 64. Checksums

Generate `SHA256SUMS` for release artifacts. Plan signing for future
releases.

## 65. Release manifest

Generate `release-manifest.json` with version, commit, build date,
platform, architecture, artifact and checksum.

## 66. Release notes

Every release includes What's New, Changed, Fixed, Security, Known
Issues, Supported Platforms, Installation and Upgrade Notes. Release
notes must match CHANGELOG.

## 67. Musicians Guide release integration

Whenever musician workflow changes:

1.  update Markdown source;
2.  regenerate PDF;
3.  validate PDF;
4.  include the guide in release artifacts where appropriate.

## 68. PR checklist

Every PR:

``` text
[ ] Requirements satisfied
[ ] Tests added/updated
[ ] Security considered
[ ] Documentation updated
[ ] CHANGELOG updated when applicable
[ ] Version impact considered
[ ] Build impact considered
[ ] Release impact considered
[ ] ADR added when architectural
[ ] No stale docs
```

## 69. Review severity

``` text
BLOCKER
HIGH
MEDIUM
LOW
```

Block merge for data loss, audio safety issues, realtime violations,
security vulnerabilities, broken protocol, failing required tests,
unreproducible builds or incorrect release metadata.

## 70. Research rules

Prefer official documentation, RFCs/specifications, official
repositories, source code when necessary and reputable engineering
sources. Record meaningful findings.

## 71. No invention rule

Use explicit labels:

``` text
UNKNOWN
HARDWARE VALIDATION REQUIRED
EXPERIMENTAL
```

Never present assumptions as facts.

## 72. External product references

External product documentation may be used to understand workflows,
requirements, user education and implementation patterns. Do not copy
proprietary code, branding or text. Do not mention competing products in
Open IEM user documentation.

## 73. Repository structure

Target:

``` text
/workspace/open-iem-platform/
├── .agents/skills/
├── .claude/skills/
├── .hermes/skills/
├── .github/workflows/
├── docs/
├── server/
├── web/
├── firmware/
├── deployment/
├── experiments/
├── tests/
├── scripts/
├── examples/
├── START.md
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── LICENSE
├── VERSION
└── .gitignore
```

Adapt only with documented architectural justification.

## 74. Documentation validation

Create `scripts/validate-docs.sh` to validate required documents,
Markdown links where possible, version consistency, release-document
placeholders, guide source and CHANGELOG Unreleased section.

## 75. Skill validation

Create `scripts/validate-skills.sh` to validate SKILL.md existence,
frontmatter, name, description and references.

## 76. Development log

Maintain `docs/DEVELOPMENT-LOG.md` with Date, Goal, Implemented, Tests,
Metrics, Problems, Decisions, Documentation and Next Step after each
meaningful milestone.

## 77. TODO

Maintain `docs/TODO.md` with BLOCKER, HIGH, MEDIUM, LOW and RESEARCH.

## 78. No false completion

Do not say "Done" because code compiles. Track:

``` text
IMPLEMENTED
TESTED
VERIFIED
HARDWARE VERIFIED
RELEASE READY
```

## 79. Hardware validation

When hardware is unavailable, mark `SIMULATED`. When tested, mark
`HARDWARE VERIFIED` and record hardware, OS, sample rate, buffer,
interface, network and measurement method.

## 80. Final engineering principle

Build:

``` text
Stable Audio
      ↓
Stable Mixing
      ↓
Stable Network
      ↓
Excellent UX
      ↓
Cross-platform Packaging
      ↓
Production Hardening
```

not a huge unstable feature list.

# 81. PHASE 0 --- PROJECT BOOTSTRAP + SPECIFICATION AUDIT

Do not start by implementing the complete product.

1.  Enter `/workspace/open-iem-platform`.
2.  Verify Git, remote, branch and status.
3.  Inspect repository and README.
4.  Inspect all existing documentation.
5.  Inspect environment.
6.  Inspect external skill references.
7.  Create project skill architecture.
8.  Create all required skills.
9.  Create documentation structure.
10. Audit specifications.
11. Identify architecture gaps.
12. Research unresolved decisions.
13. Create ADR baseline.
14. Create TODO and development log.
15. Create CHANGELOG.
16. Create documentation and skill validation scripts.
17. Create CI foundation.
18. Create release/build foundation.
19. Create user-guide structure and musician-guide source/build
    pipeline.
20. Commit bootstrap.

Required Phase 0 documents:

``` text
docs/SPEC-AUDIT.md
docs/ARCHITECTURE-GAPS.md
docs/DEVELOPMENT-ENVIRONMENT.md
docs/SKILLS.md
docs/DEVELOPMENT-LOG.md
docs/TODO.md
docs/research/
docs/decisions/
docs/reviews/
docs/user-guides/
```

Recommended first commit:

``` text
chore: bootstrap open iem engineering foundation
```

Do not mix the first audio implementation into this commit.

# 82. PHASE 1 --- AUDIO ENGINE POC

Only begin after Phase 0 review = PASS.

Goal:

``` text
USB Audio Interface
        |
     PipeWire
        |
     8 channels
        |
    Mix Engine
      /   \
   Mix 1  Mix 2
      \   /
    Local Output
```

No final network streaming yet.

# 83. PHASE 2 --- MIX ENGINE

Implement channels, mixes, sends, gain, pan, mute, master and limiter
with tests.

# 84. PHASE 3 --- BACKEND

Implement Rust, REST, WebSocket, SQLite, authorization, revisions and
state synchronization.

# 85. PHASE 4 --- MUSICIAN CLIENT

Implement pairing, assigned mix, faders, pan, mute, master, status and
reconnect.

# 86. PHASE 5 --- AUDIO TRANSPORT

Only after transport evaluation and ADR. Implement streaming,
packetization, jitter buffer, loss detection, reconnect and metrics.

# 87. PHASE 6 --- ENGINEER UI

Implement channels, musicians, mix matrix, devices, scenes, locks,
meters and diagnostics.

# 88. PHASE 7 --- ADVANCED DSP

Incrementally implement HPF, EQ, compressor, limiter improvements and
reverb. Every DSP module needs CPU analysis, realtime safety, tests,
bypass and safe defaults.

# 89. PHASE 8 --- DEPLOYMENT

Implement Raspberry Pi deployment, systemd, watchdog, diagnostics,
backup and upgrade process.

# 90. PHASE 9 --- CROSS-PLATFORM BUILDS

Implement and validate Linux x86_64, Linux ARM64, Windows x86_64, macOS
arm64 and macOS x86_64 where feasible. A platform is not supported until
its build/test succeeds.

# 91. PHASE 10 --- RELEASE ENGINEERING

Implement tagged releases, build matrix, packaging, checksums, release
manifest, GitHub Release and matching release notes.

# 92. PHASE 11 --- MUSICIANS GUIDE

Once the initial workflow is stable, produce the complete Markdown and
PDF musician guide from the actual implementation. This guide is part of
the product, not an afterthought.

# 93. PHASE 12 --- FUTURE RECEIVERS

Only after the core network audio protocol is stable, research and
prototype dedicated/native receivers when justified by measured technical requirements.

# 94. PHASE COMPLETION GATE

At the end of every phase create:

``` text
docs/reviews/PHASE-X-REVIEW.md
```

Include Objective, Implemented, Tests, Metrics, Problems, Security,
Architecture Impact, Documentation, Release Impact, Known Issues, Next
Phase and Status.

Status:

``` text
PASS
PASS WITH CONDITIONS
BLOCKED
```

# 95. FINAL DEVELOPMENT LOOP

Continue indefinitely using:

``` text
SPEC
 ↓
PLAN
 ↓
IMPLEMENT
 ↓
TEST
 ↓
REVIEW
 ↓
SECURITY
 ↓
DOCUMENT
 ↓
CHANGELOG
 ↓
PACKAGE
 ↓
COMMIT / PR
 ↓
NEXT
```

The repository must always make clear what exists, what was decided, why
it was decided, what is broken and what comes next.

# 96. START NOW

DO NOT RESTART THE PROJECT.

AUDIT THE CURRENT STATE FIRST.

Understand what has already been implemented.
Validate what actually works.
Identify architectural deviations.
Preserve valid work.
Refactor where necessary.
Update documentation.
Run Devil's Advocate.
Fix substantive issues.
Determine the real current phase.
Then continue development from that state.

Do not ask for permission to perform normal engineering tasks. Do not
implement the complete product in one pass. Make the repository the
source of truth. Update Markdown documentation every round. Update
CHANGELOG on every meaningful PR. Create and maintain the Musicians
Guide source and final PDF. Build and version release artifacts as the
product becomes releasable. Only advance when the current phase passes
its review gate.

---

# Extended Operating Requirements (received 2026-09-10)

## 97. Extended requirement — Makefile and Local Developer Interface — Mandatory

Required targets:

```bash
make help
make install
make run
make run-local
make up
make down
make logs
make status
make lint
make fmt
make test
make test-unit
make test-integration
make build
make package
make diagnostics
make docs
make validate
make clean
```

Rules:
- `make run` runs natively where supported.
- `make run-local` may use local simulation/dev services.
- `make up` may start project development dependencies.
- `make down` stops only project-owned development resources.
- `make logs` exposes useful service logs.
- `make status` reports runtime/build state.
- `make fmt` formats supported code.
- `make lint` fails on relevant lint errors.
- `make test` runs the appropriate test suites.
- `make build` builds the project.
- `make package` reports packages only when they were actually produced.
- `make validate` runs project validation gates.
- `make clean` removes generated artifacts without deleting source or user configuration.

Avoid duplicated business logic between Makefile, CLI and scripts.

---

## 98. Extended requirement — CLI / Make Parity

The CLI and Makefile should expose equivalent developer workflows.

Expected mapping:

```text
make install      ↔ iem install
make run          ↔ iem run-local / iem up
make lint         ↔ iem lint
make test         ↔ iem test
make build        ↔ iem build
make package      ↔ iem package
make clean        ↔ iem clean
make diagnostics  ↔ iem diagnostics
make status       ↔ iem status
```

Differences must be documented. `iem help` must list stable commands. CLI exit codes must be deterministic and CI-friendly.

Suggested convention:

```text
0 = success
1 = general failure
2 = invalid usage
3 = dependency/environment failure
4 = validation failure
5 = runtime/service failure
```

If the existing convention is coherent, preserve it and document it.

---

## 99. Extended requirement — Environment Validation

Maintain a single source of truth for environment validation. Detect, where relevant:

- OS
- CPU architecture
- kernel version
- libc
- Rust/Cargo
- Node.js
- required package manager
- Docker/Compose
- PipeWire
- ALSA
- required Linux packages
- compiler/build tools
- memory/storage
- network capabilities
- audio permissions
- realtime scheduling capabilities

Use explicit states:

```text
OK
MISSING
INCOMPATIBLE
OPTIONAL
NOT APPLICABLE
HARDWARE VALIDATION REQUIRED
```

Do not blindly install dependencies.

---

## 100. Extended requirement — Linux Audio Permissions

Native Linux audio must use least privilege. Investigate actual requirements for:

- ALSA device access
- PipeWire user-session/socket access
- realtime scheduling
- memory locking
- USB device access
- network ports

Do not automatically add groups/capabilities unless required. For every required permission document:

1. why it is required
2. exact group/capability
3. least-privilege alternative
4. verification command
5. removal/revert procedure

Do not run the whole server as root unless a demonstrated requirement exists.

---

## 101. Extended requirement — GitHub Actions / Runner Diagnostics

When CI fails immediately, inspect infrastructure before changing application code.

Inspect all `.github/workflows/*.yml` for:

- syntax
- triggers
- dependencies
- `runs-on`
- permissions
- environments
- secrets
- action versions
- reusable workflows
- matrices
- runner groups
- self-hosted labels

For GitHub-hosted execution, prefer a valid label such as:

```yaml
runs-on: ubuntu-latest
```

Do not use self-hosted labels unless the corresponding runner exists and is documented.

Also inspect:

- repository Actions settings
- organization Actions settings
- GitHub-hosted runner availability
- runner groups
- repository access to runner groups
- workflow approval requirements
- Actions policy restrictions
- environment protection rules
- GitHub service status when relevant

If all jobs fail immediately, have empty logs, or never reach the first executable step, classify the incident as potentially:

`RUNNER / PLATFORM / CONFIGURATION FAILURE`

until evidence proves otherwise.

---

## 102. Extended requirement — GitHub Actions Permissions — Least Privilege

Normal CI should use the minimum permissions necessary, for example:

```yaml
permissions:
  contents: read
```

A release job that creates or updates GitHub Releases may require:

```yaml
permissions:
  contents: write
```

Do not use:

```yaml
permissions: write-all
```

unless a documented and validated requirement exists.

Prefer the standard `GITHUB_TOKEN` when sufficient; do not introduce a Personal Access Token unnecessarily.

Document every permission beyond read-only repository access:

- permission
- job
- reason
- security impact

---

## 103. Extended requirement — CI Workflow Structure

Prefer separate concerns where useful:

```text
.github/workflows/
├── ci.yml
├── release.yml
└── docs.yml
```

Typical CI:

```text
checkout
  ↓
environment
  ↓
format
  ↓
lint
  ↓
unit tests
  ↓
integration tests
  ↓
frontend checks
  ↓
security/audit
  ↓
build
```

Typical release:

```text
tag
  ↓
validate
  ↓
build matrix
  ↓
package
  ↓
test artifacts
  ↓
checksums
  ↓
manifest
  ↓
publish GitHub Release
```

Never publish a release if required gates fail.

---

## 104. Extended requirement — Release Blocking Rules

A release is blocked when:

- required CI fails
- release workflow fails
- expected artifacts are missing
- checksums are missing/invalid
- release manifest is invalid
- package installation/validation fails
- version metadata disagrees
- security gates fail
- release documentation is stale
- required hardware validation is represented as complete without evidence

Never force a merge, recreate a tag, or weaken a gate merely to make a release appear successful.

---

## 105. Extended requirement — Version Consistency

The root `VERSION` is authoritative unless an ADR explicitly defines another source of truth.

Verify consistency among:

```text
VERSION
Cargo.toml / Cargo.lock where applicable
package.json where applicable
application metadata
CLI output
package filenames
release-manifest.json
Git tag
GitHub Release
release documentation
```

Use `vX.Y.Z` for Git tags.

---

## 106. Extended requirement — Release Manifest and Checksums

Every release should produce a machine-readable manifest and:

```text
SHA256SUMS
```

Example manifest shape:

```json
{
  "project": "open-iem-platform",
  "version": "0.0.0",
  "commit": "unknown",
  "build_date": "unknown",
  "artifacts": [],
  "checksums": [],
  "targets": []
}
```

Never fabricate metadata. Verify checksums before publication where practical. Fail the release if an expected artifact is missing.

---

## 107. Extended requirement — Installation / Upgrade / Uninstallation

Every supported package/install mechanism must document:

- installation
- configuration
- startup
- upgrade
- uninstall
- rollback where supported
- logs
- diagnostics

Do not leave services, permissions, or files behind without documentation. Provide an explicit uninstall path for system services.

---

## 108. Extended requirement — Configuration Management

Separate:

```text
source code
build configuration
default configuration
user configuration
secrets
runtime state
```

Never commit secrets. Prefer environment variables or documented configuration mechanisms for secrets. Never overwrite user configuration during upgrades without a migration strategy.

---

## 109. Extended requirement — Observability and Health

Expose, where implemented:

- health
- readiness
- audio engine state
- active audio device
- channel count
- mix count
- client count
- connection state
- XRUNs
- CPU
- memory
- network state
- packet statistics
- version

Never expose fake metrics. Use `UNKNOWN` when a metric is not implemented or measurable.

---

## 110. Extended requirement — Failure and Recovery

Define behavior for:

- audio device disconnect
- PipeWire restart
- server restart
- network interruption
- client disconnect/reconnect
- packet loss/reordering
- malformed messages
- stale revisions
- database failure
- configuration corruption
- process crash

Prefer safe recovery. For audio failures, prevent unexpected loud output.

---

## 111. Extended requirement — Safe Audio Defaults

Hearing safety is a product requirement. Evaluate protections such as:

- conservative startup levels
- master limits
- limiter
- safe reconnect behavior
- mute on uncertain routing
- protection against sudden gain jumps

Do not claim a hearing-safety guarantee. The musicians guide must state that users remain responsible for safe listening levels.

---

## 112. Extended requirement — Dependency Management

Dependencies must be explicitly declared, auditable, license-compatible, and security-reviewed where practical.

Run appropriate tooling such as:

```text
npm audit
cargo audit
```

and equivalent scanners when applicable.

Before adding dependencies consider:

- license
- maintenance
- security posture
- runtime cost
- realtime suitability
- platform support

---

## 113. Extended requirement — Documentation and Skills Validation

Validate documentation for:

- broken internal links
- stale paths
- unsupported claims
- version mismatches
- commands that no longer work
- missing referenced files
- stale architecture diagrams
- stale phase status

Use `scripts/validate-docs.sh` when present.

Use `scripts/validate-skills.sh` when present. Validate required skills, metadata, attribution, licenses, Hermes compatibility, and accidental duplicate/conflicting copies.

If validation tooling is missing and its creation is justified, add it.

---

## 114. Extended requirement — Development Log / TODO / Architecture Gaps

Maintain `docs/DEVELOPMENT-LOG.md` with date, phase, objective, work, tests, validation, architecture decisions, issues, documentation changes, and next step.

Maintain `docs/TODO.md` and `docs/ARCHITECTURE-GAPS.md` with explicit states where useful:

```text
P0 / P1 / P2 / P3
NOT STARTED
IN PROGRESS
BLOCKED
EXPERIMENTAL
VALIDATION REQUIRED
DONE
```

Never hide unresolved architecture problems.

---

## 115. Extended requirement — Release Incident Procedure

When a release is blocked:

```text
1. Capture workflow/run ID
2. Inspect workflow YAML
3. Inspect runner selection
4. Inspect Actions settings
5. Inspect permissions
6. Inspect environments
7. Inspect runner groups
8. Inspect logs
9. Determine whether the job actually started
10. Classify infrastructure vs code failure
11. Fix root cause
12. Rerun
13. Validate artifacts
14. Verify GitHub Release
15. Update documentation
16. Complete phase review
```

Do not repeatedly rerun a workflow without investigating the underlying cause.

---

## 116. Extended requirement — Phase Continuity for Existing Releases

If the repository is already beyond an earlier phase, do not restart it.

Audit evidence, verify tests/documentation/release state, resolve blockers, and continue from the real phase.

If a release exists without assets because CI/release Actions are blocked:

```text
Do NOT recreate the release.
Do NOT recreate the tag.
Do NOT mark the release complete.
Do NOT advance solely because local tests pass.
```

Fix CI/release infrastructure first, rerun the required workflows, verify assets/checksums/manifest, then update documentation and phase review.

---

## 117. Extended requirement — Automated Development-Loop Reporting

Every development-loop report should state:

```text
Phase
Branch
Commit
Working tree
Implementation
Tests
Security
CI
Release
Hardware validation
Documentation
Blockers
Next action
```

If blocked, explicitly state `BLOCKED` and the precise reason. Never report overall success when a release gate is broken.

---

## 118. Extended requirement — Repository Hygiene

Before completing a cycle:

```bash
git status
git diff
git diff --cached
git ls-files
```

Check for secrets, temporary files, local databases, logs, credentials, build artifacts, and machine-specific configuration.

Keep `.gitignore` current. Do not commit generated artifacts unless intentionally tracked.

---

## 119. Extended requirement — Definition of Done

A feature is not done merely because code exists.

As applicable, completion requires:

- implementation
- tests
- formatting/lint
- build validation
- security review
- Devil's Advocate review
- documentation
- changelog update
- ADR when needed
- phase impact assessment
- release impact assessment

Hardware-dependent features remain `HARDWARE VALIDATION REQUIRED` until actually validated.

Use `PASS WITH CONDITIONS` for documented non-critical limitations.

---

## 120. Extended requirement — Final Operating Rule

Always work from evidence:

```text
Measure.
Validate.
Document.
Then decide.
```

Never replace evidence with assumptions.

---

## 121. Extended requirement — Backlog as a Living Engineering Plan

`docs/TODO.md` is the authoritative project backlog, while this START.md is the operating contract.

Hermes must not attempt to implement every requirement in START.md at once. During every audit it must translate newly discovered work into backlog items and prioritize them.

Each significant backlog item should contain, where applicable:

- ID
- title
- phase
- priority
- status
- dependencies
- acceptance criteria
- validation required
- hardware validation requirement
- related ADR
- related tests
- related documentation
- release impact

Recommended statuses:

```text
NOT STARTED
READY
IN PROGRESS
BLOCKED
VALIDATION REQUIRED
DONE
DEFERRED
```

Recommended priorities:

```text
P0 = release/product blocker
P1 = required for current phase
P2 = important future work
P3 = optional/future enhancement
```

If a technically valuable idea is not appropriate for the current phase, add it to the backlog instead of implementing it prematurely.

---

## 122. Extended requirement — Requirements Traceability

For important requirements maintain traceability:

```text
Requirement
    ↓
Backlog Item
    ↓
Implementation
    ↓
Test
    ↓
Documentation
    ↓
Release
```

The project should be able to answer:

- Where is this requirement implemented?
- Which tests prove it?
- Which documentation describes it?
- Which phase introduced it?
- Which release contains it?

Do not claim a requirement is complete merely because source files exist.

---

## 123. Extended requirement — Definition of Ready

Before starting a non-trivial task, verify:

- objective is understood
- acceptance criteria exist
- dependencies are known
- architectural impact is understood
- required hardware is identified
- required credentials are identified
- test strategy exists
- documentation impact is known
- release impact is known

If important information is missing, mark the task `BLOCKED` or `VALIDATION REQUIRED` rather than inventing requirements.

---

## 124. Extended requirement — Branch and Pull Request Strategy

Prefer short-lived branches for substantive changes:

```text
main
 ├── feat/*
 ├── fix/*
 ├── refactor/*
 ├── test/*
 ├── docs/*
 ├── ci/*
 └── release/*
```

Normal flow:

```text
branch
  ↓
implementation
  ↓
tests
  ↓
security review
  ↓
code review
  ↓
Devil's Advocate
  ↓
CI
  ↓
PR
  ↓
merge
```

Direct commits to `main` may be used only for small, low-risk maintenance when the repository policy permits it.

Do not bypass branch protection or required CI checks.

Before creating a PR, inspect:

```bash
git status
git diff
git log --oneline -20
```

PR descriptions should state:

- objective
- implementation
- tests
- security impact
- architecture impact
- documentation changes
- known limitations
- hardware validation status
- release impact

---

## 125. Extended requirement — Audio Test Harness

The project must develop a repeatable audio test harness so the Mix Engine can be validated without requiring physical hardware for every test.

The harness should eventually support deterministic signals such as:

- silence
- sine wave
- impulse
- white/noise test signal where appropriate
- clipping input
- multiple simultaneous channels

Validate at minimum:

- channel isolation
- gain
- pan
- mute
- master
- send levels
- independent mixes
- limiter behavior
- clipping behavior
- sample-rate handling
- buffer behavior
- reset/recovery behavior

Conceptual flow:

```text
Generated Input
      ↓
Audio Engine
      ↓
Mix Engine
      ↓
Expected Output
      ↓
Automated Analysis
      ↓
PASS / FAIL
```

The harness must not replace physical audio validation.

---

## 126. Extended requirement — Audio Performance and Benchmarking

Performance claims must be backed by reproducible measurements.

Maintain benchmark documentation where useful:

```text
docs/benchmarks/
├── AUDIO-LATENCY.md
├── CPU-MEMORY.md
├── XRUNS.md
├── NETWORK-JITTER.md
└── TRANSPORT.md
```

Measure where applicable:

- end-to-end latency
- processing latency
- CPU utilization
- memory utilization
- XRUNs
- jitter
- packet loss
- reconnect time
- synchronization error

Do not publish target or observed numbers as facts until measured.

---

## 127. Extended requirement — Hardware Validation Matrix

Maintain a matrix distinguishing simulation, software validation and hardware validation.

Example:

| Capability | Docker | Linux x86_64 | Raspberry Pi | Windows | macOS | Android | iPadOS |
|---|---|---|---|---|---|---|---|
| Mix Engine | TEST | TEST | TEST | TEST | TEST | EXPERIMENTAL | EXPERIMENTAL |
| PipeWire | SIMULATED/NA | TEST | HARDWARE VALIDATION REQUIRED | N/A | N/A | N/A | N/A |
| USB Audio | SIMULATED | TEST | HARDWARE VALIDATION REQUIRED | TBD | TBD | EXPERIMENTAL | EXPERIMENTAL |
| Audio Transport | SIMULATED | TEST | HARDWARE VALIDATION REQUIRED | TBD | TBD | TBD | TBD |

Adapt the matrix to actual evidence.

Never turn `SIMULATED`, `BUILD`, or `TEST` into `SUPPORTED` without appropriate validation.

---

## 128. Extended requirement — Audio Clock, Drift and Synchronization

The architecture must explicitly address clocking before claiming multi-device synchronized audio.

Investigate:

- audio interface clock
- server clock
- receiver clock
- clock drift
- timestamping
- buffer drift
- resampling
- synchronization strategy
- multiple receiver synchronization
- behavior after network interruptions
- recovery from clock divergence

Create an ADR before committing to a synchronization architecture.

If unresolved:

`UNKNOWN`

If physical validation is needed:

`HARDWARE VALIDATION REQUIRED`

Do not assume independent device clocks remain synchronized indefinitely.

---

## 129. Extended requirement — Pairing and Device Identity

Define a secure device pairing flow before allowing arbitrary clients to control mixes.

Conceptual flow:

```text
New Device
    ↓
Pairing
    ↓
Authentication
    ↓
Device Identity
    ↓
Musician Assignment
    ↓
Authorized Mix
```

The design should address:

- pairing code/token
- device identity
- revocation
- reassignment
- expired pairing
- duplicate devices
- lost devices
- authorization after reconnect

A client must never be able to select another musician's mix merely by changing a client-side identifier.

---

## 130. Extended requirement — API and WebSocket Compatibility

The public control API and protocol must be versioned.

Current namespaces:

```text
/api/v1
/ws/v1
```

Define policy for:

- backward compatibility
- breaking changes
- deprecation
- client/server version mismatch
- message versioning
- unknown message types
- unknown fields
- stale revisions
- reconnect synchronization

Future native clients and receivers must not require uncontrolled protocol forks.

Breaking protocol changes require an ADR and migration strategy.

---

## 131. Extended requirement — State Synchronization

The engineer UI, musician clients and server must have a clear authoritative-state model.

Required principles:

```text
Server = authoritative state
Client = local representation
```

For state-changing commands:

```text
command
  ↓
validate
  ↓
authorize
  ↓
apply
  ↓
increment revision
  ↓
ack/confirm
  ↓
broadcast authoritative state
```

Handle:

- simultaneous edits
- stale clients
- lost WebSocket messages
- reconnect
- duplicate commands
- out-of-order commands
- optimistic UI rollback

Do not silently overwrite newer state with stale client state.

---

## 132. Extended requirement — Release Recovery and Existing Releases

When an existing release is blocked, recover it instead of recreating history.

Example:

```text
Existing tag/release
       ↓
CI failure
       ↓
DO NOT recreate tag
       ↓
DO NOT delete release
       ↓
Diagnose root cause
       ↓
Fix CI/configuration
       ↓
Rerun
       ↓
Validate artifacts
       ↓
Validate checksums
       ↓
Validate manifest
       ↓
Publish/attach assets
       ↓
Update documentation
```

Do not force merges or disable gates merely to obtain a green pipeline.

If a release has no assets, it is not considered complete merely because the GitHub Release object exists.

---

## 133. Extended requirement — GitHub Actions Runner Failure Diagnosis

When a workflow fails immediately, especially when all jobs fail with empty logs, distinguish runner/platform/configuration failure from application failure.

Inspect:

- workflow syntax
- `runs-on`
- runner availability
- self-hosted labels
- runner groups
- repository access to runner groups
- GitHub-hosted runner eligibility
- Actions enabled state
- allowed actions policy
- workflow execution protections
- organization policies
- environment protections
- required approvals
- `GITHUB_TOKEN` permissions
- reusable workflow permissions

Do not modify application code to compensate for a runner that never started.

GitHub documents that `permissions` can be scoped at workflow or job level and that specifying permissions limits unspecified permissions; use least privilege. For release publication, `contents: write` may be required, while normal CI commonly needs only read access. citeturn0search0turn0search7

Repository and organization settings can independently restrict whether Actions are enabled, which actions are allowed, and workflow execution. Verify these settings when jobs fail before execution. citeturn0search1turn0search3

---

## 134. Extended requirement — GitHub Actions Permission Policy

Prefer least privilege.

Normal CI example:

```yaml
permissions:
  contents: read
```

Release job example, only when required:

```yaml
permissions:
  contents: write
```

Do not default to:

```yaml
permissions: write-all
```

Do not request a PAT when `GITHUB_TOKEN` is sufficient.

If a permission is required, document:

- workflow
- job
- permission
- reason
- security impact

GitHub's documentation states that specifying individual permissions causes unspecified permissions to become `none`, which supports a least-privilege approach. citeturn0search0

---

## 135. Extended requirement — Configuration and Database Migration Safety

SQLite schema changes must use versioned migrations.

Every migration must define:

- version
- forward migration
- validation
- compatibility impact
- rollback strategy where feasible

Configuration changes must be versioned where necessary.

Upgrades must not silently destroy:

- users
- device assignments
- scenes
- presets
- permissions
- system configuration

Backup/restore procedures must be tested before being described as production-ready.

---

## 136. Extended requirement — Backup, Restore and Recovery Validation

The production deployment must eventually support safe recovery of important state.

Evaluate backup/restore for:

- SQLite database
- scenes
- presets
- configuration
- certificates/identities where applicable

Validate:

```text
backup
  ↓
clean environment
  ↓
restore
  ↓
validate schema
  ↓
validate state
  ↓
start service
  ↓
functional test
```

A backup mechanism is not considered production-ready until restoration has been tested.

---

## 137. Extended requirement — Security Threat Model

Maintain a lightweight threat model covering:

- unauthorized musician control
- unauthorized engineer control
- stolen/lost client device
- malicious local-network client
- malformed WebSocket messages
- malformed audio packets
- replayed commands
- stale authentication
- compromised server
- malicious package/update
- dependency compromise
- exposed management interface

Document trust boundaries between:

```text
Audio Interface
Server
Control Clients
Audio Receivers
Local Network
Internet/Cloud, if ever introduced
```

Do not assume the local network is inherently trusted.

---

## 138. Extended requirement — Supply Chain and Release Integrity

Evaluate adding:

- SBOM generation
- dependency license report
- dependency vulnerability report
- artifact checksums
- artifact provenance/attestation where practical
- signed releases where practical

Do not make supply-chain features mandatory for the MVP if they block core development, but add them to the backlog with appropriate priority.

Release artifacts must be traceable to:

```text
version
commit
workflow run
build target
source
checksums
```

---

## 139. Extended requirement — Crash Handling and Recovery

The server must eventually define behavior for:

- process crash
- audio backend crash
- PipeWire restart
- device disconnect
- database failure
- corrupted configuration
- network failure
- client failure

Production Raspberry Pi deployment should use appropriate service supervision.

Recovery must favor safe audio behavior and avoid unexpected loud output.

Do not claim crash recovery until it has been tested.

---

## 140. Extended requirement — Network Fault Injection

Before declaring audio transport production-ready, create repeatable tests for:

- packet loss
- packet duplication
- packet reordering
- jitter
- temporary network outage
- reconnection
- congestion
- malformed packets
- delayed packets
- receiver restart
- server restart

Use simulation for initial development and hardware/network validation before production claims.

---

## 141. Extended requirement — Safe Audio Failure Policy

For uncertain routing or recovery conditions, prefer safe behavior.

Evaluate:

- startup mute
- reconnect mute
- master level limits
- limiter
- protection against sudden gain jumps
- safe handling of stale state
- safe scene recall

The system must not claim to guarantee hearing safety. User documentation must emphasize responsible listening levels.

---

## 142. Extended requirement — Telemetry and Privacy Decision

Explicitly decide whether the product sends telemetry.

The default architecture is local-first.

Unless explicitly implemented and documented:

- do not send usage telemetry
- do not send audio content to cloud services
- do not collect unnecessary personal data
- do not require Internet connectivity for live audio

If telemetry is introduced in the future, create an ADR covering:

- data collected
- purpose
- retention
- opt-in/opt-out
- security
- privacy impact
- offline behavior

---

## 143. Extended requirement — Accessibility and Internationalization

For the PWA, evaluate baseline accessibility:

- keyboard navigation
- focus states
- readable labels
- accessible controls
- sufficient contrast
- screen-reader semantics where practical
- touch-friendly controls

Internationalization may remain future work unless required by the product, but text must not be unnecessarily hard-coded into architecture that would prevent future localization.

If not implemented, record as a backlog item rather than claiming support.

---

## 144. Extended requirement — Architecture Fitness Review

At the end of major phases, verify that implementation still respects the core architecture:

```text
Platform-independent Core
          ↓
Audio Abstraction
          ↓
Platform Backend
```

and:

```text
Control Plane ≠ Audio Plane
```

Look specifically for accidental coupling such as:

- Mix Engine importing PipeWire-specific code
- business logic in HTTP handlers
- UI becoming authoritative state
- database operations inside realtime code
- network operations inside realtime callbacks
- platform-specific assumptions leaking into core

If architectural drift is detected, record it in `docs/ARCHITECTURE-GAPS.md` and fix or backlog it.

---

## 145. Extended requirement — Final Extended Operating Contract

Hermes must continuously maintain four layers of truth:

```text
START.md
  = engineering operating contract

Architecture / ADRs
  = architectural decisions

docs/TODO.md
  = executable backlog

Code + Tests + CI
  = implementation evidence
```

If these disagree:

1. inspect evidence
2. identify the discrepancy
3. document it
4. determine the technically justified correction
5. update the appropriate source of truth
6. validate again

Never silently let contradictory documentation, code and tests accumulate.

The objective is not to maximize the number of files or features. The objective is to produce a validated, maintainable, secure and reproducible real-time IEM platform.
