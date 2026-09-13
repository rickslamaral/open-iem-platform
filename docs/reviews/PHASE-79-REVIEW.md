# Phase 79 Review — compatibilidade OpenSSL 3.5 na geração de assinaturas

**Data:** 2026-09-13
**Status:** VERIFIED — correção local e CI remoto aprovados.

## Objetivo

Corrigir falha do job `Python Release Archive Security Tests` no run `34728518286`. OpenSSL 3.5 rejeitou `openssl pkeyutl -sign` para Ed25519 sem `-rawin`; 10 dos 56 testes falharam no helper de assinatura.

## Implementado

- Helpers de teste em `tests/test_verify_release_signature.py` e `tests/test_validate_release_bundle.py` usam `-rawin`.
- Código de produção permanece inalterado: verificação continua usando `/usr/bin/openssl`, descritores protegidos por `O_NOFOLLOW` e limites de entrada.

## Verificação real

- `python3 -m pytest -q tests/test_validate_release_archive.py tests/test_verify_release_signature.py tests/test_validate_release_bundle.py`: **56 passed**.
- `python3 -m py_compile tests/test_verify_release_signature.py tests/test_validate_release_bundle.py`: aprovado.
- `git diff --check`: aprovado.

## Segurança

Nenhum secret adicionado. `-rawin` só corrige semântica de entrada para assinatura Ed25519; não relaxa validação, algoritmo permitido, limites ou proteção contra symlink.

## Revisão independente

A falha foi isolada pelo log remoto: todos os 10 failures ocorrem no comando de geração de assinatura dos testes, com os validadores não sendo exercitados nesses casos. Correção mínima aplicada aos dois helpers duplicados.

## Limitações e gate

- Novo CI ainda pendente.
- PR #40 continua aberto e não mergeado.
- Release, runtime ARM64, Raspberry Pi 5, PipeWire/ALSA e mídia WebRTC continuam não validados.

## Decisão

Aprovar correção local. Fazer push e aguardar CI completo. Não fazer merge nem release antes de todos os jobs verdes.
