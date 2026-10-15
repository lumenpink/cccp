# Spec #127: Atualizar Contagem de Tickets Abertos no Catálogo

## 1. Contexto e Motivação
O Catálogo do Estado Gosplan (`catalog.json`) mantém um registro consolidado de todos os repositórios gerenciados pelo camarada. Atualmente, o sumário do catálogo (`cccp catalog list`) inspeciona o estado local e os arquivos `.antigravity/tickets.md`. Contudo, repositórios sem clone local ou cujo inventário de tickets ainda não foi persistido no arquivo canônico do catálogo ficam sem contagem consolidada.
O Ticket #127 implementa a sincronização e atualização da contagem de tickets abertos armazenada nos metadados do catálogo (`open_tickets`), permitindo atualizar todos os repositórios (`--all`) ou repositórios selecionados por slug, nome ou alias.

## 2. Requisitos & Aceite
1. **Modelo de Dados do Catálogo:**
   - Adicionar o campo `open_tickets: Option<usize>` na estrutura `CatalogEntry`.
   - Suportar deserialização flexível (`open_tickets` ou `tickets`) no `RawCatalogEntry`.
   - Refletir o valor persistido na sumarização e listagem do catálogo quando aplicável.
2. **Subcomando CLI:**
   - `cccp catalog update-tickets [REPOS]... [--all]` com alias `sync-tickets`.
   - Se `--all` for passado, processa todos os repositórios do catálogo.
   - Se repositórios específicos forem passados (slug, nome ou alias), processa apenas eles.
   - Se nenhum argumento for passado, detecta se o repositório do diretório atual está no catálogo; caso contrário, exibe erro orientando o uso de `--all` ou argumentos.
3. **Resiliência e Degradação Graciosa:**
   - Consulta a API da forge remota utilizando `ForgeClient`. Em caso de falha de conexão, timeout ou erro HTTP da forge, recorre à contagem de tickets locais (se houver clone local com `tickets.md` / `.antigravity/tickets.md`) ou mantém a contagem anterior sem abortar os demais itens do lote.
   - Salva o arquivo de catálogo persistido com as contagens atualizadas.
4. **Relatório e Feedback:**
   - Apresenta feedback visual detalhado informando o número de repositórios atualizados, seus respectivos tickets e eventuais avisos de conexão.
