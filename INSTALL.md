# Open IEM Platform — instalação multiplataforma

Open IEM roda localmente em Raspberry Pi, PC, laptop, MacBook, Linux físico/VM e Windows com Docker Desktop. O plano de áudio físico exige Linux/Raspberry Pi; Windows e macOS fornecem ambiente local de controle via Docker, não certificação de PipeWire/ALSA.

## Matriz de suporte

| Ambiente | Caminho | Áudio físico | Estado |
|---|---|---:|---|
| Raspberry Pi 5, Raspberry Pi OS 64-bit | `scripts/install.sh` ou Docker | Sim, após validar hardware | ARM64 suportado; 32-bit não suportado |
| Ubuntu/Debian x86_64 ou ARM64 | `scripts/install.sh` ou Docker | ALSA/PipeWire, após validar host | Suportado |
| Linux em PC/laptop/VM | Docker ou `scripts/install.sh` | VM/Docker não prova áudio físico | Suportado para controle |
| macOS Intel/Apple Silicon | Docker Desktop | Não | Suportado para controle |
| Windows 10/11 | Docker Desktop + WSL2 | Não | Suportado para controle |

Docker, QEMU, cross-build ARM64 e áudio virtual são evidência de software. Não são validação de Raspberry Pi, USB, PipeWire, ALSA, latência ou WebRTC físico.

## Regras padrão locais

- HTTP local nasce habilitado. HTTPS é opt-in.
- Sessão sobrevive a reload por refresh cookie `HttpOnly` e `SameSite=Strict`.
- `Secure` só deve ser ativado quando acesso usa HTTPS:

```text
OPENIEM_REQUIRE_SECURE_COOKIES=true
```

- QR e sessão criada via QR expiram juntos. Padrão: 4 horas. Configurável entre 60–86400 segundos:

```text
OPENIEM_QR_SESSION_TTL_SECONDS=14400
```

- Não use `OPENIEM_ALLOW_INSECURE_HTTP_ACK`; essa confirmação artificial foi removida.
- Não publique HTTP na Internet. Para acesso externo, use reverse proxy TLS.
- Chaves, senha SoundTech, tokens e QR secrets ficam fora do Git e nunca aparecem em logs/chat.

## Raspberry Pi / Ubuntu / Debian / Linux

### Pré-requisitos

- Sistema 64-bit: Raspberry Pi OS Bookworm 64-bit, Ubuntu ou Debian.
- Rede local estável.
- Pelo menos 8 GB livres para build Docker; 4 GB RAM mínimo, 8 GB recomendado no Raspberry Pi.
- Git, `curl`, `sudo`, internet HTTPS.

No Raspberry Pi, confirme espaço antes do build. `/tmp` cheio causa `No space left on device` mesmo quando o disco ainda tem espaço:

```bash
df -h / /tmp
df -ih / /tmp
sudo journalctl --vacuum-time=7d
sudo apt-get clean
```

Não apague `/tmp` de outro processo em execução. Se `/tmp` for tmpfs pequeno, use temporário em disco:

```bash
sudo mkdir -p /var/tmp/open-iem
sudo chmod 1777 /var/tmp/open-iem
export TMPDIR=/var/tmp/open-iem
```

### Instalação nativa

O instalador exige commit imutável. Nunca use branch ou tag mutável:

```bash
# Substitua SHA por commit completo confiável antes de executar qualquer código privilegiado.
export OPENIEM_REF=<COMMIT-SHA-40-CHARS>
[[ "$OPENIEM_REF" =~ ^[0-9a-fA-F]{40}$ ]] || { printf "OPENIEM_REF must be 40-character commit SHA\n" >&2; exit 1; }
curl -fsSL "https://raw.githubusercontent.com/rickslamaral/open-iem-platform/$OPENIEM_REF/scripts/install.sh" \
  -o /tmp/openiem-install.sh
chmod 0755 /tmp/openiem-install.sh

sudo OPENIEM_REF="$OPENIEM_REF" /tmp/openiem-install.sh --run-tests
```

Instalação customizada:

```bash
sudo OPENIEM_REF=<COMMIT-SHA-40-CHARS> \
  OPENIEM_CONFIG_DIR=/etc/openiem \
  OPENIEM_STATE_DIR=/var/lib/openiem \
  /tmp/openiem-install.sh
```

O instalador:

1. Detecta `apt`, `dnf`, `yum`, `pacman`, `zypper` ou `apk`.
2. Não faz downgrade silencioso de Node.js existente.
3. Usa Rust existente somente se atender versão mínima; caso contrário usa `rustup`.
4. Gera chaves Ed25519 somente em diretório protegido.
5. Preserva senha existente; gera senha nova apenas quando arquivo seguro não existe.
6. Instala serviço systemd quando disponível.
7. Faz build atômico; falha não troca release ativa.
8. Valida Rust, frontends, documentação e áudio headless com `--run-tests`.

Verificação sem instalar:

```bash
sudo OPENIEM_REF=<COMMIT-SHA-40-CHARS> \
  /tmp/openiem-install.sh --validate-only \
  --validation-report /var/tmp/openiem-validation.txt
```

Acesse após configurar firewall/LAN:

