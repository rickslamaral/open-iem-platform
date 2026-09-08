# Skills Registry

All project-specific agent skills for Open IEM Platform.

## Skill Orchestration

For a new major feature:

```
product-spec
    ↓
superpowers
    ↓
architect-designer
    ↓
senior-backend / senior-frontend / realtime-audio-engineer
    ↓
test-master
    ↓
security-review
    ↓
code-review
    ↓
code-documenter
    ↓
pr-review
```

Not every task requires every skill. Use the minimum appropriate set.

## Skills

### architect-designer

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/architect-designer/` |
| **Purpose** | System architecture, ADR creation, trade-off analysis |
| **Trigger** | Any major architectural decision, new component, protocol choice |
| **Inputs** | Problem statement, existing ADRs, existing docs |
| **Outputs** | ADR in `docs/adr/`, architecture description, gap updates |
| **Dependencies** | product-spec (for requirements context) |

---

### product-spec

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/product-spec/` |
| **Purpose** | PRD, user stories, acceptance criteria, MVP scope, roadmap |
| **Trigger** | New feature request, ambiguous requirements, MVP scope question |
| **Inputs** | User/engineer request, existing product docs |
| **Outputs** | Feature spec in `docs/product/`, updated MVP/roadmap |
| **Dependencies** | None (first in chain) |

---

### superpowers

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/superpowers/` |
| **Purpose** | Enforces structured engineering discipline before implementation |
| **Trigger** | Any significant feature — before coding begins |
| **Inputs** | Problem statement, existing specs |
| **Outputs** | Completed workflow doc, ADR, implementation plan |
| **Dependencies** | product-spec, architect-designer |

---

### senior-backend

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/senior-backend/` |
| **Purpose** | Rust backend: REST, WebSocket, SQLite, domain logic, state |
| **Trigger** | Server-side implementation or modification |
| **Inputs** | Feature spec, API design, data model |
| **Outputs** | Rust implementation, migrations, tests |
| **Dependencies** | architect-designer, product-spec |

---

### senior-frontend

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/senior-frontend/` |
| **Purpose** | React/TS PWA: musician and engineer UIs |
| **Trigger** | Frontend implementation or modification |
| **Inputs** | UX spec, API contract, WebSocket protocol |
| **Outputs** | React components, PWA config, tests |
| **Dependencies** | senior-backend (for API contract) |

---

### realtime-audio-engineer

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/realtime-audio-engineer/` |
| **Purpose** | PipeWire, ALSA, audio graph, DSP, transport, realtime safety |
| **Trigger** | Any audio path work, transport decision, XRUN investigation |
| **Inputs** | Audio architecture spec, transport evaluation |
| **Outputs** | Audio engine implementation, transport evaluation, latency data |
| **Dependencies** | architect-designer |
| **Special** | **Veto authority over unsafe realtime architecture** |

---

### test-master

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/test-master/` |
| **Purpose** | All test types: unit, integration, WebSocket, audio, network |
| **Trigger** | Before or after any feature implementation |
| **Inputs** | Acceptance criteria, feature spec, implementation |
| **Outputs** | Test files, coverage report, test status labels |
| **Dependencies** | product-spec (for acceptance criteria) |

---

### code-documenter

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/code-documenter/` |
| **Purpose** | README, API docs, architecture docs, ADRs, changelog, phase reviews |
| **Trigger** | After significant implementation, API change, phase completion |
| **Inputs** | Implementation, specs, previous docs |
| **Outputs** | Updated docs in `docs/`, code comments, CHANGELOG |
| **Dependencies** | All implementation skills |

---

### security-review

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/security-review/` |
| **Purpose** | Auth, authorization, input validation, dependency audit |
| **Trigger** | Before production milestones, auth changes, new API endpoints |
| **Inputs** | Implementation, security model, deps list |
| **Outputs** | `docs/reviews/SECURITY_REVIEW.md` |
| **Dependencies** | senior-backend (for implementation context) |

---

### code-review

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/code-review/` |
| **Purpose** | Correctness, realtime safety, architecture compliance, tests |
| **Trigger** | Before any significant PR merge, at phase milestones |
| **Inputs** | Implementation diff, spec, tests |
| **Outputs** | `docs/reviews/CODE_REVIEW.md` with BLOCKER/HIGH/MEDIUM/LOW findings |
| **Dependencies** | realtime-audio-engineer (for audio path review) |

---

### pr-review

| Field | Value |
|-------|-------|
| **Path** | `.agents/skills/pr-review/` |
| **Purpose** | Full changeset review: requirements, diff, tests, migration safety |
| **Trigger** | Before merging any PR or declaring a phase complete |
| **Inputs** | Full diff, linked spec, test results |
| **Outputs** | `docs/reviews/PR_REVIEW.md` with READY/READY WITH CHANGES/NOT READY |
| **Dependencies** | code-review, security-review, test-master |
