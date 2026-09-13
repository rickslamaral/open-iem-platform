# Phase 82 Review — auditoria de segurança do instalador

**Data:** 2026-09-13
**Status:** BLOCKED — PR #41 não aprovado para merge.

## Escopo

Auditoria independente do instalador Linux e dos workflows no commit `4d20485` / PR #41.

## Findings

- **HIGH — fonte sem verificação criptográfica:** `scripts/install.sh` baixa o repositório e constrói o checkout remoto. Mesmo com `--ref`, o fluxo padrão usa `main` mutável e não valida assinatura/checksum da fonte. Fixar commit/tag e validar autenticidade antes do build.
- **MEDIUM — dependência Node.js:** caminho `apt` instala `nodejs` do repositório do SO, mas o build exige Node.js >= 20. Em distribuições com Node.js 18, host sofre mutação e instalação falha depois. Validar versão antes de mutar host e instalar runtime suportado.
- **MEDIUM — instalação parcial:** binários, assets, chaves e unit systemd são alterados em etapas; falha posterior deixa instalação mista. Usar staging completo, publicação atômica e rollback.
- **MEDIUM — reexecução de UI:** `cp -a web/*/dist "$PREFIX/web/*"` pode aninhar `dist` ou preservar conteúdo velho quando destino existe. Copiar conteúdo para staging limpo e publicar atomicamente; não apagar árvore ativa sem rollback.

## Evidência

- `make validate`: aprovado localmente.
- `bash -n scripts/install.sh`: aprovado.
- CI PR #41 run `34739632116`: concluído com sucesso.
- Reviews independentes `code-review` e `security-review`: findings acima.
- Instalação real, release, PipeWire/ALSA, WebRTC e Raspberry Pi 5: não validados.

## Gate

**FAIL.** Não fazer merge ou release até fechar HIGH/MEDIUM. Atualizar TODO, README, CHANGELOG e log após correção e repetir testes/reviews.
