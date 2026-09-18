# Interface de linha de comando

## Estado real

A interface oficial de desenvolvimento é o `Makefile`. O binário administrativo existente chama-se `open-iem-admin`. O binário `iem` agora despacha comandos tipados para alvos existentes do Makefile.

`iem` não possui instalação global nem pacote de release. Execute o binário via Cargo na raiz do repositório; compatibilidade de OS e suporte de hardware seguem limitados às validações documentadas.

## Makefile atual

Executar na raiz do repositório:

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
make test-audio
make test-unit
make test-integration
make build
make package
make diagnostics
make docs
make validate
make clean
```

`make run` e `make run-local` iniciam o `api-server` nativo e exigem configuração local. `run-local` não cria um ambiente de simulação isolado; áudio permanece `SIMULATED` no VPS. `make fmt` formata somente o workspace Rust; frontends não têm alvo de formatação neste Makefile.

`make up`, `down` e `logs` usam Docker Compose para recursos de desenvolvimento do projeto. `make up` reconstrói imagens com `--build`; as UIs ficam vinculadas a localhost no Compose atual. Não significam suporte de produção.

`make package` informa que nenhum pacote de release foi produzido localmente. Artefatos de release continuam condicionados ao pipeline CI.

## CLI administrativo

O binário `open-iem-admin` é construído pelo crate `server/admin-cli`:

```bash
cargo run --manifest-path server/Cargo.toml --bin open-iem-admin -- --help
```

Ele gerencia usuários, sessões e health através da API administrativa. Token pode vir de `--token` ou `OPEN_IEM_ADMIN_TOKEN`. HTTP é permitido somente para loopback; servidor remoto exige HTTPS. Nunca registre token em shell history, documentação ou Git.

## CLI `iem`

Construir e executar na raiz:

```bash
cargo run --manifest-path server/Cargo.toml --bin iem -- --help
cargo run --manifest-path server/Cargo.toml --bin iem -- status

# snapshot local; não chama API e não contém credenciais
cargo run --manifest-path server/Cargo.toml --bin iem -- config backup --output config.json
cargo run --manifest-path server/Cargo.toml --bin iem -- config restore --input config.json
```

Comandos suportados: `help`, `status`, `diagnostics`, `docs`, `test`, `build`, `up`, `down` e `config`. Os oito primeiros chamam somente alvos fixos do Makefile. `iem run` permanece fora do contrato. Falhas do `make` preservam código de saída não-zero.

## Paridade implementada para `iem`

`iem` despacha somente alvos fixos do Makefile, exceto `config`, que executa operações locais de arquivo. Não aceita comandos arbitrários, shell ou argumentos adicionais:

| Comando | Workflow equivalente atual |
|---|---|
| `iem help` | `make help` |
| `iem status` | `make status` |
| `iem diagnostics` | `make diagnostics` |
| `iem docs` | `make docs` |
| `iem test` | `make test` |
| `iem build` | `make build` |
| `iem up` | `make up` |
| `iem down` | `make down` |
| `iem config backup --output PATH` | grava snapshot JSON local de configuração vazia/default |
| `iem config restore --input PATH` | lê, desserializa e valida snapshot JSON local |

## Limites do contrato

`iem run` não entra no contrato inicial: execução depende de configuração, chaves JWT, rede e modo de áudio. Instalação global, empacotamento e suporte adicional de plataforma exigem especificação própria.

## Status de validação

- Makefile: interface local implementada.
- `open-iem-admin`: binário implementado; uso depende de API configurada.
- `iem`: implementado; versão `iem 0.3.1`; comandos Makefile fixos e snapshot local `config`; sem instalação global ou pacote de release.
- CI remoto: `BLOCKED`; runs recentes falharam antes dos steps com `runner_id=0`.
- PipeWire, WebRTC media e Raspberry Pi 5: `HARDWARE VALIDATION REQUIRED`.
