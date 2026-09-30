# Active Tickets - cccp
> Synced at: 2026-09-30 00:56:37 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#86] [rust] feat(installer): script de bootstrap para cccp.2lp.in com download do binário estático
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/86

### Description
## Contexto
Manter a experiência de instalação em 1 linha (`curl -fsSL https://cccp.2lp.in | sh`) baixando o binário estático compilado.

## Requisitos
- Atualizar o script de instalação servido em `site/install.sh`:
  - Detectar `OS` (`Linux`, `Darwin`) e `ARCH` (`x86_64`, `aarch64`, `armv7l`).
  - Buscar a última release do Codeberg.
  - Baixar o binário estático correspondente.
  - Instalar em `~/.local/bin/cccp` com permissão de execução.
- Validar integridade via hash SHA-256.

## Critérios de Aceitação
- Executar `curl -fsSL https://cccp.2lp.in | sh` instala o binário funcional em qualquer distribuição Linux ou macOS.

---
## [#85] [rust] ci(build): workflow de compilação estática musl multiplataforma
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/85

### Description
## Contexto
Automatizar a compilação contínua e release de binários estáticos no Codeberg Actions / Forgejo Actions.

## Requisitos
- Criar `.forgejo/workflows/release.yaml`:
  - Compilação estática para `x86_64-unknown-linux-musl`.
  - Compilação estática para `aarch64-unknown-linux-musl` (ARM64).
  - Compilação nativa para macOS `x86_64` e `aarch64` (Apple Silicon).
- Gerar checksums SHA-256 de todos os artefatos gerados.
- Anexar os binários aos releases de tags do repositório.

## Critérios de Aceitação
- Releases geram automaticamente binários executáveis prontos para download.

---
## [#84] [rust] feat(tickets): sincronizador de issues para .antigravity/tickets.md
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/84

### Description
## Contexto
Sincronização de tickets de forges remotos para o arquivo de contexto de IA `.antigravity/tickets.md`.

## Requisitos
- Buscar issues abertas no forge configurado do repositório.
- Formatar as issues em Markdown estruturado com:
  - Número e título com tags.
  - Estado, autor e data.
  - Corpo da descrição.
- Respeitar o limite de tamanho máximo (ceiling de 10 KB).

## Critérios de Aceitação
- Atualização limpa de `.antigravity/tickets.md` com paridade de formato em relação ao script anterior.

---
## [#83] [rust] feat(forges): cliente HTTP com rustls para Codeberg/Forgejo, GitHub e GitLab
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/83

### Description
## Contexto
Consumo de APIs de forges Git sem dependência externa do binário `curl` ou de OpenSSL dinâmico.

## Requisitos
- Implementar cliente HTTP assíncrono ou síncrono enxuto (`ureq` ou `reqwest` com `rustls`).
- Suporte a tokens de autenticação via variáveis de ambiente (`CODEBERG_TOKEN`, `GITHUB_TOKEN`, etc.).
- Tratamento de rate limiting com recuo exponencial e logs informativos.

## Critérios de Aceitação
- Conexões HTTPS 100% seguras com certificados raiz embutidos via `webpki-roots`.

---
## [#82] [rust] feat(hook-runner): despachador interno cccp hook commit-msg e pre-push
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/82

### Description
## Contexto
Ponto de entrada de altíssima performance para os hooks invocados pelo Git.

## Requisitos
- Implementar subcomando interno `cccp hook commit-msg <path>`:
  - Lê o arquivo de mensagem fornecido pelo Git.
  - Executa o linter em < 3ms.
  - Aborta o commit se inválido com mensagem clara e didática.
- Implementar `cccp hook pre-push`:
  - Executa rotina de validação e CI local pré-push.

## Critérios de Aceitação
- Execução imperceptível em tempo de resposta durante o fluxo normal de desenvolvimento.

---
## [#81] [rust] feat(hooks): gerador de stubs mínimos e instalador de hooks (.git/hooks/)
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/81

### Description
## Contexto
O CCCP instala hooks locais no repositório Git para garantir a conformidade dos commits.

## Requisitos
- Implementar subcomando `cccp install` que instala em `.git/hooks/`:
  - `commit-msg`: stub mínimo de 2 linhas delegando para `cccp hook commit-msg "$1"`.
  - `pre-push`: stub delegando para `cccp hook pre-push`.
  - `post-commit`: stub delegando para `cccp hook post-commit`.
- Criar backup automático (`.bak`) de hooks pré-existentes.
- Suportar flag de desinstalação `cccp uninstall`.

## Critérios de Aceitação
- Hooks instalados funcionam com qualquer shell POSIX (`/bin/sh`) invocando o binário `cccp`.

