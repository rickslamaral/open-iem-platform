# Phase 82 Review — auditoria de segurança do instalador

**Data:** 2026-09-13
**Status:** BLOCKED — findings principais corrigidos localmente; PR #41 aguarda novo CI e revisão independente.

## Escopo

Auditoria independente do instalador Linux e dos workflows no commit `4d20485` / PR #41.

## Findings

- **Corrigido — fonte mutável:** instalador exige SHA completo de 40 caracteres, faz fetch limitado, checkout detached e compara `git rev-parse HEAD` antes do build. Autenticidade criptográfica da origem ainda depende de transporte Git HTTPS/SSH; assinatura de commit permanece evolução futura.
- **Pendente MEDIUM — dependência Node.js:** caminho `apt` instala `nodejs` do repositório do SO, mas o build exige Node.js >= 20. O check ocorre depois da instalação de pacotes; mover validação/preflight antes de mutar host.
- **Corrigido — instalação parcial de artefatos:** build completo entra em staging limpo e release versionado; `current` preserva release anterior se falha antes do commit. Chaves e systemd ainda são operações posteriores e exigem teste de rollback real.
- **Corrigido — reexecução de UI:** conteúdo de cada `dist/` vai para diretório de staging limpo; não aninha `dist` nem preserva arquivos obsoletos.

## Gate atual

Falha mantida por Node.js preflight e ausência de teste de instalação real. Não fazer merge ou release até novo CI e revisão independente.

## Evidência

- `make validate`: aprovado localmente.
- `bash -n scripts/install.sh`: aprovado.
- CI PR #41 run `34739632116`: concluído com sucesso.
- Reviews independentes `code-review` e `security-review`: findings acima.
- Instalação real, release, PipeWire/ALSA, WebRTC e Raspberry Pi 5: não validados.

## Gate

**FAIL.** Não fazer merge ou release até fechar HIGH/MEDIUM. Atualizar TODO, README, CHANGELOG e log após correção e repetir testes/reviews.
