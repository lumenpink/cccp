# Especificação Técnica: Ticket #123
## feat(ci): adicionar coluna dedicada de tempo decorrido na fila do monitor

### 1. Contexto e Problema
No monitor interativo `cccp repo top`, a visualização atual das esteiras de CI exibe o estado consolidado (`✓ pass`, `✗ fail`, `~ running`, `~ waiting`), mas não fornece informação visual sobre há quanto tempo uma tarefa está aguardando na fila de runners ou executando.
Adicionalmente, tarefas já concluídas trazem durações na descrição (ex.: `"Successful in 14m21s"`, `"Failing after 6m4s"`), mas esses dados ficam ocultos na tabela principal do monitor.
A inclusão de uma coluna dedicada `TEMPO` com largura fixa de 7 caracteres proporciona clareza imediata para identificar gargalos, filas lentas ou builds travados.

### 2. Requisitos Técnicos
1. **Cálculo e Extração de Duração (`format_ci_elapsed_time`)**:
   - Implementar em `src/repo_top.rs` (e exportar):
     `pub fn format_ci_elapsed_time(ci: &[CiPipelineStatus], now_epoch: u64) -> String`
   - Para jobs ativos na fila (`running`, `waiting`, `blocked`, `pending`):
     - Localizar o timestamp `updated_at` (ou `created_at`).
     - Converter ISO-8601 para unix epoch em UTC sem dependências externas.
     - Formatar o tempo decorrido (`now_epoch.saturating_sub(ts_epoch)`):
       - `< 60s`: `"{s}s"` (ex.: `45s`).
       - `< 60m`: `"{m}m"` (ex.: `14m`).
       - `< 24h`: `"{h}h{m:02}m"` (ex.: `1h05m`).
       - `>= 24h`: `"{d}d{h}h"` (ex.: `2d4h`).
   - Para jobs concluídos (`success`, `failure`, `fail`):
     - Se o campo `description` contiver padrão `"in <dur>"` ou `"after <dur>"`, extrair a duração (ex.: `"14m21s"`, `"6m4s"`).
     - Truncar para caber em 7 caracteres.
   - Caso não haja timestamp nem descrição de duração, ou lista vazia: retornar `"-"`.
2. **Coluna Dedicada no Layout do `repo top` (`render_top_view`)**:
   - Adicionar a coluna `TEMPO` (largura 7 caracteres) entre `ESTADO CI` e `TICKETS`.
   - Cabeçalho:
     `"  {:<repo_width$} {:<8} {:<16} {:<7} {:<7} {:<14}"` com `"REPOSITÓRIO"`, `"FORJA"`, `"ESTADO CI"`, `"TEMPO"`, `"TICKETS"`, `"LOCAL"`.
   - Ajustar o orçamento de colunas responsivo para caber exatamente em 80 colunas (`repo_width = 21` em 80 colunas, expandindo com espaço extra).
   - Estilização: Exibir tempo em ciano para `running`, amarelo para `waiting`/`blocked`, e esmaecido (`dim`) para `-` ou histórico.

### 3. Critérios de Aceitação
- A tabela do `repo top` exibe o cabeçalho `TEMPO` com 7 colunas alinhadas.
- Repositórios com pipelines ativas exibem a duração decorrida calculada com base no ISO-8601 de `updated_at`.
- Repositórios concluídos com duração na descrição exibem o tempo gasto (ex.: `14m21s`).
- Repositórios sem esteiras ou sem timestamp exibem `"-"`.
- Testes unitários cobrem o parsing de datas ISO-8601 (com timezone e Z), formatação de durações e renderização responsiva da tabela.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` aprovados com 100% de sucesso.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**:
  - **[Resumo]**: Inclusão de coluna dedicada TEMPO com largura de 7 caracteres no monitor interativo repo top, exibindo a duração de execução ativa ou tempo em fila.
  - **[Solução]**: Implementação de `parse_iso8601_to_epoch`, `format_duration_concise` e `format_ci_elapsed_time` em `src/repo_top.rs`, ajustando o orçamento de colunas responsivo e alinhamento visual.
  - **[Validação]**: 21 testes unitários e de mutação em `repo_top_test`, `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` aprovados com 100% de sucesso.
