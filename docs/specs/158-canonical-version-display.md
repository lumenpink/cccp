# Especificação Técnica: Resolução e Exibição da Versão Canônica e Higienização do GOSTVER

**Ticket Associado:** #158  
**Branch:** `fix/158-canonical-version-display`  
**Norma:** GOST 1.5.0  

## 1. Objetivo
Garantir que a versão reportada pelo binário `cccp` (via `cccp version`, `cccp -V`, `cccp --version` e no módulo de atualização `update`) seja a versão canônica e autoritativa do repositório (SemVer 2.0.0 extraído do arquivo `VERSION` em tempo de compilação), em vez da tag da última release estática gravada no `Cargo.toml`. Adicionalmente, higienizar a leitura do `GOSTVER` no comando `cccp version` para exibir estritamente a versão e perfil declarados (ex.: `1.5.0`), em vez do arquivo multilinha bruto.

---

## 2. Requisitos Arquiteturais e Implementação

### 2.1. Injeção da Versão Canônica em Tempo de Compilação (`build.rs`)
- Criar script de compilação `build.rs` na raiz do crate `cccp`.
- O script lê o arquivo `VERSION` (se presente no momento da compilação):
  - Limpa quebras de linha e espaços.
  - Caso ausente ou vazio, utiliza `CARGO_PKG_VERSION` como fallback gracioso.
- Exporta a variável de ambiente de compilação:
  `cargo:rustc-env=CCCP_CANONICAL_VERSION=<versão>`
- Configura o rastreamento de mudanças:
  `cargo:rerun-if-changed=VERSION`

### 2.2. Exibição Canônica no CLI e Módulos
- **`src/cli.rs`**:
  - Configurar `#[command(version = env!("CCCP_CANONICAL_VERSION"))]` na estrutura `Cli` para que `cccp -V` e `cccp --version` reportem a versão canônica compilada.
- **`src/main.rs` (`Commands::Version`)**:
  - Obter a versão do CCCP via `env!("CCCP_CANONICAL_VERSION")`.
  - Exibir: `★ CCCP v<CCCP_CANONICAL_VERSION>`.
  - Higienizar a leitura do arquivo `GOSTVER`: iterar pelas linhas procurando `GOST_VERSION=`, extrair aspas e exibir somente a versão (ex.: `1.5.0`). Caso não encontre, reportar `unknown` ou `none` se o arquivo não existir.
  - Para `Project Version`, exibir o conteúdo do `VERSION` do repositório corrente (se presente).
- **`src/update.rs` (`get_current_version`)**:
  - Manter suporte ao override por variável de ambiente `CCCP_VERSION`.
  - Como fallback padrão, utilizar `env!("CCCP_CANONICAL_VERSION")` em vez de `CARGO_PKG_VERSION`.

---

## 3. Critérios de Aceitação
1. `cccp -V` e `cccp --version` exibem a versão canônica (ex.: `cccp 3.2.0-dev...` quando compilado na presença de `VERSION`).
2. `cccp version` exibe `★ CCCP v<versão_canônica>` e na linha `GOST Specification` exibe estritamente a versão extraída de `GOSTVER` (ex.: `1.5.0`), em uma única linha limpa.
3. Se executado com `-f json`, o campo `"cccp"` reflete a versão canônica e `"gost"` reflete a versão limpa.
4. Testes automatizados validam a extração correta de `CCCP_CANONICAL_VERSION`, a sanitização de `GOSTVER` e a consistência do updater.
