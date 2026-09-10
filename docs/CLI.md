# Interface de linha de comando

## Estado real

A interface oficial de desenvolvimento é o `Makefile`. O binário administrativo existente chama-se `open-iem-admin`.

O binário geral `iem` **ainda não existe**. Não há instalação oficial, pacote, compatibilidade de OS nem suporte de hardware a declarar para esse nome.

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
make test-unit
make test-integration
make build
make package
make diagnostics
make docs
make validate
make clean
```

`make run` e `make run-local` iniciam o `api-server` nativo e exigem configuração local. `run-local` não cria um ambiente de simulação isolado; áudio permanece `SIMULATED` no VPS.

`make up`, `down` e `logs` usam Docker Compose para recursos de desenvolvimento do projeto. Não significam suporte de produção.

`make package` informa que nenhum pacote de release foi produzido localmente. Artefatos de release continuam condicionados ao pipeline CI.

## CLI administrativo

O binário `open-iem-admin` é construído pelo crate `server/admin-cli`:

```bash
cargo run --manifest-path server/Cargo.toml --bin open-iem-admin -- --help
```

Ele gerencia usuários, sessões e health através da API administrativa. Token pode vir de `--token` ou `OPEN_IEM_ADMIN_TOKEN`. Nunca registre token em shell history, documentação ou Git.

## Paridade planejada para `iem`

Implementação futura deve definir, testar e documentar contrato antes de criar alias ou pacote:

| Comando futuro | Workflow equivalente atual |
|---|---|
| `iem help` | `make help` |
| `iem status` | `make status` |
| `iem diagnostics` | `make diagnostics` |
| `iem docs` | `make docs` |
| `iem test` | `make test` |
| `iem build` | `make build` |
| `iem up` | `make up` |
| `iem down` | `make down` |

`iem run` não entra no contrato inicial: execução depende de configuração, chaves JWT, rede e modo de áudio. A implementação também precisa definir instalação, códigos de saída, logs, Windows/Linux/Raspberry Pi e rollback de pacote.

## Status de validação

- Makefile: interface local implementada.
- `open-iem-admin`: binário implementado; uso depende de API configurada.
- `iem`: `NOT STARTED`.
- CI remoto: `BLOCKED`; runs recentes falharam antes dos steps com `runner_id=0`.
- PipeWire, WebRTC media e Raspberry Pi 5: `HARDWARE VALIDATION REQUIRED`.