---
## [#80] [rust] feat(commit-exec): despachador de commit via Git CLI com suporte a GPG/SSH signing
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/80

### Description
## Contexto
Executar o commit mantendo total compatibilidade com assinaturas criptográficas GPG/SSH e agentes locais do usuário.

## Requisitos
- Após a composição da mensagem no wizard `cccp cz`, invocar o binário `git commit` via `std::process::Command`.
- Passar a mensagem formatada de forma segura (via arquivo temporário ou stdin) para evitar problemas de escape de shell.
- Preservar TTY interativo para prompts de senha do SSH-agent ou pinentry do GPG.

## Critérios de Aceitação
- O commit é assinado corretamente quando `commit.gpgsign = true` estiver habilitado no Git do usuário.

---
## [#79] [rust] feat(cz): wizard interativo de commit de terminal com inquire/dialoguer
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/79

### Description
## Contexto
Substituir o wizard interativo de commit do shell por uma experiência de terminal moderna e fluida.

## Requisitos
- Implementar fluxo interativo de perguntas:
  1. Tipo de alteração (menu de navegação por setas com descrições).
  2. Escopo da alteração (opcional, com histórico ou busca).
  3. Mensagem concisa do título (com contador de caracteres em tempo real e alerta visual em > 72 chars).
  4. Descrição detalhada do corpo (opcional).
  5. Houve quebra de compatibilidade (Breaking Change)?
  6. Referências a issues/tickets fechados.
- Preview da mensagem completa formatada antes de comitar.

## Critérios de Aceitação
- Interface intuitiva, navegável por teclado, sem travamentos de TTY.

---
## [#78] [rust] feat(lint): analisador sintático estrito de mensagens de commit
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/78

### Description
## Contexto
O CCCP audita mensagens de commit para garantir conformidade estrita com o padrão Conventional Commits.

## Requisitos
- Implementar validador de sintaxe:
  - Estrutura: `<type>(<scope>): <subject>`
  - Tipos válidos: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
  - Limite de comprimento: linha de título <= 72 caracteres.
  - Ausência de ponto final no título.
  - Linha em branco obrigatória separando título do corpo.
- Retornar erros detalhados e sugestões de correção.

## Critérios de Aceitação
- Subcomando `cccp lint --message "..."` valida ou rejeita mensagens informando a regra violada.

---
## [#77] [rust] test(semver): bateria de testes unitários para cálculo preditivo de versão
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/77

### Description
## Contexto
Garantir que a matemática de versões semânticas preditivas cubra todos os cenários de borda.

## Requisitos
- Testes cobrindo:
  - Sem tags prévias (bootstrap da versão inicial `0.1.0`).
  - Transições `patch` vs `minor` vs `major`.
  - Múltiplos commits combinados com pesos diferentes.
  - Pré-releases e metadados de build com hash curto do git.

---
## [#76] [rust] feat(semver): calculadora preditiva de SemVer 2.0.0 a partir de Conventional Commits
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/76

### Description
## Contexto
Cálculo preditivo autônomo da próxima versão semântica (SemVer 2.0.0) com base no histórico de commits desde a última tag.

## Requisitos
- Utilizar a crate `semver` para parsing e comparação de versões.
- Implementar as regras de progressão:
  - `feat`: incrementa MINOR (ex: `1.2.0` -> `1.3.0`).
  - `fix`, `perf`, `refactor`: incrementa PATCH (ex: `1.2.0` -> `1.2.1`).
  - `BREAKING CHANGE` ou indicador `!` (ex: `feat!:`): incrementa MAJOR (ex: `1.2.0` -> `2.0.0`).
- Suportar pré-releases incrementais (ex: `-dev.X+hash`).

## Critérios de Aceitação
- Implementação 100% aderente à especificação SemVer 2.0.0.

---
## [#75] [rust] feat(git-reader): leitor do repositório local e histórico de tags via gix (Gitoxide)
- **Status**: `open`
- **Labels**: rust
- **Link**: https://codeberg.org/lumenpink/cccp/issues/75

### Description
## Contexto
O CCCP precisa analisar histórico de commits, tags e estado do repositório em frações de milissegundo, sem invocar subprocessos de shell.

## Requisitos
- Utilizar a crate `gix` (Gitoxide, 100% puro Rust) para:
  - Localizar a raiz do repositório Git a partir do diretório atual.
  - Ler referências e tags anotadas/leves.
  - Ler mensagens de commit a partir da última tag de versão até `HEAD`.
  - Inspecionar status de modificações na working tree (clean/dirty).

## Critérios de Aceitação
- Leitura do histórico recente e tags realizada em < 5ms em repositórios com milhares de commits.

---
