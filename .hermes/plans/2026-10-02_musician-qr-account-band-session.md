# Musician QR Account and Band Session Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Permitir que músico veja QR no Musician UI, use convite temporário para criar usuário/senha próprios e entre numa sessão de banda ou na sessão `Default/Padrão`.

**Architecture:** QR funciona somente como convite de bootstrap, nunca como credencial permanente. O servidor consome o convite uma vez, cria identidade e credencial Argon2id, associa perfil à banda opcional e cria sessão revogável. Banda e roster são decisões server-authoritative; cliente nunca escolhe usuários, canais ou permissões.

**Tech Stack:** Rust/Axum, SQLite migrations, JWT/access sessions existentes, refresh cookie HttpOnly, React/TypeScript, Vitest, Docker Compose.

---

## Regras de produto

1. Musician UI exibe QR temporário em `http://<host>:5173`; host e porta vêm da configuração do deployment.
2. QR contém somente convite de alta entropia/uso limitado. Nunca contém senha, senha hash, JWT ou credencial permanente.
3. Músico lê QR, informa `username`, senha, nome, instrumento e banda opcional.
4. Servidor valida tudo, cria usuário `Musician`, armazena somente hash Argon2id e cria perfil/sessão.
5. Banda selecionada leva à sessão dessa banda.
6. Banda ausente leva à sessão `Default/Padrão`.
7. Roster mostra somente músicos com sessão ativa, não revogada e não expirada, no mesmo escopo de banda.
8. Logout, expiração, revogação e desconexão removem músico do roster.
9. Engineer/Admin controlam bandas, assignment de mix, canais e limites. Musician não ganha autorização por escolher banda.

## Implementação por etapas

### Task 1: Fixar contrato e migração de dados

**Files:**
- Modify: `server/api-server/src/db.rs`
- Test: `server/api-server/src/db.rs` e testes de integração de rotas

Adicionar migration versionada para:
- `bands(id, name, normalized_name, active, created_at)`;
- `musician_profiles.band_id` nullable;
- constraints para banda ativa e vínculo válido;
- índice de roster por `band_id`, status e sessão ativa.

Não alterar M001-M004. Migration deve falhar fechado se schema esperado divergir.

### Task 2: Criar identidade com credencial no exchange

**Files:**
- Modify: `server/api-server/src/routes/qr.rs`
- Modify: `server/api-server/src/auth.rs`
- Modify: `server/api-server/src/db.rs`
- Test: testes QR e auth

Alterar exchange para aceitar contrato versionado:

```json
{
  "qr_secret": "...",
  "username": "...",
  "password": "...",
  "display_name": "...",
  "instrument_id": "...",
  "band_id": null
}
```

Regras:
- segredo QR consumido atomicamente antes da criação;
- username normalizado, único e não enumerável;
- senha validada e hash Argon2id;
- senha nunca aparece em logs, auditoria, URL ou resposta;
- rollback completo se criação de usuário, perfil, banda ou sessão falhar;
- exchange retorna access token somente conforme contrato atual e refresh cookie HttpOnly;
- não permitir que QR sobrescreva usuário existente.

### Task 3: Endpoints de catálogo e roster

**Files:**
- Create/modify: `server/api-server/src/routes/bands.rs`
- Modify: `server/api-server/src/routes/mod.rs`
- Modify: `server/api-server/src/main.rs`
- Test: testes RBAC e integração

Adicionar:
- catálogo de bandas visível no onboarding;
- listagem de roster da sessão autenticada;
- filtro obrigatório por `band_id` ou sessão `Default/Padrão`;
- filtro obrigatório por sessão ativa/revogação/expiração;
- Engineer/Admin podem criar/editar/desativar bandas;
- Musician só lê catálogo permitido e roster do próprio escopo.

Não retornar senha, hash, tokens ou dados de músicos fora do escopo.

### Task 4: Exibir QR no Musician UI

**Files:**
- Modify: `web/musician/src/App.tsx`
- Modify: `web/musician/src/components/Login.tsx`
- Create/modify: componentes QR e onboarding
- Test: `web/musician/src/**/*.test.tsx`

Implementar tela de display do QR para Engineer/Admin ativar o convite e tela de onboarding para o músico:
- mostrar QR com TTL e estado `active/expired/revoked`;
- não persistir segredo em localStorage, URL ou logs;
- câmera com fallback manual;
- formulário username/senha/nome/instrumento/banda;
- mensagens sem revelar se username já existe quando isso permitir enumeração;
- limpar segredo e senha após sucesso, erro terminal ou logout.

A porta padrão é `5173`, mas UI deve usar origin/configuração atual, não URL hard-coded.

### Task 5: Sessão e roster em tempo real

**Files:**
- Modify: `server/api-server/src/middleware.rs`
- Modify: `server/api-server/src/routes/profile.rs` ou nova rota de roster
- Modify: `web/musician/src/App.tsx`
- Test: Rust integration + Musician Vitest

Implementar atualização de presença:
- heartbeat/reconnect controlado pelo servidor;
- sessão ativa expira sem heartbeat dentro do TTL definido;
- logout revoga sessão;
- roster remove sessão inativa;
- WebSocket autoriza banda e mix em cada conexão/mensagem;
- cliente não consegue forjar `band_id`, user ID ou assignment.

### Task 6: Validação e documentação

**Files:**
- Modify: `START.md`
- Modify: `docs/TODO.md`
- Modify: `docs/DEVELOPMENT-HANDOFF.md`

Executar:

```bash
scripts/validate-docs.sh
git diff --check
cargo fmt --all --manifest-path server/Cargo.toml -- --check
cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path server/Cargo.toml
cd web/musician && npm run typecheck && npm test -- --run && npm run build
```

Documentar separadamente:
- implementado em código;
- validado apenas por CODE/CI/SIMULATED;
- pendente de runtime físico WebRTC/DTLS-SRTP, PipeWire/ALSA, LAN e Raspberry Pi 5.

## Riscos e decisões

- QR exposto na tela não é login por si só; ele é convite temporário. O login permanente nasce no formulário após leitura.
- Não usar banda como autorização. Banda define escopo de sessão; Engineer/Admin definem mix e canais.
- Não mostrar roster global. Default contém somente músicos sem banda; banda contém somente músicos daquela banda.
- “Online” deve ser derivado de sessão ativa no servidor, não de flag enviada pelo navegador.
- Criação de senha exige política explícita, Argon2id e nunca recuperação por QR reutilizado.
- Seleção de banda deve ser opcional e fail-closed: banda inexistente/inativa rejeita exchange.
