# Engineer Bootstrap and Musician QR Onboarding Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Corrigir primeiro acesso do Engineer e estruturar onboarding de músico sem senha, com QR temporário controlado por Admin/Engineer.

**Architecture:** Manter autenticação server-authoritative. Engineer bootstrap usa token temporário apenas para `PUT /api/v1/auth/password`; troca revoga sessões e exige novo login. Musician usa perfil separado de credencial, convite QR temporário armazenado somente como hash, refresh token HttpOnly revogável e autorização de mix no servidor.

**Tech Stack:** Rust/Axum, SQLite, JWT Ed25519, refresh token HttpOnly, Argon2id para contas com senha, React/TypeScript/Vitest, QR gerado no cliente ou servidor sem transportar token permanente.

---

## Estado implementado neste ciclo

- Engineer Console agora interpreta `must_change_password`.
- Dashboard e WebSocket ficam bloqueados durante primeiro acesso.
- Tela de troca chama `PUT /api/v1/auth/password`.
- Após sucesso, token é descartado e novo login é exigido.
- Cobertura frontend adicionada: bootstrap, troca, divergência de senha.
- `START.md` recebeu regra do músico e limites de segurança do QR.

Verificação local atual:

```bash
cd web/engineer
npm test -- --run
npm run typecheck
npm run build
```

Resultado esperado neste ciclo: 59 testes PASS, typecheck PASS, build PASS.

---

## Task 1 — Consolidar contrato de autenticação

**Files:**
- Modify: `server/api-server/src/routes/auth.rs`
- Modify: `web/engineer/src/App.tsx`
- Modify: `web/musician/src/api/auth.ts`
- Test: `server/api-server/src/routes/auth.rs`

**Acceptance:** Login response e clientes compartilham `access_token`, `role` e `must_change_password`; nenhum cliente assume campos inexistentes.

**Tests:** login normal, login bootstrap, schema inválido, senha expirada.

---

## Task 2 — Fechar first-login Engineer no backend

**Files:**
- Inspect/modify: `server/api-server/src/middleware.rs`
- Modify: `server/api-server/src/routes/auth.rs`
- Test: `server/api-server/src/routes/auth.rs`

**Acceptance:** Conta bootstrap só acessa logout/password; troca inválida não altera hash; troca válida limpa flag e revoga sessões; novo login funciona.

**Tests:**

```bash
cargo test --manifest-path server/Cargo.toml auth
```

---

## Task 3 — Corrigir first-login Engineer no frontend

**Files:**
- `web/engineer/src/App.tsx`
- `web/engineer/src/App.test.tsx`

**Acceptance:** Nenhuma chamada de dashboard ou WebSocket ocorre enquanto `must_change_password=true`; senha divergente não chama API; sucesso exige login novo.

**Tests:**

```bash
cd web/engineer
npm test -- --run
npm run typecheck
npm run build
```

---

## Task 4 — Modelar perfil persistente de músico

**Files:**
- Modify: `server/api-server/src/db.rs`
- Create: migration/versioned schema conforme padrão existente
- Create/modify: `server/api-server/src/routes/profile.rs`
- Tests: DB + API integration

**Model:** `profile_id`, `user_id`, `display_name`, `instrument_id`, `status`, timestamps. User identity, profile e assignment ficam separados.

**Rules:** catálogo de instrumentos controlado; nome Unicode validado por bytes/caracteres; músico edita somente próprio perfil; Engineer/Admin gerenciam conforme RBAC.

**Endpoints:**

- `GET /api/v1/me/profile`
- `PUT /api/v1/me/profile`
- `GET /api/v1/admin/musicians`

---

## Task 5 — Implementar convite QR temporário

**Files:**
- Modify: `server/api-server/src/db.rs`
- Create/modify: `server/api-server/src/routes/qr.rs`
- Modify: router/middleware/rate-limit modules
- Tests: API integration and concurrency

**Security contract:**

- segredo aleatório de alta entropia;
- somente hash persistido;
- TTL obrigatório;
- single-use ou `max_uses` estrito;
- rotação e revogação;
- rate limit por IP/dispositivo;
- nenhum JWT, refresh token ou segredo bruto em URL/log/telemetria;
- HTTPS obrigatório fora do override de desenvolvimento;
- auditoria registra evento e ID, nunca credencial.

**Endpoints:**

- `POST /api/v1/admin/qr-mode`
- `GET /api/v1/admin/qr-mode`
- `POST /api/v1/admin/qr-mode/rotate`
- `POST /api/v1/onboarding/qr/exchange`
- `DELETE /api/v1/admin/sessions/{id}`

---

## Task 6 — Musician PWA sem senha

**Files:**
- Modify: `web/musician/src/components/Login.tsx`
- Modify: `web/musician/src/api/auth.ts`
- Modify: `web/musician/src/App.tsx`
- Tests: component/API tests

**Flow:** ler QR → confirmar servidor/projeto → informar nome/instrumento → exchange → receber access token em memória + refresh cookie HttpOnly → restaurar sessão por refresh após reload.

**Proibido:** senha vazia, senha compartilhada, usuário genérico, token permanente em QR, token no `localStorage`, token em query string.

---

## Task 7 — Sessão e revogação operacional

**Files:**
- Modify: DB/session routes
- Modify: Engineer Console musician management
- Tests: session lifecycle

**Acceptance:** logout local, revogação por Admin/Engineer, sessão expirada, dispositivo perdido e QR desativado encerram WebSocket e removem acesso.

---

## Task 8 — ADR e documentação

**Files:**
- Create: `docs/decisions/ADR-013-musician-profile-qr-onboarding.md`
- Modify: `START.md`
- Modify: `docs/TODO.md`
- Modify: `docs/DEVELOPMENT-HANDOFF.md`
- Modify: `docs/guides/MUSICIANS-GUIDE.md`

**Acceptance:** requisito, decisões, ameaças, estados `PENDING/BLOCKED`, evidência e tarefas ficam sincronizados. Não declarar runtime/LAN/hardware validado.

---

## Gates finais

```bash
bash scripts/validate-docs.sh
bash scripts/validate-skills.sh
cargo fmt --manifest-path server/Cargo.toml --all -- --check
cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path server/Cargo.toml
(cd web/musician && npm test -- --run && npm run typecheck && npm run build)
(cd web/engineer && npm test -- --run && npm run typecheck && npm run build)
```

Security review obrigatório antes de merge: replay QR, brute force, CSRF/Origin, fixation, sessão roubada, escopo RBAC e logs sem segredos.

## Decisões abertas

1. Engineer pode ativar QR globalmente ou somente Admin?
2. QR permite um músico ou lote limitado de músicos?
3. Quem define assignment: Engineer/Admin depois do onboarding ou músico escolhe mix?
4. Refresh de músico sobrevive por quanto tempo?
5. Como revogar sessão específica e todas sessões de um perfil?
6. Catálogo inicial de instrumentos e identificadores canônicos?
