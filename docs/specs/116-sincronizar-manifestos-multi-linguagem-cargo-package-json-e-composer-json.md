# Especificação Técnica: Ticket #116
## feat(version): sincronizar manifestos multi-linguagem Cargo, package.json e composer.json

### 1. Contexto e Motivação
A sincronização introduzida no Ticket #114 cobriu arquivos `Cargo.toml` baseados na declaração de `members` no workspace raiz. Contudo, em repositórios da Banca que empregam monorepos ou possuem bibliotecas avulsas em subpastas (`libs/`, `packages/`, etc.), ou em projetos baseados em JavaScript/TypeScript (Node/Bun) e PHP (Composer), os arquivos de manifesto mantêm versões estáticas desatualizadas.

Adicionalmente, no ecossistema PHP, é requisito do projeto que a chave `"version"` conste expressamente no arquivo `composer.json`, garantindo rastreabilidade sem dependência de ferramentas externas.

### 2. Requisitos Técnicos
1. **Despachante Unificado (`sync_project_manifests`)**:
   - Substituir a chamada restrita a Cargo por um despachante geral em `src/semver_calc.rs`:
     `pub fn sync_project_manifests(repo_root: &Path, new_version: &str) -> Result<Vec<PathBuf>, String>`.
   - Compatibilidade retroativa: manter `sync_cargo_toml_versions` delegando ou apontando para a rotina de sincronização de manifestos Cargo.

2. **Varredura Recursiva com Filtro de Diretórios**:
   - Percorrer o repositório identificando manifestos suportados (`Cargo.toml`, `package.json`, `composer.json`).
   - Ignorar estritamente diretórios de compilação, dependências e controle de versão:
     `.git`, `target`, `node_modules`, `vendor`, `dist`, `build`, `coverage`, `.antigravity`.

3. **Regras de Atualização por Formato**:
   - **`Cargo.toml`**: Atualizar `version = "..."` sob as seções `[package]` e `[workspace.package]`, preservando comentários e dependências em subcrates e bibliotecas avulsas.
   - **`package.json`**: Atualizar o campo de nível superior `"version": "..."`, preservando a formatação e a indentação original (2 espaços, 4 espaços ou tabulação).
   - **`composer.json`**: Garantir a presença do campo de nível superior `"version": "..."`. Caso já exista, atualizá-lo; caso esteja ausente, injetá-lo preservando a formatação do JSON.

4. **Pontos de Integração no CCCP**:
   - `src/hook_runner.rs` (`run_pre_commit` e `reconcile_merge_version`).
   - `src/release.rs` (`create_tag_and_sync`).
   - `src/semver_calc.rs` (`write_version_file`).
   - Inclusão automática de todos os manifestos modificados no `git add`.

### 3. Critérios de Aceitação
- Sincronização de todos os `Cargo.toml` encontrados no repositório fora das pastas ignoradas.
- Sincronização de arquivos `package.json` em projetos Node/Bun e monorepos.
- Sincronização e injeção do campo `version` em arquivos `composer.json` em projetos PHP.
- Preservação rigorosa de formatação e ausência de efeitos colaterais em dependências.
- Testes cobrindo cenários mistos (Rust + Node + PHP no mesmo workspace).
