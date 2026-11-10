# Especificação Técnica: Expurgar libgit2 e Unificar no Git CLI Nativo

**Ticket Associado:** #156  
**Branch:** `refactor/156-expurgar-libgit2-e-unificar-no-git-cli-nativo`  
**Norma:** GOST v3.0.0 / Lilya / Kollontai

---

## 1. Contexto e Motivação
O maestro `cccp` adotou no Ticket #60 a biblioteca `libgit2` (via crate `git2` sobre C-ABI) sob a premissa de viabilizar leituras rápidas e dispensar o binário `git` no ambiente local.

No entanto, a arquitetura provou-se incoerente:
1. Mais de 45 pontos do código em [`src/`](file:///home/lumen/src/cccp/src/) já invocam `Command::new("git")` diretamente para todas as operações essenciais (commit, merge, push, hooks, tags, status). O binário `git` já é um requisito absoluto de execução do sistema.
2. A inclusão da C-ABI através de `libgit2-sys` impõe um custo enorme:
   - Compilação lenta de fontes em C em cada compilação fria.
   - Dependências pesadas de toolchain C e bibliotecas de sistema (`libssl`, `openssl-sys`, `zlib`).
   - Fragilidade extrema em compilação cruzada multi-arquitetura (musl, arm64, riscv64).
   - Desalinhamento com configurações do usuário (`.gitconfig`, SSH agent, GPG signing, safe.directory).

Para Natasha, manter dois motores concorrentes de Git (leitura via C-ABI e escrita via CLI) é dívida técnica desnecessária. A decisão correta é unificar 100% das operações Git através do executável `git` do sistema.

---

## 2. Requisitos de Engenharia

### 2.1 Preservação da API Pública em `src/git.rs`
A interface pública de [`src/git.rs`](file:///home/lumen/src/cccp/src/git.rs) deve permanecer 100% retrocompatível:
- `struct GitRepo`
- `struct TagInfo`
- `struct CommitSummary`
- `GitRepo::open(path: Option<&Path>) -> Result<Self, String>`
- `GitRepo::get_recent_commits(&self, limit: usize) -> Vec<String>`
- `inspect_local_status(root: &Path) -> LocalRepoStatus`
- `get_remote_url(root: &Path) -> Option<String>`

### 2.2 Implementação Nativa via Subprocessos Git
Substituir as chamadas de `git2::Repository` por comandos subprocessados estruturados:
- Descoberta da raiz: `git rev-parse --show-toplevel`.
- Branch atual: `git branch --show-current` com fallback para `git rev-parse --abbrev-ref HEAD`.
- Identificador de HEAD: `git rev-parse HEAD`.
- Data de HEAD: `git log -1 --format=%cd --date=format:%Y%m%d`.
- Quantidade de pais de HEAD: `git rev-list --parents -n 1 HEAD`.
- Tags SemVer: `git tag --list` ordenadas ou `git describe --tags --abbrev=0`.
- Commits desde a tag: `git log <tag>..HEAD --format=%H%x1f%h%x1f%s%x1f%B%x1e`.
- Detecção de dirty: `git status --porcelain=v1`.

### 2.3 Refatoração em `src/ci_pipeline.rs`
Na função `create_pages_commit`:
- Substituir a manipulação de árvore manual via `git2::treebuilder` por encanamento git (`git write-tree` e `git commit-tree`) ou `git checkout --orphan` em diretório de trabalho temporário, desacoplando o deploy de páginas da biblioteca C.

### 2.4 Remoção da Dependência `git2`
- Remover `git2` de `Cargo.toml`.
- Eliminar qualquer link estático ou de C associado a `libgit2`.

---

## 3. Critérios de Aceitação e Quality Gate
1. `Cargo.toml` não contém a dependência `git2`.
2. Todos os testes unitários e de integração de git (`tests/git_test.rs` e `tests/git2_integration_test.rs`) passam sem regressão.
3. `cargo clippy --all-targets -- -D warnings` executa com zero avisos.
4. `cargo fmt --check` em conformidade.
5. Binário compila sem requerer toolchain ou bibliotecas C de `libgit2`.
