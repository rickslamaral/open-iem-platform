# ADR-016 — Modo kiosk Linux com QR público local

- **Status:** Proposed
- **Data:** 2026-10-06
- **Decisor:** Arquitetura Open IEM

## Contexto

Instalações locais em Raspberry Pi, Ubuntu e Debian podem operar como kiosk. Usuários na LAN precisam abrir uma URL pública local, visualizar QR e seguir para onboarding/login. O recurso não pode embutir credenciais no instalador nem expor `access_token`, `refresh_token` ou QR secret em query string, logs, argumentos de processo ou armazenamento persistente do navegador.

## Decisão

Adicionar kiosk como perfil opcional do instalador Linux. A instalação pergunta se deve ativar kiosk; automação usa `--kiosk` ou `--no-kiosk` e nunca bloqueia aguardando input. Quando ativo, um serviço systemd opcional inicia navegador Chromium/Chrome em usuário não privilegiado após rede e sessão gráfica. Sem display ou navegador, instalação mantém API funcional e imprime URL LAN para abertura em outro dispositivo.

A URL pública usa hostname mDNS quando disponível e endereços LAN descobertos em runtime. Nenhum IP fica gravado no Git. HTTP local continua padrão; HTTPS é opt-in com `OPENIEM_REQUIRE_SECURE_COOKIES=true` somente quando TLS estiver realmente ativo.

QR continua bearer capability e usa contrato existente de geração, rotação, revogação, rate limit e consumo. QR e sessão criada por QR usam exatamente o mesmo TTL, configurável por `OPENIEM_QR_SESSION_TTL_SECONDS`; padrão 4 horas, mínimo 60 segundos, máximo 86400 segundos. Não existe mais TTL independente de 10 minutos.

O frontend lê convite do fragmento, remove o fragmento imediatamente com `history.replaceState` e mantém segredo somente em memória. Exchange usa fluxo existente; refresh token fica em cookie `HttpOnly`, access token fica em memória. Sessão autenticada segue para login/fluxo existente sem persistir tokens.

## Segurança

- Kiosk roda sem privilégios, com perfil de navegador dedicado, sem sync, downloads, extensões ou navegação externa.
- API não recebe segredo de admin via instalador. Ativação/rotação precisa ocorrer pelo fluxo autenticado existente ou contrato local explícito revisado.
- Erros de QR expirado, revogado ou usado não revelam causa detalhada.
- Headers de cache/referrer devem impedir retenção acidental do convite.
- HTTP LAN não é seguro contra observadores; documentação exige HTTPS para redes não confiáveis.
- Desativar kiosk remove autostart e preserva banco, chaves, sessões e instalação da API.

## Consequências

- Kiosk gráfico exige desktop e Chromium/Chrome já disponíveis ou dependências opcionais detectadas; não instala desktop silenciosamente em host headless.
- Raspberry Pi OS, Ubuntu e Debian precisam de testes separados para X11/Wayland, display manager e nomes de pacote.
- QR público por até 4 horas aumenta risco de fotografia/replay; rotação, revogação, limite de uso e rate limit continuam obrigatórios.
- Hardware, áudio físico e WebRTC real permanecem não validados por testes de kiosk.

## Critérios de aceitação

1. Prompt interativo pergunta ativação; flags não interativas funcionam.
2. Reexecução é idempotente e reboot reabre kiosk quando habilitado.
3. `openiem-server.service` inicia antes do navegador.
4. URL LAN funciona sem IP fixo no repositório.
5. QR e sessão expiram no mesmo instante configurado.
6. QR expirado, revogado e reutilizado não criam sessão.
7. Nenhum token/segredo aparece em URL persistente, logs, storage ou argumentos.
8. Sem desktop, instalação não falha: API ativa e fallback documentado.
9. `systemd-analyze verify` passa nas units.
10. Desativação remove somente kiosk.
