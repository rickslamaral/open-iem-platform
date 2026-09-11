# Phase 34 — CLI output correctness

## SPEC

Corrigir contrato real do `open-iem-admin`: mensagens HTTP 404 não podem afirmar que rotas já implementadas ainda são planejadas; tabelas devem preservar colunas presentes em qualquer objeto da resposta, não somente no primeiro.

## PLAN

1. Extrair coleta determinística de chaves JSON para helper testável.
2. Corrigir tratamento de 404 para erro genérico sem estado falso.
3. Adicionar testes unitários para colunas union e 404 sem vazamento de corpo.
4. Executar fmt, testes, clippy, docs e scans.
5. Fazer review independente e documentar estado real.

## Limites

Sem alterar API, autenticação, transporte ou contrato de instalação do binário `iem`. CI remoto continua bloqueado por runner não alocado.
