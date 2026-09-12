# Phase 58 Review — verificação local de assinatura Ed25519

**Data:** 2026-09-12
**Status:** PASS WITH CONDITIONS — workflow integrado localmente; secret externo e CI remoto pendentes; não mergeada.

## Objetivo

Adicionar verificação criptográfica detached para artefatos antes de checksum, validação estrutural e extração. SHA-256 publicado no mesmo canal não autentica origem.

## Implementado

- `scripts/verify-release-signature.py` verifica assinatura Ed25519 com OpenSSL e falha fechado.
- Artefato, assinatura e chave pública precisam ser arquivos regulares; symlink é rejeitado.
- Testes offline cobrem assinatura válida, artefato alterado, assinatura ausente e chave pública symlink.
- Guia Raspberry Pi baixa `.sig`, exige chave pública instalada em `/etc/openiem/release-signing-key.pem` por canal independente e verifica antes da extração.

## Verificação

- `python3 -m pytest -q tests/test_verify_release_signature.py tests/test_validate_release_archive.py`: PASS — 12 testes.
- `python3 -m py_compile` dos dois scripts: PASS.
- `bash -n` dos 8 blocos Bash do guia Raspberry Pi: PASS.
- `make validate`: PASS — Rust, frontends, documentação, PDF e skills.
- `git diff --check`: PASS.
- Testes adicionais sugeridos pelo revisor (assinatura inválida): PASS — incluído na suíte de 12 testes.
- A chave privada não está no repositório.
- CI remoto continua falhando antes dos steps (`runnerId: null`).

## Limitações e decisão

- O workflow agora produz e publica `.sig` para archives x86_64 e ARM64, mas exige o secret externo `OPENIEM_RELEASE_SIGNING_KEY_PEM`.
- O secret é validado como presente e Ed25519; arquivo temporário recebe `trap` de limpeza mesmo após falha.
- O instalador passa a exigir assinatura, portanto não deve ser usado contra releases históricos sem `.sig`. Fingerprint/distribuição autenticada da chave pública, CI remoto e hardware Raspberry Pi continuam pendentes.

**Decisão:** PASS WITH CONDITIONS. Integração local aprovada; provisionar secret/fingerprint e executar CI real antes de merge ou release.
