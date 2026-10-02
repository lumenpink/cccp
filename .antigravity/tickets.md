# Active Tickets - cccp
> Synced at: 2026-10-02 01:30:29 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#122] refactor(core): remover scripts e artefatos shell legados remanescentes
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/122

### Description
## Description

Apesar dos esforços recentes de migração para Rust, ainda restam arquivos, scripts e artefatos legados do modelo antigo em shell script no repositório. É necessário realizar uma auditoria completa na árvore do projeto para identificar, revisar e remover em definitivo todos os componentes que se tornaram obsoletos e não são mais utilizados pela versão em Rust.

## Tarefas

- [ ] Realizar busca por scripts shell (`*.sh`) e arquivos auxiliares do modelo legado no repositório.
- [ ] Verificar se há dependências ou referências a esses scripts em documentações, workflows de CI ou templates.
- [ ] Remover os scripts e artefatos obsoletos identificados.
- [ ] Garantir que o build e os testes em Rust continuem passando sem regressões.

## Expected vs Actual

- **Expected:** Repositório enxuto contendo apenas o ecossistema e fontes Rust, sem resquícios de implementações obsoletas em shell.
- **Actual:** Arquivos residuais do modelo legado em shell continuam presentes na árvore do projeto.

---
## [#121] feat(build): adicionar script de compilacao e instalacao do binario
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/121

### Description
## Description

Atualmente, a compilação e instalação do binário `cccp` precisa ser feita manualmente via comandos do Cargo. Sentiu-se a falta de um script no repositório (por exemplo, `install.sh` ou alvo no `Makefile`) que compile o projeto em release e copie ou instale o binário no seu devido local (ex.: `~/.local/bin`, `/usr/local/bin` ou diretório apropriado do ambiente).

## Expected vs Actual

- **Esperado:** Existência de um script de instalação no repositório (ex.: `./install.sh` ou `make install`) que faça o build em release (`cargo build --release` ou `cargo install --path .`) e instale o executável no PATH padrão do usuário.
- **Atual:** Não há script de compilação/instalação direta disponível no repositório após a migração para Rust.

## Critérios de Aceitação

- [ ] Criar script de instalação documentado no repositório.
- [ ] Garantir verificação prévia da presença do toolchain Rust/Cargo.
- [ ] Definir e documentar o diretório de destino padrão da instalação (com possibilidade de sobrescrever o prefixo/destino).
- [ ] Atualizar o README/documentação com instruções de uso do script.

---
## [#120] feat(cli): restaurar mensagens e terminologia tematica sovietica nos comandos Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/120

### Description
## Description
Durante a migração dos scripts legados em shell para o binário Rust (e a recente reformulação da CLI), as mensagens temáticas soviéticas (como verificação de 'adequação ideológica' no comando `check`) foram removidas ou substituídas por saídas neutras/convencionais.

É necessário inspecionar os commits e versões anteriores (antes de `refactor(legacy): remover scripts shell legados apos paridade Rust`) para mapear as mensagens de status, avisos e saídas dos comandos antigos e restaurar a identidade estilística no ecossistema Rust.

## Steps to Reproduce
1. Executar comandos de checagem/validação na versão Rust (ex: `cccp check` ou equivalente).
2. Observar a saída de texto padrão/genérica sem a estilização ideológica soviética original.

## Expected vs Actual
- **Esperado:** As mensagens de status, logs e validações devem manter o vocabulário temático (ex: inspeção de adequação ideológica, conformidade com os planos do Gosplan, etc.), preservando a experiência dos scripts legados.
- **Atual:** Mensagens padronizadas em inglês ou neutras que descaracterizam o clima soviético da ferramenta.

---
## [#119] feat(cli): restaurar formatacao visual de ticket com bordas na saida do diva
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/119

### Description
## Description

Na versão em shell script do cccp, os tickets produzidos pelo comando de triagem (diva) eram renderizados no terminal com um formato gráfico de ticket estilizado, utilizando bordas decorativas e margens demarcando cada issue.

Com a migração para Rust, a saída perdeu essa apresentação estética. Esta issue propõe resgatar a exibição em caixa/cartão de ticket para as issues triadas.

