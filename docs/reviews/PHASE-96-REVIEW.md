# Phase 96 Review — snapshots locais de configuração

## Escopo

Adicionar operações locais `iem config backup` e `iem config restore` ao CLI,
sem API, shell ou credenciais, com leitura limitada antes da desserialização.

## Implementação

- `iem config backup --output PATH` grava snapshot JSON atômico.
- `iem config restore --input PATH` lê e valida snapshot sem alterar estado em execução.
- Caminhos com symlink são rejeitados.
- Restore lê no máximo `1 MiB + 1 byte` e rejeita payload maior antes da desserialização.

## Evidência

```text
cargo test --manifest-path server/Cargo.toml -p admin-cli
8 passed; 0 failed
```

Evidência permanece CODE local. Instalação operacional, runtime, PipeWire/ALSA,
WebRTC/Opus e Raspberry Pi 5 não foram validados.

## Pendências

- Validar fluxo em instalação limpa.
- Não declarar release ou suporte de hardware com base neste CLI.
