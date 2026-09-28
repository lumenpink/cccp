# Active Tickets - cccp
> Synced at: 2026-09-28 01:15:31 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

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