## Expected vs Actual

- **Esperado:** A visualização dos tickets gerados pelo diva no terminal deve apresentar moldura/bordas e margens (estilo cartão de ticket ASCII/Unicode).
- **Atual:** A saída do diva no terminal não conta mais com a moldura e formatação de ticket presente na versão legada.

---
## [#118] feat(core): implementar suporte a idioma padrão
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/118

### Description
## Description

Implementar um mecanismo de idioma padrão que permita definir uma língua default para o projeto. Isso deve ser capaz de lidar com a internacionalização (Intl) para garantir que todos os elementos do sistema sejam exibidos e utilizados corretamente em diferentes idiomas.

## Steps to Reproduce

1. Adicionar um arquivo de configuração para definir a língua padrão.
2. Implementar funções para detectar o idioma do usuário ou da sessão atual.
3. Criar uma interface para permitir que os usuários escolham a língua preferida.
4. Atualizar todos os componentes (tickets, comandos, funções e variáveis) para usar a linguagem padrão.
5. Implementar suporte ao Intl para garantir que as strings sejam traduzidas corretamente.

## Expected vs Actual

- Todos os elementos do sistema devem ser exibidos na língua padrão definida.
- A internacionalização deve funcionar corretamente, permitindo a translação de todas as mensagens e textos para diferentes idiomas.

---
## [#117] feat(cli): add 'update' command for self-update
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/117

### Description
## Description

Add a new command `update` to the CLI that checks for available updates and performs an automatic update if necessary.

## Steps to Reproduce
1. Run `cccp update`
2. The CLI will check for any available updates and prompt the user to proceed with the update.
3. If an update is available, the CLI will download and install it automatically.

## Expected vs Actual
- **Expected**: The application should be updated to the latest version without manual intervention.
- **Actual**: The application should check for updates and offer to perform them automatically.

---
## [#116] feat(cli): injetar contexto do projeto automaticamente na execucao do diva
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/116

### Description
## Descrição

Ao executar o comando `diva`, o assistente deve extrair e repassar automaticamente o contexto relevante da aplicação/repositório (como estrutura do projeto, símbolos, assinaturas de funções ou visão geral da arquitetura) para o modelo.

Dessa forma, o usuário pode referenciar funções, módulos ou componentes diretamente no prompt sem a necessidade de explicar manualmente sua existência ou propósito.

## Comportamento Esperado

- O comando `diva` inspeciona automaticamente a base de código/projeto atual antes de enviar a requisição ao modelo.
- O contexto coletado (manifests, arquivos principais, definições de funções ou resumo do projeto) é incorporado ao payload/system prompt da sessão de forma transparente.
- O modelo passa a reconhecer termos, funções e módulos existentes no projeto sem necessidade de flags manuais de contexto.

---
## [#114] test(e2e): end-to-end integration tests for compiled cccp standalone binary
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/114

### Description
Implement end-to-end tests in tests/e2e_cli_test.rs invoking the compiled cccp binary directly across realistic workflows: repo initialization, hook install and intercept, lint stdin, config, and completion.

---
## [#113] test(cli): functional tests for command parsing, global flags, and dispatch
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/113

### Description
Implement functional tests in tests/cli_test.rs covering CLI command dispatch, global flags (-C/--dir, -v/-vv, -q, --no-color), and parameter validation.

---
## [#112] test(cultural): unit tests for theme colors, soviet registry, and order 227
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/112

### Description
Implement unit tests in tests/cultural_test.rs for NO_COLOR compliance, constructivist color palette, historical registry, and Order 227 («Ни шагу назад!»).

---
## [#111] test(lint): comprehensive unit and functional tests for GOST commit linting
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/111

### Description
Implement exhaustive test coverage for src/lint.rs (tests/lint_test.rs) covering types, scopes, 72-char header ceiling, breaking changes, and commit sanitization.

---
## [#110] feat(i18n): implement internationalization (i18n) and multilingual support
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/110

### Description
Implement pure Rust embedded i18n architecture with en-US default fallback, supporting en-US, pt-BR, ru-RU, and es-ES. Resolve language via CLI flag, CCCP_LANG, LC_ALL/LANG, and config.

---
