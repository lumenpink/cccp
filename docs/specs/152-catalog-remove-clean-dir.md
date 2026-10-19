# Especificação Técnica: Remoção Segura de Repositórios do Catálogo com Deleção Opcional de Diretório Limpo

**Ticket Associado:** #152  
**Branch:** `feat/152-catalog-remove-clean-dir`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Aprimorar o comando de remoção de repositórios do catálogo do CCCP (`cccp catalog remove` / `rm`), adicionando a capacidade opcional de excluir fisicamente o diretório local do repositório em disco (`--delete-dir` / `--rm-dir`), sob rigorosa trava de segurança atômica: o diretório só pode ser excluído se estiver estritamente limpo (*clean*, sem modificações não salvas, sem arquivos untracked e sem commits pendentes de envio). Caso contrário, a operação inteira deve falhar e nada deve ser alterado (nem o disco, nem o catálogo).

## 2. Requisitos Funcionais e Critérios de Aceitação

1. **Parâmetros CLI (`src/cli.rs`):**
   - Na variante `CatalogSubcommands::Remove`:
     ```rust
     #[command(alias = "rm")]
     Remove {
         /// Repository name, slug, or alias to remove from catalog
         #[arg(required = true)]
         repo: String,

         /// Also delete the local repository directory if it is completely clean
         #[arg(long = "delete-dir", alias = "rm-dir")]
         delete_dir: bool,
     }
     ```

2. **Função Core Estendida (`src/catalog.rs`):**
   - Assinatura:
     ```rust
     pub fn remove_repo_from_catalog_ext(
         identifier: &str,
         delete_dir: bool,
         config: Option<&crate::config::Config>,
     ) -> Result<(CatalogEntry, PathBuf, Option<PathBuf>), String>;
     ```
   - O terceiro elemento da tupla de retorno indica o caminho do diretório removido (se `delete_dir == true` e o diretório existia).
   - Manter retrocompatibilidade: `remove_repo_from_catalog(identifier, config)` chama `remove_repo_from_catalog_ext(identifier, false, config)`.

3. **Trava Estrita de Segurança e Falha Atômica:**
   - Se `delete_dir == true`:
     1. Localiza o clone local via `find_local_repo_path_with_config(&entry, config)`.
     2. Se o diretório local existir em disco:
        - Avalia o status via `crate::git::inspect_local_status(&local_path)`.
        - Se `!status.is_clean` (arquivos modificados, staged ou untracked) ou `status.ahead > 0` (commits locais não enviados):
          - **ABORTAR IMEDIATAMENTE COM ERRO**:
            `Cannot delete repository directory: repository is not clean (modified files: X, unpushed commits: Y). Entire operation aborted.`
          - O diretório físico **NÃO** é tocado.
          - O catálogo `catalog.json` **NÃO** é alterado.
     3. Se o repositório estiver limpo:
        - Remove o diretório físico via `fs::remove_dir_all(&local_path)`.
        - Se a exclusão física falhar, aborta sem alterar o catálogo.
     4. Remove a entrada do catálogo e salva `catalog.json`.

4. **Feedback Visual Gosplan (`src/main.rs`):**
   - Se removido apenas do catálogo:
     `★ Comrade! Removed repository '<name>' (<slug>) from State Catalog (<path>)`
   - Se o diretório também foi excluído com sucesso:
     `★ Comrade! Removed repository '<name>' (<slug>) from State Catalog and purged clean directory (<dir>)`
   - Em caso de erro:
     `Error: ...` e código de saída de falha.