```text
Musician: http://<IP-DO-SERVIDOR>:5173
Engineer: http://<IP-DO-SERVIDOR>:5174
API:      http://<IP-DO-SERVIDOR>:3000
```

### Docker em Linux/Raspberry

Use Docker quando não quiser instalar toolchain Rust/Node no host:

```bash
cp .env.example .env.local 2>/dev/null || true
mkdir -p keys
openssl genpkey -algorithm ed25519 -out keys/ed25519_private.pem
openssl pkey -in keys/ed25519_private.pem -pubout -out keys/ed25519_public.pem
chmod 600 keys/ed25519_private.pem

docker compose config --quiet
docker compose up --build -d
docker compose ps
```

Para acesso pela LAN, altere bindings de `127.0.0.1` para `0.0.0.0` apenas em rede confiável e ajuste `VITE_API_BASE_URL`/origens para o IP informado em runtime. Não grave IP fixo no repositório.

## Windows 10/11

1. Instale Docker Desktop com backend WSL2 e Linux containers.
2. Instale Git for Windows.
3. Clone o repositório.
4. Gere chaves usando PowerShell com OpenSSL do WSL ou Git Bash.
5. Execute Docker Compose.

```powershell
git clone https://github.com/rickslamaral/open-iem-platform.git
Set-Location open-iem-platform
wsl --status
docker version
docker compose version
New-Item -ItemType Directory -Force keys | Out-Null
```

Abra WSL na pasta do projeto. A partir daí, gere chaves e suba a stack:

```bash
openssl genpkey -algorithm ed25519 -out keys/ed25519_private.pem
openssl pkey -in keys/ed25519_private.pem -pubout -out keys/ed25519_public.pem
chmod 600 keys/ed25519_private.pem
docker compose config --quiet
docker compose up --build -d
```

Abra `http://localhost:5173`, `http://localhost:5174` e `http://localhost:3000`. Windows não valida áudio físico, PipeWire, ALSA ou Raspberry Pi.

## macOS

1. Instale Docker Desktop para Apple Silicon ou Intel.
2. Instale Git e OpenSSL (`brew install git openssl` se necessário).
3. Clone o repositório.
4. Gere chaves fora do Git.
5. Suba Docker Compose.

```bash
git clone https://github.com/rickslamaral/open-iem-platform.git
cd open-iem-platform
mkdir -p keys
openssl genpkey -algorithm ed25519 -out keys/ed25519_private.pem
openssl pkey -in keys/ed25519_private.pem -pubout -out keys/ed25519_public.pem
chmod 600 keys/ed25519_private.pem
docker version
docker compose config --quiet
docker compose up --build -d
```

Use `http://localhost:5173`, `http://localhost:5174` e `http://localhost:3000`. macOS não valida áudio Linux físico; Docker Desktop fornece somente ambiente de controle.

## Modo kiosk Linux (broker QR local seguro)

Instalador Linux terá prompt:

```text
Ativar modo kiosk para exibir QR público na tela local? [y/N]
```

Automação usará `--kiosk` ou `--no-kiosk`. Quando ativo, serviço systemd inicia Chromium/Chrome em usuário sem privilégio após rede, API e sessão gráfica. Usuários locais na LAN poderão abrir URL descoberta em runtime. Sem desktop ou navegador, instalação não falha: API continua ativa e imprime URL para outro dispositivo.

Regras:

- QR e sessão expiram juntos conforme `OPENIEM_QR_SESSION_TTL_SECONDS`.
- Fragmento do convite é removido da barra com `history.replaceState`.
- Tokens não entram em query string, logs, argumentos, `localStorage` ou `sessionStorage`.
- Kiosk não recebe senha administrativa embutida.
- Desativar kiosk remove autostart, não remove banco, chaves ou sessões.
- Detalhes e critérios: `docs/adr/ADR-016-kiosk-public-qr.md`.

## HTTPS opcional

Para LAN local simples, HTTP é padrão. Para TLS, configure reverse proxy e defina:

```text
OPENIEM_REQUIRE_SECURE_COOKIES=true
```

Sem HTTPS, não exponha portas fora da LAN confiável. Navegadores também podem bloquear câmera/`BarcodeDetector` em HTTP por não ser contexto seguro; use URL manual ou HTTPS.

## Diagnóstico de erros já corrigidos

- `404: request failed`: UI e API vinham de revisões Docker diferentes. Rebuildar API e UIs juntas elimina contrato misto.
- Sessão encerrava após reload: cookie `Secure` em HTTP descartado pelo navegador. HTTP local agora não usa `Secure` por padrão.
- `error[E0658] atomic_try_update` / `fetch_update deprecated`: contadores usam API estável com CAS.
- `403 forbidden: origin is not allowed`: informar origem real em runtime; não fixar IP no Git.
- `No space left on device`: liberar `/tmp` ou definir `TMPDIR` em filesystem persistente.
- `Camera QR scanning is not supported by this browser`: câmera exige contexto seguro; usar HTTPS ou convite manual.
- `npm Security Audit`: locks mantidos nas versões corrigidas antes do push.

## Parar sem perder dados

```bash
docker compose down
```

Não use `docker compose down -v` se quiser preservar banco, usuários e sessões.
