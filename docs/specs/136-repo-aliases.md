# Especificação Técnica: Criação e Gerenciamento de Apelidos de Repositórios

**Ticket Associado:** #136  
**Branch:** `feat/136-repo-aliases`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Permitir a criação, atribuição, remoção e resolução unificada de apelidos (*aliases*) para repositórios registrados no catálogo do CCCP, com validação de unicidade e prevenção de colisões contra nomes canônicos, slugs e outros apelidos.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Cadastro de Repositório com Apelidos:**
   - O comando `cccp catalog add` aceita o argumento `-a, --alias <ALIAS>` (múltiplas ocorrências ou valores separados por vírgula) para registrar apelidos já no momento da adição do repositório.
2. **Gerenciamento Dedicado de Apelidos (`cccp catalog alias`):**
   - `cccp catalog alias add <REPO> <ALIASES...>`: adiciona um ou mais apelidos ao repositório especificado.
   - `cccp catalog alias remove <REPO> <ALIASES...>`: remove apelidos do repositório especificado.
   - `cccp catalog alias list [REPO]`: lista apelidos de um repositório específico ou de todo o catálogo.
3. **Prevenção e Rejeição de Colisões:**
   - Um apelido é rejeitado se for idêntico (case-insensitive) ao nome canônico de qualquer repositório no catálogo.
   - Um apelido é rejeitado se for idêntico (case-insensitive) ao slug (`owner/repo`) de qualquer repositório no catálogo.
   - Um apelido é rejeitado se já estiver em uso por outro repositório cadastrado.
   - Apelidos duplicados dentro do mesmo repositório são normalizados/desduplicados automaticamente.
4. **Resolução Unificada de Nomes:**
   - Comandos como `resolve_target_repo`, `cccp divan`, `cccp ticket` e `cccp catalog` resolvem o identificador usando a seguinte ordem de precedência:
     1. URL remota
     2. Slug canônico exato (`owner/repo`)
     3. Nome canônico exato
     4. Apelido cadastrado
     5. Descoberta no sistema de arquivos local (`~/src/<name>`, `~/src/<alias>`)
5. **Mensagens de Erro Construtivistas:**
   - Erros detalhados indicando exatamente o tipo de conflito (com o nome canônico ou com outro repositório) e identificador problemático.

## 3. Modelo de Dados e Funções no Core
- Métodos em `src/catalog.rs`:
  - `add_repo_aliases(repo: &str, new_aliases: &[String], config: Option<&Config>) -> Result<CatalogEntry, String>`
  - `remove_repo_aliases(repo: &str, aliases_to_remove: &[String], config: Option<&Config>) -> Result<CatalogEntry, String>`
  - `validate_alias_collision(target_slug: &str, alias: &str, catalog: &[CatalogEntry]) -> Result<(), String>`
  - Atualização de `add_remote_repo`: suporte a `aliases: &[String]`.
