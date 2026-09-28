# Active Tickets - cccp
> Synced at: 2026-09-28 01:10:11 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#49] refactor(cli): desduplicar aliases no micro help e unificar comando de tickets
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/49

### Description
## Description

Ao executar `cccp` sem parâmetros, a listagem simplificada de comandos (micro help) mostra aliases como `ticket|tickets`. Além disso, o help geral menciona `sync-tickets`, gerando dúvida sobre a existência de 3 comandos para a mesma finalidade.

O micro help deve ser limpo para exibir unicamente o comando principal/canônico de cada funcionalidade, reservando a listagem de aliases apenas para a documentação detalhada de ajuda.

## Steps to Reproduce

1. Executar `cccp` sem passar nenhum argumento.
2. Observar a listagem rápida mostrando `ticket|tickets`.
3. Executar `cccp --help` e observar a presença do alias `sync-tickets`.

## Expected vs Actual

- **Esperado:** O micro help deve exibir apenas um nome de comando canônico por funcionalidade (ex: `tickets`).
- **Atual:** O micro help lista variações com pipe (`ticket|tickets`) e causa redundância com outros aliases (`sync-tickets`).

---
## [#48] fix(apps): corrigir identificação de forge e metadados no app tickets e demais subapps
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/48

### Description
## Description

O aplicativo `tickets` está se identificando em suas mensagens/metadados como um conector específico para as APIs do GitHub ou Codeberg. No entanto, o comportamento correto definido é o acesso genérico aos 4 tipos de forgeapps suportados em qualquer forge.

É necessário verificar a implementação e documentação interna do app `tickets` e auditar todos os outros subaplicativos para garantir que nenhum esteja com identificação de backend/forge incorreta.

## Steps to Reproduce

1. Executar o app `tickets` ou inspecionar suas descrições/logs de ajuda.
2. Verificar menções explícitas e limitantes a GitHub/Codeberg em vez da abstração de forgeapps.

## Expected vs Actual

- **Esperado:** Os apps devem se identificar de forma agnóstica de forge, indicando o suporte aos tipos de forgeapps configurados.
- **Atual:** O app `tickets` se identifica incorretamente como acessando diretamente a API do GitHub/Codeberg.

---
