# Especificação Técnica: Ticket #134
## fix(catalog): isolar auto sync e testes do catalogo real em disco

### 1. Contexto e Problema
No ticket #133, introduziu-se a sincronização automática das esteiras de CI no `catalog.json` através da função `auto_sync_catalog_ci_status`.
No entanto, foi identificada uma falha crítica de isolamento:
1. **Acoplamento Impuro com o Ambiente do Usuário**:
   - `auto_sync_catalog_ci_status` invocava `save_catalog(cat, None)`.
   - A função `save_catalog` busca o catálogo padrão em disco (`~/src/catalog.json`) quando nenhum arquivo ou configuração explícita é fornecida.
2. **Poluição por Testes Unitários**:
   - Os testes em `tests/repo_top_test.rs` executavam `auto_sync_catalog_ci_status` com dados voláteis (`test-repo`).
   - Isso fez com que a execução de testes (`cargo test`) sobrescrevesse o catálogo real do usuário em disco com dados de teste.
   - Em ambientes de CI ou workstations de desenvolvimento, isso causa falhas de hermeticidade e corrupção de dados locais.

### 2. Requisitos Técnicos
1. **Assinatura Explícita e Isolamento em `auto_sync_catalog_ci_status` (`src/repo_top.rs`)**:
   - Atualizar a assinatura da função para aceitar explicitamente o caminho do catálogo de destino:
     `pub fn auto_sync_catalog_ci_status(cat: &mut [CatalogEntry], summaries: &[CatalogRepoSummary], target_path: Option<&Path>) -> bool`
   - Se `target_path` for `None`, a função apenas atualiza as estruturas em memória (`cat`), retornando `true` se houve modificação, **sem realizar qualquer escrita em disco**.
   - Se `target_path` for `Some(path)`, e houver modificações, a gravação é realizada estritamente no arquivo apontado por `path`.
2. **Integração no Loop do Monitor (`run_repo_top`)**:
   - Resolver o caminho canônico do catálogo no início de `run_repo_top`:
     `let resolved_path = catalog_path.map(PathBuf::from).or_else(|| crate::catalog::find_catalog_path(None, None));`
   - Repassar `resolved_path.as_deref()` para `auto_sync_catalog_ci_status`.
3. **Hermeticidade Absoluta nos Testes Unitários (`tests/repo_top_test.rs`)**:
   - Testes de alteração em memória devem passar `None` para `target_path`.
   - Testes que validam persistência em disco devem utilizar estritamente diretórios temporários descartáveis (`tempfile::tempdir()`).
   - Garantir que nenhum teste unitário acesse ou grave em caminhos reais do host (`~/src/catalog.json` ou `~/.config/cccp/catalog.json`).

### 3. Critérios de Aceitação
- `auto_sync_catalog_ci_status` não grava em disco quando `target_path` é `None`.
- `auto_sync_catalog_ci_status` grava exclusivamente no arquivo fornecido quando `target_path` é `Some(path)`.
- Todos os testes de `tests/repo_top_test.rs` executam sem efeitos colaterais no filesystem do usuário.
- O catálogo real em `~/src/catalog.json` permanece intacto após `cargo test`.
- Cobertura completa de testes, `cargo clippy` e `cargo fmt` aprovados.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**:
  - **[Resumo]**: Isolamento hermético de `auto_sync_catalog_ci_status` e salvamento explícito em disco.
  - **[Solução]**: Adicionado `save_catalog_to_path` em `catalog.rs`, parâmetro `target_path: Option<&Path>` em `auto_sync_catalog_ci_status` (desativando escritas quando `target_path` for `None`), e migração de todos os testes unitários para sandboxes descartáveis com `tempdir`.
  - **[Validação]**: 26 testes unitários em `repo_top_test` aprovados em 0.00s, catálogo real do usuário inalterado durante a execução dos testes, `cargo fmt --check` e `cargo clippy` 100% limpos.
