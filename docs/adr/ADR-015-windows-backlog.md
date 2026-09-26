# ADR-015 — Windows WASAPI/ASIO fora do MVP Linux-first

- **Status:** Accepted
- **Data:** 2026-09-26

## Contexto

T16 exige decidir se backend Windows permanece no MVP. Produto atual é Linux-first, focado em Debian, Ubuntu e Raspberry Pi OS amd64/arm64. Backend nativo Windows WASAPI/ASIO não está implementado nem validado. Docker Desktop executa imagens Linux de desenvolvimento e não prova áudio nativo Windows.

## Decisão

Windows WASAPI/ASIO fica **fora do MVP**. T16 é `NOT_APPLICABLE` para release Linux. Não implementar backend nativo Windows nesta etapa. WASAPI/ASIO permanece backlog futuro.

Windows + Docker Desktop pode ser smoke/control-plane de desenvolvimento, nunca suporte nativo de áudio ou plataforma validada. Reabrir escopo exige decisão de produto com backend, CI Windows, testes nativos e critérios de aceitação.

## Racional

Manter Windows no MVP exigiria escopo técnico, CI, hardware e evidências inexistentes. Adiar evita claims sem evidência e separa release Linux de certificação de plataforma.

## Consequências

- MVP menor e claims alinhados à evidência.
- Release Linux não bloqueada por backend Windows inexistente.
- Usuários Windows não têm suporte nativo WASAPI/ASIO no MVP.

## Alternativas consideradas

- Manter WASAPI/ASIO no MVP: rejeitada; faltam implementação, CI e validação nativa.
- Usar Docker Desktop como suporte de áudio: rejeitada; containers Linux são development-only.
- Implementar somente WASAPI: rejeitada; ainda amplia escopo sem critérios.
- Adiar WASAPI/ASIO: escolhida; coincide com direcionamento Linux-first.
