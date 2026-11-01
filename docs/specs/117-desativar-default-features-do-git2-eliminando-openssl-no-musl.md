# Especificação Técnica: Ticket #117

## 1. Contexto e Problema
A esteira de publicação para a Banca (`deploy-banca` em `.forgejo/workflows/ci.yml`) falhava com erro de linkagem estática (`undefined reference to fopen64`) ao compilar o executável para `x86_64-unknown-linux-musl`.
A investigação revelou que a dependência `git2` trazia por padrão as funcionalidades de transporte de rede (`https` e `ssh`), puxando os crates C `openssl-sys` e `libssh2-sys`. Na compilação C do OpenSSL (`o_fopen.c`), era gerada uma referência a `fopen64`, símbolo inexistente na biblioteca padrão musl (onde `fopen` é 64-bit nativo).
Além disso, tentativas anteriores com `musl-gcc` causavam colisão de runtime CRT e falha de segmentação (*SIGSEGV* código 139).
Como o `cccp` utiliza o `git2` exclusivamente para manipulação local de repositório, objetos, commits e cálculo SemVer — e já utiliza `ureq` com `rustls` puro para operações de rede HTTP —, o suporte a transporte de rede C no `git2` é desnecessário.

## 2. Requisitos Técnicos
1. No `Cargo.toml`, atualizar a dependência do `git2` para desativar features padrão:
   `git2 = { version = "0.20", default-features = false }`.
2. Assegurar que `openssl-sys` e `libssh2-sys` sejam completamente removidos da árvore de dependências (`Cargo.lock`).
3. Preservar integralmente todas as capacidades autônomas de manipulação Git local (`Repository::discover`, `Repository::open`, `check_is_dirty`, árvores e commits).
4. No arquivo `.forgejo/workflows/ci.yml`, manter o linker e flags estáticas em conformidade com o ambiente musl.
5. Em `tests/ci_pipeline_test.rs`, adicionar testes verificando que `Cargo.toml` não ativa `default-features` nem `vendored-openssl` no `git2`.

## 3. Critérios de Aceite
- Compilação estática de `x86_64-unknown-linux-musl` bem-sucedida, sem erros de linkagem ou referências a `fopen64`.
- O teste de fumaça `./target/x86_64-unknown-linux-musl/release/cccp --version` deve executar com código 0 e exibir a versão preditiva.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos.
- `tests/ci_pipeline_test.rs` aprovado com asserções positivas e de mutação.

## 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**: Configuração `git2 = { version = "0.20", default-features = false }` aplicada em Cargo.toml, eliminando openssl-sys e libssh2-sys da árvore de dependências. O binário estático x86_64-unknown-linux-musl compila limpo sem fopen64 e executa cccp --version com código 0.
