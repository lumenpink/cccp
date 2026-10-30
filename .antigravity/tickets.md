# Active Tickets - cccp
> Synced at: 2026-10-30 22:38:47 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#104] fix(top): aplicar carriage return crlf e limpeza de linha no modo raw
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/104

### Description
### Descrição
No modo raw do terminal (enable_raw_mode), o caractere \n (Line Feed) apenas desce o cursor verticalmente sem efetuar o retorno de carro (Carriage Return) para a coluna 0, provocando o efeito escada (staircase effect) em que cada linha começa onde a anterior terminou.

### Requisitos
1. Padronizar todas as quebras de linha em render_top_view para \r\n.
2. Adicionar escape ANSI de limpeza de linha (\x1b[K) para evitar resíduos visuais.
3. Testes unitários comprovando que render_top_view produz quebras \r\n válidas para modo raw.

---
