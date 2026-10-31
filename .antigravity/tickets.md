# Active Tickets - cccp
> Synced at: 2026-10-31 00:18:48 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#105] fix(top): sincronizacao live da forja, filtro fail+pending e ordenacao por fila
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/105

### Description
### Descrição
1. O 'cccp repo top' não atualizava automaticamente a partir da API da forja nem resolvia token via config do tea, mantendo status estático em vez de refletir o status real dos repositórios (como kokorosim e lumenlohn).
2. Adicionar suporte ao filtro conjunto de falha e pendente ('fail+pending').
3. Definir ordenação padrão para pending/fail/pass e priorizar por ordem de chegada na fila do runner.

---
