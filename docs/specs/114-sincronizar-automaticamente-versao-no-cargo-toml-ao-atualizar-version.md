# Especificação Técnica: Ticket #114
## feat(version): sincronizar automaticamente versao no Cargo.toml ao atualizar VERSION

### 1. Contexto e Motivação
Nos projetos do ecossistema da Banca geridos pelo CCCP em Rust, a versão oficial de lançamento e pré-lançamento é controlada pelo arquivo `VERSION`.
Entretanto, o arquivo `Cargo.toml` (seja na raiz de um pacote ou em crates de um workspace) frequentemente permanece estático com a versão inicial (ex.: `0.1.0-dev.1`).

Como o ecossistema Cargo e bibliotecas CLI padrão como `clap` utilizam nativamente `env!("CARGO_PKG_VERSION")`, os binários compilados acabam exibindo versões obsoletas ao serem invocados com `--version`.

### 2. Requisitos Técnicos
1. **Módulo de Sincronização de Manifestos Cargo (`sync_cargo_toml_versions`)**:
   - Detectar a existência de `Cargo.toml` na raiz do repositório.
   - Atualizar a chave `version = "..."` sob a seção `[package]` ou `[workspace.package]`.
   - Se for um workspace Cargo com membros declarados (`members = [...]`), localizar e atualizar também o `[package].version` nos manifestos dos membros que possuem versão declarada própria.
   - Preservar a formatação, comentários e ordem do arquivo TOML.
   - Retornar a lista de caminhos relativos dos manifestos alterados para inclusão no stage (`git add`).

2. **Pontos de Integração no CCCP**:
   - **Pre-commit hook (`src/hook_runner.rs`)**: Ao sincronizar o arquivo `VERSION` para o próximo commit, executar a sincronização do `Cargo.toml` e incluir os arquivos alterados com `git add`.
   - **Pós-merge (`reconcile_merge_version` em `src/hook_runner.rs`)**: Ao reconciliar o `VERSION` divergente após merge, atualizar os arquivos `Cargo.toml` correspondentes e incluí-los no commit de reconciliação.
   - **Releases e Tags (`src/release.rs`)**: Ao gerar o commit de tag/release com a versão estável, atualizar os manifestos `Cargo.toml` e incluí-los no stage.
   - **Cálculo SemVer (`src/semver_calc.rs`)**: Na função `write_version_file`, sincronizar também os arquivos `Cargo.toml` quando existentes.

### 3. Critérios de Aceitação
- Atualizações em projetos com pacote único (`[package].version`) sincronizam o `Cargo.toml` raiz.
- Atualizações em projetos com múltiplos crates (`workspace.members`) sincronizam tanto o manifesto raiz quanto os crates membros.
- Se o repositório não for Rust (sem `Cargo.toml`), o comportamento para outros tipos de projeto permanece inalterado.
- Os arquivos modificados são adicionados ao stage do git juntamente com o arquivo `VERSION`.
- Testes unitários e de integração validam a sincronização em repositórios simulados mono-pacote e workspace.
