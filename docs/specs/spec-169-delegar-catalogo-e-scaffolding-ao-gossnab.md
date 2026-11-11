# Especificação Técnica: Delegar Catálogo e Scaffolding ao Gossnab (Ticket #169)

## 1. Contexto e Motivação
O `cccp` concentrava a lógica de scaffolding de novos repositórios e de mutação de catálogos multi-repositório. Para desarmar a concorrência em arquivos locais e consolidar o Gossnab como fonte canônica do ecossistema, os comandos de gerenciamento de catálogo (`repo`) e provisionamento (`new`) foram delegados formalmente ao `gossnab`.

## 2. Decisões Arquiteturais e Alternativas Descartadas (Para a Natasha)
- **Delegação Transparente com Preservação de CLI:**
  - *Decisão:* No `cmd_newrepo` e nos comandos de catálogo, tentar despachar diretamente para o binário `gossnab` (`gossnab new` e `gossnab catalog`) quando presente, mantendo a rotina local desacoplada como fallback caso o gossnab não esteja instalado.
  - *Alternativa descartada:* Purgar os comandos de imediato da CLI sem aviso ou delegação. Descartada para evitar quebra abrupta de scripts existentes de desenvolvedoras.

## 3. Resultados da Execução
- Atualização em `src/newrepo.rs` com delegação direta ao Gossnab.
- Atualização de descrições dos comandos de catálogo na CLI em `src/cli.rs`.
- Suíte de testes `newrepo_test` e `catalog_test` 100% aprovada.
