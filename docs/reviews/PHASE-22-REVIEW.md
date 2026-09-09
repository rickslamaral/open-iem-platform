# Phase 22 Review — TLS Deployment Configuration

**Status:** PASS WITH CONDITIONS (findings addressed before merge)
**Data:** 2026-09-09
**Ambiente:** VPS Linux x86_64; áudio SIMULATED. RPi 5 hardware não disponível.

## Objetivo

Fechar gate HIGH da Phase 3: entregar configuração completa de TLS para o Raspberry Pi 5 e corrigir inconsistência crítica de variáveis de ambiente no docker-compose.yml.

## Implementado

### 1. Correção de variáveis de ambiente (docker-compose.yml)
- Prefixo legado `OPEN_IEM_*` substituído por `OPENIEM_*` em alinhamento com o binário `api-server`.
- Variáveis corrigidas: `OPENIEM_BIND_ADDR`, `OPENIEM_ALLOW_INSECURE_HTTP`, `OPENIEM_DB_PATH`, `OPENIEM_JWT_PRIVATE_PEM`, `OPENIEM_JWT_PUBLIC_PEM`.
- URL do healthcheck corrigida de `/health` para `/api/v1/health`.
- Mount de volume renomeado de `./secrets` para `./keys`.
- Aviso explícito no compose: arquivo é dev-only; produção exige Caddy TLS.

### 2. Aviso de prefixo legado no binário (main.rs)
- `api-server` detecta variáveis `OPEN_IEM_*` no startup e emite `tracing::warn!` explícito por variável, prevenindo configurações silenciosamente incorretas em ambientes de produção com variáveis legadas.

### 3. Caddy TLS configuration (deployment/caddy/Caddyfile)
- `auto_https off` para uso de certificado mkcert na LAN.
- Bloco `iem.local`: TLS com certs mkcert, `reverse_proxy 127.0.0.1:8080`, headers `X-Real-IP`, `X-Forwarded-For`, `X-Forwarded-Proto`.
- Redirecionamento HTTP → HTTPS permanente.
- Comentários de segurança: `api-server` não deve receber tráfego externo direto; `OPENIEM_ALLOW_INSECURE_HTTP` deve estar ausente em produção.

### 4. systemd service (deployment/systemd/openiem-server.service)
- Usuário dedicado `openiem`, sem shell interativo.
- Bind em loopback (`127.0.0.1:8080`), `OPENIEM_ALLOW_INSECURE_HTTP` ausente (fail-closed).
- Hardening: `NoNewPrivileges=true`, `PrivateTmp=true`, `ProtectSystem=strict`, `ProtectHome=true`, `ReadWritePaths=/var/lib/openiem`.
- `LimitNOFILE=65536`, `Restart=on-failure`, `RestartSec=5s`.

### 5. Raspberry Pi 5 deployment guide (deployment/raspberry-pi/README.md)
- Guia completo: binary download + checksum, service user, JWT keys, systemd, Caddy+mkcert, CA trust por OS.
- **Adicionado após security review:** instruções críticas de proteção da `rootCA.key` (chmod 600, sem backup não criptografado, procedimento de revogação, validade de 2 anos e 3 meses).
- Limitações marcadas: PipeWire SIMULATED, ARM64 não validado em hardware, mDNS Android, PREEMPT_RT pendente.

### 6. ADR-011 (docs/adr/ADR-011-tls-deployment.md)
- Decisão: terminação TLS no Caddy; `api-server` sempre liga ao loopback em produção.
- Alternativas rejeitadas documentadas: mTLS (complexo), HTTP-LAN (browsers bloqueiam), Nginx (suportado, não documentado).

## Verificação

- Rust fmt: PASS.
- Rust clippy `--all-targets -D warnings`: PASS.
- cargo test --workspace: 199 testes, 0 falhas.
- Static scan (secrets/injection/eval/pickle/SQL): PASS.
- Independent code review: passed=true.

## Security Review

| ID | Severidade | Finding | Ação |
|---|---|---|---|
| HIGH-01 | HIGH | rootCA.key sem documentação de chmod 600 e risco de MITM sistêmico | ✅ Corrigido: instrução explícita de chmod 700/600, aviso de backups, procedimento de revogação adicionados ao README |
| MED-01 | MEDIUM | OPENIEM_ALLOW_INSECURE_HTTP=true no compose pode ser usado acidentalmente em produção | ✅ Corrigido: aviso WARNING explícito no compose |
| MED-02 | MEDIUM | Variáveis legadas OPEN_IEM_* silenciosamente ignoradas | ✅ Corrigido: startup warning no binário Rust |
| MED-03 | MEDIUM | Certificados mkcert sem renovação documentada | ✅ Corrigido: validade documentada, instrução de verificação de expiração |
| LOW-01 | LOW | JWT no Sec-WebSocket-Protocol visível em logs Caddy debug | Documentado; access token tem TTL 15min; formato de log Caddy configurável pelo operador |
| LOW-02 | LOW | CA de usuário Android não trusted por apps por padrão | Info — Open IEM não tem app nativo Android; usuário acessa via browser; documentado como limitação |

## Condições de PASS WITH CONDITIONS
- [x] HIGH-01 corrigido no README.
- [x] MED-01 corrigido no compose.
- [x] MED-02 corrigido no binário.
- [x] MED-03 corrigido no README.
- [ ] LOW-01 (Caddy log sanitization): requer configuração do operador; documentado como limitação.
- [ ] Validação de hardware RPi 5: bloqueio de hardware.

## TODO fechados
- [x] HIGH: Add HTTPS/TLS listener and fail-closed transport configuration (Phase 3 security follow-up)

## Limitações
- PipeWire/ALSA: SIMULATED no VPS.
- ARM64 cross-compilado, não validado em hardware real.
- Caddy log sanitization do `Sec-WebSocket-Protocol`: documentado como LOW, requer configuração do operador.
