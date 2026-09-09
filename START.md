# OPEN IEM PLATFORM

# START.md --- Master Engineering Bootstrap & Development Specification

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
Pi, x86_64 mini PCs, future dedicated receivers, future ESP32 research,
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
-   ESP32 receiver research;
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
prototype dedicated/native receivers and ESP32-class hardware.

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

Execute:

``` text
PHASE 0 — PROJECT BOOTSTRAP + SPECIFICATION AUDIT
```

Do not ask for permission to perform normal engineering tasks. Do not
implement the complete product in one pass. Make the repository the
source of truth. Update Markdown documentation every round. Update
CHANGELOG on every meaningful PR. Create and maintain the Musicians
Guide source and final PDF. Build and version release artifacts as the
product becomes releasable. Only advance when the current phase passes
its review gate.
