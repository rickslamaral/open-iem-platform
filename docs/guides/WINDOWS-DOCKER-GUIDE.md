# Open IEM Platform no Windows via Docker Desktop

Guia de preparação para Windows 10/11. O repositório contém configuração Compose planejada, mas os três Dockerfiles referenciados atualmente não existem; portanto `docker compose up --build` está **BLOCKED** até esses arquivos serem adicionados. Use comandos nativos para validar software enquanto isso.

> **Limite:** Windows não valida servidor de áudio. PipeWire, ALSA, áudio real, WebRTC de mídia, latência, runtime ARM64 e Raspberry Pi continuam `SIMULATED` ou não validados. Use Raspberry Pi 5/Linux para validação de hardware.

## 1. Pré-requisitos

Instale:

- Windows 10/11 x64
- Docker Desktop com backend WSL2 e Linux containers
- Git for Windows
- PowerShell 5.1+ ou PowerShell 7+
- WSL2 com uma distribuição Linux, recomendado para gerar chaves e executar comandos Unix

Verifique Docker:

```powershell
docker version
docker compose version
```

O Docker Desktop precisa estar iniciado antes de subir a stack.

## 2. Obter código

No PowerShell:

```powershell
git clone https://github.com/rickslamaral/open-iem-platform.git
cd open-iem-platform
```

Não use `docker compose` em modo Windows containers. O projeto usa imagens Linux.

## 3. Gerar chaves JWT

As chaves não pertencem ao repositório. Gere-as dentro do diretório do projeto usando WSL:

```bash
mkdir -p keys

openssl genpkey \
  -algorithm ed25519 \
  -out keys/ed25519_private.pem

openssl pkey \
  -in keys/ed25519_private.pem \
  -pubout \
  -out keys/ed25519_public.pem
```

No WSL, `/mnt/c/...` aponta para discos Windows. Exemplo, se projeto estiver em `C:\Users\Ricardo\open-iem-platform`:

```bash
cd /mnt/c/Users/Ricardo/open-iem-platform
```

Confirme que arquivos existem:

```bash
ls -l keys/ed25519_*.pem
```

Nunca faça commit de `keys/`. Nunca envie chaves privadas por chat ou GitHub.

## 4. Subir ambiente local

**Estado atual: BLOCKED.** `docker-compose.yml` referencia `server/Dockerfile.dev`, `web/musician/Dockerfile.dev` e `web/engineer/Dockerfile.dev`, mas esses arquivos ainda não existem no repositório. Não há evidência de que o Compose suba nesta revisão.

**Segurança:** configuração Compose usa `OPENIEM_BIND_ADDR=0.0.0.0:3000`, `OPENIEM_ALLOW_INSECURE_HTTP=true` e publica HTTP sem TLS. Isso serve somente para localhost ou LAN isolada. Não encaminhe porta 3000 à Internet nem use em produção; exposição externa exige Caddy/Nginx com TLS conforme `deployment/caddy/`.

Quando os Dockerfiles forem adicionados e testados, o comando planejado será:

```bash
docker compose up --build
```

Primeiro build pode demorar. A stack planejada inicia três serviços:

- API Rust: `api-server`
- Interface do músico: `musician-ui`
- Console do engenheiro: `engineer-ui`

Mantenha terminal aberto para acompanhar logs. Para executar em segundo plano:

```bash
docker compose up --build -d
```

## 5. Acessar interfaces

**Disponibilidade atual: BLOCKED pelo Compose ausente.** URLs abaixo são os endpoints planejados em `docker-compose.yml`; não foram validados neste workspace.

Abra no navegador Windows:

```text
API:          http://localhost:3000
Musician PWA: http://localhost:5173
Engineer UI:  http://localhost:5174
```

Health check no PowerShell:

```powershell
Invoke-WebRequest http://localhost:3000/api/v1/health
```

Ou no WSL/Git Bash:

```bash
curl --fail http://localhost:3000/api/v1/health
```

Ver status dos serviços:

```bash
docker compose ps
```

Ver logs da API:

```bash
docker compose logs -f api-server
```

## 6. Fluxo de smoke test

**Estado atual: não executável via Docker Compose**, pois os Dockerfiles estão ausentes. Após desbloqueio, o fluxo de controle planejado é:

1. Inicie a stack e confirme `api-server` saudável.
2. Crie usuário e atribua mix usando procedimento documentado do backend/admin CLI; Engineer UI não oferece criação administrativa completa.
3. Faça login como engenheiro.
4. Abra Musician PWA em outra aba, em `http://localhost:5173`.
5. Faça login como músico.
6. Altere gain, pan e mute.
7. Confirme atualização de estado no Engineer UI via WebSocket.
8. Reinicie a API:

   ```bash
   docker compose restart api-server
   ```

9. Confirme reconexão da interface e ressincronização do estado.

Esse fluxo testa plano de controle. Não testa caminho de áudio.

## 7. Parar e limpar

Parar containers mantendo banco SQLite:

```bash
docker compose down
```

Parar e remover volume, incluindo banco local:

```bash
docker compose down -v
```

> `docker compose down -v` é destrutivo para dados do ambiente local. Não execute se quiser preservar usuários, sessões ou estado armazenado.

Remover imagens de desenvolvimento não é necessário para uso normal. `docker image prune` pode remover imagens não usadas de outros projetos Docker; confirme o impacto antes de executar:

```bash
docker image prune
```

## 8. Problemas comuns

### `docker: command not found`

Docker Desktop não está instalado, não está no `PATH` ou ainda não iniciou. Abra Docker Desktop e repita:

```powershell
docker version
```

### `Cannot connect to the Docker daemon`

Docker Desktop está fechado ou WSL2/backend Linux não iniciou. Abra Docker Desktop, aguarde ficar pronto e execute:

```powershell
wsl --status
docker compose ps
```

### Falha ao montar `./keys:/secrets`

Verifique:

- Docker Desktop tem acesso ao diretório do projeto.
- `keys/ed25519_private.pem` existe.
- `keys/ed25519_public.pem` existe.
- Docker Desktop está usando Linux containers.

### Porta já ocupada

Verifique no PowerShell:

```powershell
Get-NetTCPConnection -LocalPort 3000,5173,5174 -ErrorAction SilentlyContinue
```

Pare processo conflitante ou altere portas no `docker-compose.yml`. Se alterar a porta da API, atualize também `VITE_API_BASE_URL` das UIs.

### Interface abre, mas API não responde

Veja estado e logs:

```bash
docker compose ps
docker compose logs --tail=200 api-server
```

A API precisa ficar `healthy` antes das UIs iniciarem corretamente.

### Lentidão extrema no build

Mantenha o projeto dentro do filesystem Linux do WSL, por exemplo `~/open-iem-platform`, em vez de `/mnt/c`. Isso reduz custo de I/O entre Windows e containers.

## 9. O que este ambiente prova

| Capacidade | Windows + Docker Desktop |
|---|---|
| API REST | Planejado; Docker Compose BLOCKED |
| JWT e autorização | Planejado; Docker Compose BLOCKED |
| SQLite | Planejado; Docker Compose BLOCKED |
| WebSocket e ressincronização | Planejado; Docker Compose BLOCKED |
| Musician PWA | Planejado; Docker Compose BLOCKED |
| Engineer UI | Planejado; Docker Compose BLOCKED |
| PipeWire/ALSA | Não validado |
| Áudio real | Não validado |
| WebRTC media real | Não validado |
| ARM64/Raspberry Pi | Não validado |
| Latência/XRUN | Não validado |

Windows serve para smoke test de software. Não serve como evidência de funcionamento do produto de áudio.
