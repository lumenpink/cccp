# Especificação Técnica — Ticket #81: cccp repo top

## 1. Visão Geral e Propósito
O subcomando `cccp repo top` (com alias `cccp catalog top`) introduz um monitor em tempo real no estilo `top`/TUI para inspeção contínua e interativa da integridade e do estado de CI (esteiras e pipelines) de todos os repositórios catalogados no workspace.

## 2. Requisitos Funcionais e Critérios de Aceite
- **RF-01 (Filtros por Estado de CI)**:
  - `All`: Exibe todos os repositórios.
  - `Ok`: Repositórios com pipelines concluídos com sucesso (`pass` / `success`).
  - `Failure`: Repositórios contendo ao menos uma falha (`fail` / `failure` / `error`).
  - `Queued`: Repositórios com pipelines em fila ou em execução (`pending`, `running`, `waiting`, `progress`).
  - `NoQueue`: Repositórios sem esteira registrada ou sem pipelines pendentes.
- **RF-02 (Ordenação Configurável e Interativa)**:
  - `ProcessOrder`: Ordem canônica do catálogo (padrão inicial).
  - `CiStatus`: Prioridade de criticidade: Falhas > Em Fila/Execução > Ok > Sem Fila.
  - `RepoName`: Ordem alfabética ascendente (case-insensitive) pelo nome do repositório.
- **RF-03 (Paginação e Dimensionamento)**:
  - A exibição calcula dinamicamente a área útil baseada na altura e largura do terminal (`crossterm::terminal::size()`).
  - Desconto de linhas reservadas para header, sumário estatístico e bottom bar.
  - Suporte a rolagem/paginação navegável (linha a linha e página inteira).
- **RF-04 (Ciclo de Atualização Automática)**:
  - Polling a cada 10 segundos por padrão (configurável via `--interval`).
  - Atualização manual instantânea acionada via teclado.
- **RF-05 (Bottom Bar e Atalhos de Controle)**:
  - Barra de rodapé exibindo instrução de ajuda: `Ctrl+H help`.
  - Comandos interativos de controle precedidos por `Ctrl`:
    - `Ctrl+H`: Alterna modal/visão de ajuda com resumo das teclas.
    - `Ctrl+S` / `Ctrl+O`: Alterna critério de ordenação (`process` -> `status` -> `name`).
    - `Ctrl+F`: Alterna filtro de estado de CI (`all` -> `ok` -> `fail` -> `queued` -> `none`).
    - `Ctrl+R`: Força recarregamento imediato dos dados.
    - `Ctrl+Q` / `Esc` / `q`: Encerra o monitor e restaura o terminal.
- **RF-06 (Arquitetura Testável e Desacoplada)**:
  - Toda a lógica de filtragem, ordenação, paginação e renderização da visualização é implementada em funções e estruturas puras (`RepoTopState`, `render_top_view`), desacopladas do I/O do terminal em modo raw.
  - Permite testes de contrato determinísticos e sem dependência de TTY interativo.

## 3. Modelo de Dados e Estruturas
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiTopFilter {
    All,
    Ok,
    Failure,
    Queued,
    NoQueue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiTopSort {
    ProcessOrder,
    CiStatus,
    RepoName,
}

#[derive(Debug, Clone)]
pub struct RepoTopState {
    pub filter: CiTopFilter,
    pub sort: CiTopSort,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub show_help: bool,
    pub interval_secs: u64,
}
```

## 4. Integração na CLI
- `CatalogSubcommands::Top(RepoTopArgs)`
- `cccp repo top [--filter <FILTER>] [--sort <SORT>] [--interval <SECS>]`
- Alias: `cccp catalog top`
