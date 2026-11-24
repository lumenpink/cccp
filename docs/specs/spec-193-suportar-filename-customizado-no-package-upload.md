# Especificação Técnica — Ticket #193

## 1. Problema Identificado
O comando `cccp package upload` expõe o argumento `--filename <FILENAME>` no parser de linha de comando (`cli.rs`), porém essa opção não é repassada para `PackageUploadConfig` nem utilizada na função `upload_package`. Como consequência:
- Para arquivos avulsos, o nome do arquivo publicado é extraído obrigatoriamente de `target_path.file_name()`.
- Para diretórios empacotados, o nome é formatado obrigatoriamente como `{dir_name}.tar.gz`.

Isso impede que pipelines de CI publiquem binários com nomes canônicos de distribuição (como `kn-linux-amd64` gerado a partir do binário `target/release/kn`), quebrando o download em dependentes que buscam a versão com sufixo de arquitetura.

## 2. Decisão Técnica (Árvore de Decisão para a Natasha)
- **Alternativa Descartada:** Renomear o arquivo no disco antes de chamar `cccp package upload`.
  - *Motivo do descarte:* Forçaria cópias desnecessárias e scripts adicionais em CI, contrariando o propósito do `cccp` de prover uma interface limpa e declarativa.
- **Solução Adotada:** Adicionar o campo `pub filename: Option<String>` em `PackageUploadConfig`. Em `upload_package`, se `config.filename` estiver presente, ele será usado tanto para arquivos avulsos quanto para pacotes `.tar.gz`. `resolve_package_upload_config` aceitará `override_filename: Option<&str>` e `main.rs` repassará `up.filename.as_deref()`.

## 3. Escopo de Mudanças
- `src/package.rs`:
  - Adicionar `pub filename: Option<String>` em `PackageUploadConfig`.
  - Atualizar `upload_package` para honrar `config.filename` quando fornecido.
  - Atualizar `resolve_package_upload_config` com `override_filename: Option<&str>`.
- `src/main.rs`:
  - Passar `up.filename.as_deref()` em `resolve_package_upload_config`.
- `tests/package_spec.rs`:
  - Testar que `upload_package` e `resolve_package_upload_config` respeitam o filename customizado.

## 4. Validação e Critérios de Aceite
- [x] Testes unitários cobrindo customização de `filename` para arquivos e diretórios.
- [x] `cargo fmt --check` 100% aderente.
- [x] `cargo clippy --all-targets -- -D warnings` sem warnings.
- [x] `cargo test` 100% passando.
- [x] `cccp check --strict` 100% (24/24).
