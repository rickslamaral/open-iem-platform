# Phase 82 Review — auditoria de segurança do instalador

**Data:** 2026-09-13
**Status:** PASS WITH CONDITIONS — Node.js preflight corrigido; pendente instalação real, CI novo e Raspberry Pi 5.

## Escopo

Auditoria independente do instalador Linux e dos workflows no commit `4d20485` / PR #41. Iteração de correção aplicada em `scripts/install.sh`.

## Findings

- **Corrigido — fonte mutável:** instalador exige SHA completo de 40 caracteres, faz fetch limitado, checkout detached e compara `git rev-parse HEAD` antes do build. Autenticidade criptográfica da origem ainda depende de transporte Git HTTPS/SSH; assinatura de commit permanece evolução futura.
- **Corrigido — dependência Node.js:** `preflight_node_check()` chamado antes de `install_deps`; se `node` presente e < 20, falha imediatamente sem instalar pacotes. `check_tools` mantém verificação final após instalação.
- **Corrigido — instalação parcial de artefatos:** build completo entra em staging limpo e release versionado; `current` preserva release anterior se falha antes do commit. Chaves e systemd ainda são operações posteriores e exigem teste de rollback real.
- **Corrigido — reexecução de UI:** conteúdo de cada `dist/` vai para diretório de staging limpo; não aninha `dist` nem preserva arquivos obsoletos.

## Gate atual

Findings de MEDIUM resolvidos. PR #41 pode prosseguir para CI e revisão independente. Instalação real, release v0.3.1 e Raspberry Pi 5 continuam não validados.

## Evidência

- `make validate`: aprovado localmente.
- `bash -n scripts/install.sh`: aprovado.
- `scripts/install.sh --dry-run --skip-deps --ref <sha40>`: aprovado; host não alterado.
- Node.js v22.22.3 no host: preflight passa corretamente.
- CI PR #41 run `34749067566`: concluído com sucesso (antes desta iteração).
- Instalação real, release, PipeWire/ALSA, WebRTC e Raspberry Pi 5: não validados.

## Gate

**FAIL.** Não fazer merge ou release até fechar HIGH/MEDIUM. Atualizar TODO, README, CHANGELOG e log após correção e repetir testes/reviews.
