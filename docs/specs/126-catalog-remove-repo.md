# Especificação Técnica: Remoção de Itens do Catálogo

**Ticket Associado:** #126  
**Branch:** `feat/126-catalog-remove-repo`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Permitir a remoção determinística de repositórios registrados no catálogo do Gosplan através da CLI (`cccp catalog remove` ou `cccp catalog rm`), localizando o item alvo por slug, nome canônico ou apelido configurado (alias).

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Função Core de Remoção (`src/catalog.rs`):**
   - Assinatura:
     ```rust
     pub fn remove_repo_from_catalog(
         identifier: &str,
         config: Option<&crate::config::Config>,
     ) -> Result<(CatalogEntry, std::path::PathBuf), String>;
     ```
   - Localização flexível: case-insensitive por `entry.slug`, `entry.name` ou qualquer string presente em `entry.aliases`.
   - Se o repositório não for encontrado, retorna erro explicativo:
     `Repository '<identifier>' not found in catalog`.
   - Se encontrado, remove do array, persiste as alterações no arquivo `catalog.json` via `save_catalog` e retorna a entrada removida e o caminho do catálogo.
2. **Subcomando CLI (`src/cli.rs` e `src/main.rs`):**
   - Variante `Remove` no enum `CatalogSubcommands`:
     ```rust
     #[command(alias = "rm")]
     Remove {
         /// Repository name, slug, or alias to remove from catalog
         #[arg(required = true)]
         repo: String,
     }
     ```
   - Exibe mensagem com formatação Gosplan:
     `★ Comrade! Removed repository '<name>' (<slug>) from State Catalog (<path>)`
