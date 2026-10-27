# Changelog

## [Unreleased]

### Remote and Upstream Repository Isolation
- **Isolamento de Repositórios Remotos e Upstream (`src/catalog.rs`)**: Protege repositórios marcados como upstream ou puramente remotos contra intervenções acidentais e mutações locais:
  - **Bloqueio de Assemble em Upstream**: Rejeita qualquer operação de assemble em repositórios com `is_upstream = true`, preservando a integridade das referências oficiais.
  - **Exclusão Segura em Lote**: Comandos em lote ignoram automaticamente repositórios upstream e sem workspace local de trabalho, registrando-os em `skipped_remote`. (closes #66)


### Selective Specification Assemble and Working Tree Protection
- **Assemble Seletivo por Repositório e Trava de Working Tree (`cccp repo assemble` / `src/catalog.rs`)**: Refina a compilação da especificação GOST (`.gost/GOST.md`):
  - **Flag `--repo <slug>`**: Permite compilar a especificação de um único repositório por slug, nome ou apelido, sem iterar por todo o catálogo.
  - **Trava de Working Tree Limpa**: Inspeciona a working tree antes da montagem e aborta a execução caso haja modificações pendentes não commitadas, prevenindo corrupção de arquivos locais.
  - **Automação de Ciclo GOST (`--auto-commit`)**: Cria automaticamente commit canônico (`docs(gost): atualizar especificacao gost compilada`) quando houver alterações geradas em `.gost/GOST.md`. (closes #65)


### Maestro Git Locks and Origin Push Parity
- **Travas contra Comandos Git Manuais e Paridade de Push (`cccp push` / `src/push.rs` / `src/hook_runner.rs`)**: Protege o fluxo de desenvolvimento contra desvios normativos e falhas concorrentes de sincronização:
  - **Travas nos Hooks Git**: Intercepta comandos diretos `git commit` e `git push` via hooks `pre-commit` e `pre-push`, bloqueando execuções manuais não orquestradas pelo Maestro CCCP (com bypass transparente em CI e `CCCP_BYPASS_LOCKS=1`).
  - **Paridade com Tracking Branch (`origin`)**: Inspeciona a branch remota antes de qualquer envio no `cccp push`, realizando reconciliação automática fast-forward quando local está atrasado (`behind > 0, ahead == 0`) e impedindo o push em caso de divergência real de histórico (`ahead > 0, behind > 0`). (closes #64)


### GOST 2.0 Architectural and Governance Audit Compliance
- **Expansão da Auditoria Canônica para GOST 2.0 (`cccp check` / `src/audit.rs`)**: Implementa a validação estrita das novas regras de arquitetura e governança da especificação GOST v2.0.0 e perfis Kollontai/Lilya:
  - `GOST-20-SITE-TYPE-AND-SSOT-DOCS`: Exige README enxuto com teto de até 50 linhas e presença de `site/index.html` para documentação detalhada.
  - `GOST-24-ZERO-BINARIES-IN-GIT`: Veta estritamente binários e pacotes compilados no repositório (`.exe`, `.so`, `.dll`, `.dylib`, `.tar.gz`, `.zip`, `.deb`, `.rpm`, `.apk`, etc.).
  - `GOST-26-METADATA-MANIFEST-SSOT`: Valida conformidade do manifesto `.gost/metadata.json` (campos obrigatórios como `slug`, `title`, `site_type`, `theme`) e proíbe jargão técnico promocional (`soberano`, `hexagonal`, `ssot`, `rust puro`, etc.).
  - `GOST-27-DDD-HEXAGONAL-PURITY`: Garante a pureza do núcleo de domínio (`src/domain/`), vetando I/O acoplado, redes e processos (`std::fs`, `std::net`, `tokio::fs`, `sqlx`, etc.). (closes #70)


### DD-TDD Phase 7 & 8 Verification Tools: Mutate and Fast Hygiene (GOST 2.0 / Kollontai 2.0)
- **Subcomandos de Validação Ativa Local (`cccp mutate` / `cccp hygiene` / `cccp fix`)**: Implementa a automação de verificação de qualidade nas Fases 7 e 8 do ciclo DD-TDD conforme Seções 7.3 e 8 da norma GOST v2.0.0 e perfil Kollontai v2.0.0:
  - `cccp mutate [--base <b>] [--dry-run]`: Resolve as ferramentas canônicas de teste de mutação declaradas no `GOSTVER` (`cargo-mutants --in-diff` para Rust, `infection --git-diff-lines` para PHP, `stryker run --diff` para JS/TS, etc.) restritas ao diff contra a branch base.
  - `cccp hygiene [--fix] [--lang <langs>]` / `cccp fix`: Executa verificação incremental rigorosa (`-D warnings`) e formatadores determinísticos das stacks declaradas, com suporte à correção automática de estilo quando acionado com `--fix` ou via alias `cccp fix`. (closes #73)


### DD-TDD Lifecycle Subcommands and Phase Progression (GOST 2.0 / Kollontai 2.0)
- **Subcomandos de Ciclo de Vida DD-TDD (`cccp start` / `cccp commit` / `cccp close` / `cccp status`)**: Implementa o fluxo determinístico das 9 fases DD-TDD do Maestro conforme a norma GOST v2.0.0 e perfil Kollontai v2.0.0:
  - `cccp start <id> [--base <b>]`: Valida working tree limpa, resolve metadados do ticket e cria/faz checkout automático da branch padronizada `<tipo>/<id>-<slug>`.
  - `cccp commit "<resumo>"`: Deduz deterministicamente o ticket e a fase atual (`ph3`..`ph8`) a partir do branch e histórico de commits, valida as travas de fase (bloqueia código em `ph3`, exige teste vermelho em `ph4`, sucesso em `ph5`, ordem `test` -> `refactor` em `ph7`) e formata `<tipo>(<escopo>): <resumo> (ref #<id> - ph<N>)`.
  - `cccp close "<resumo>"`: Exige Laudo Textual de Resolução no corpo do commit com cabeçalhos obrigatórios `[Resumo]:`, `[Solução]:`, `[Validação]:` (>= 80 caracteres) e formata `<tipo>: <resumo> (closes #<id> - ph9)`.
  - `cccp status`: Rastreador visual do progresso de fases DD-TDD no terminal com indicação da fase atual e próximo comando obrigatório. (closes #72)


### Libgit2 Engine Integration with SSL Support
- **Migração do Motor Git para Libgit2 com SSL Nativo (`src/git.rs`)**: Substitui o motor de leitura e inspeção do repositório baseado em Gitoxide (`gix`) pela biblioteca canônica `libgit2` (crate `git2`) com suporte completo a SSL/HTTPS e OpenSSL, fornecendo leitura ultrarrápida de tags SemVer, histórico de commits, detecção de branch e estado dirty. (closes #60)


### Tickets Divan CLI Parsing & Command Aliases
- **Parsing de Parâmetros e Aliases em `cccp ticket divan` (`cccp tickets divan` / `cccp divan`)**: Suporta o alias no plural `tickets` no subcomando de tickets e adiciona o comando direto `cccp divan` no nível raiz. Habilita o parsing de repositórios qualificados (`owner/repo`), repositórios locais e opções intercaladas no Clap sem falhas na CLI. (closes #52)


### CI Pipeline Permissions Sanitization
- **Forgejo Actions Permissions Directive Removal (`.forgejo/workflows/ci.yml`)**: Remove diretiva incompatível `permissions: contents: write` do fluxo de CI no Forgejo, eliminando advertências desnecessárias do runner e auditando a ausência contínua dessa diretiva via testes de conformidade. (closes #58)


### Canonical Version Injection and GOSTVER Display Sanitization
- **Canonical Version Injection from `VERSION` & Clean GOSTVER Parsing (`cccp version` / `cccp -V` / `cccp update`)**: Injeta a versão canônica e autoritativa do repositório (extraída de `VERSION` via `build.rs` em tempo de compilação com fallback para `CARGO_PKG_VERSION`) no binário `cccp`, garantindo que `cccp version`, `cccp -V`, `cccp --version` e `get_current_version()` reportem a versão real reconciliada (ex.: `3.2.0-dev.220+...`) em vez da tag estática desatualizada no `Cargo.toml`. Adicionalmente, higieniza a leitura do arquivo `GOSTVER` no comando `cccp version` para extrair estritamente o valor de `GOST_VERSION="..."` (ex.: `1.5.0`), eliminando a exibição incorreta do arquivo multilinha bruto. (closes #158)

### Domain-Driven CLI Consolidation and Standardization
- **Domain-Driven CLI Architecture & Command Consolidation (`cccp ticket` / `cccp repo` / `cccp sync` / `cccp check` / `cccp version` / `cccp gen-version`)**: Refatora e consolida a interface de linha de comando (`cccp`) orientada a domínios de negócio claros, reduzindo o menu de ajuda de 31 para 14 comandos canônicos essenciais:
  - **Domínio `ticket`**: Unifica todas as operações com tickets sob `cccp ticket` (`list`, `new`, `divan`, `sync`, `close`, `comment`). Permite criação direta de tickets na forja remota sem abrir editor via flags CLI (`cccp ticket new -t/--title -b/--body -l/--label -r/--remote`) essencial para agentes e automações, além de absorver o Divã (`cccp ticket divan`).
  - **Domínio `repo`**: Absorve operações de governança e inventário do catálogo e criação (`cccp repo init`, `list`, `add`, `rm`, `status`, `sync-tickets`, `assemble`).
  - **Comando `sync`**: Mantido na raiz como orquestrador único de sincronização completa de ecossistema (tickets da forja, especificação GOST via `GOSTVER`, diretivas `AGENTS.md` e catálogo).
  - **Separação de `version` e `gen-version`**: `cccp version` agora exibe as versões instaladas (binário CCCP, especificação GOST e versão do projeto) com suporte a `-f json/text`, enquanto `cccp gen-version` executa o cálculo preditivo SemVer 2.0.0 via histórico git com suporte a `--bump` e `--write`.
  - **Padronização Canônica e Limpeza da Raiz**: Padroniza `cccp check [--strict]` como comando canônico de qualidade GOST (eliminando aliases confusos `audit` e `gost`), remove comandos obsoletos da raiz (`editor`, `ci`, `serve`), absorve `set-av` em `cccp agent set`, `check-update` em `cccp update --check`, e camaradas em `cccp soviet --comrades`. (closes #156)

### Continuous Integration Pipeline Status Visualization and Filtering in Catalog
- **Catalog CI Pipeline Status Visualization & Filters (`cccp catalog list --ci <status>` / `--ci-details` / `--sort ci`)**: Apresenta de forma visual e padronizada a coluna `CI` na tabela do Catálogo Gosplan (`cccp catalog list`), com indicadores semânticos (`✓ pass`, `✗ fail`, `⏳ pending`, `-`) calculados a partir das esteiras sincronizadas no catálogo ou em cache local (`.antigravity/ci_status.json`). Introduz suporte ao filtro `--ci <pass|fail|pending|none>`, flag `--ci-details` para exibição expandida com nomes, status, datas e URLs de cada pipeline, e ordenação por prioridade de atenção via `--sort ci` (falhas no topo, seguidas de pendências e sucessos). (closes #155)

### Continuous Integration Pipeline Status Collection and Sync
- **CI Pipeline Status Collection & Sync (`cccp ticket` / `cccp sync` / `cccp catalog sync`)**: Adiciona coleta e sincronização contínua do status de execução da esteira de CI mais recente para todas as esteiras e fluxos configurados na forja remota (Codeberg, Forgejo, GitHub, GitLab) via `ForgeClient::fetch_ci_status`. Persiste os metadados das pipelines no cache local (`.antigravity/ci_status.json`) e na estrutura do catálogo (`CatalogEntry::ci_status`) de forma atômica e resiliente, com degradação graciosa em caso de ausência de esteiras ou falhas de conectividade. (closes #154)

### Banca Distribution Endpoints Migration and Configurable Update Channels
- **Banca Channels Migration & Config Support (`cccp update` / `cccp check-update` / `.cccprc`)**: Migra os endpoints de distribuição remota de binários para a infraestrutura soberana da Banca (`https://banca.2lp.in/downloads/cccp/cccp-stable` e `cccp-nightly`). Introduz suporte ao canal de distribuição configurável no arquivo de governança do projeto (`.cccprc` ou `.cccprs`) através da chave `channel = stable|nightly`, mantendo precedência com flags CLI (`-c, --channel`) e variáveis de ambiente (`UPDATE_CHANNEL`). Adiciona validação estrita que rejeita canais desconhecidos com mensagem de erro explícita e resolução graciosa e transparente no updater. (closes #157)

### Pipeline Compilation and Banca Binary Distribution
- **Pipeline Compilation & Banca Artifact Delivery (`cccp update` / `cccp check-update` / `cccp tag` / `cccp hook`)**: Adequação completa do CCCP à diretiva GOST v1.5.0 para compilação estrita na esteira e distribuição de binários pela Banca (`https://banca.2lp.in/downloads/cccp/`). Elimina a execução onerosa de `act` no git `pre-push` por padrão (mantendo apenas verificação semântica instantânea ou via `CCCP_ACT_FORCE=1`), transpondo a auditoria de CI em container exclusivamente como Pre-Tag Gate durante o comando `cccp tag`. Remove dependência e fallbacks a releases do Codeberg no mecanismo de auto-update, garantindo consumo atômico soberano dos canais estável e nightly da Banca. (closes #156)

### Direct Remote Forge Ticket Count Synchronization in Catalog
- **Direct Remote Forge Ticket Count Sync (`cccp catalog sync` / `cccp sync --catalog-tickets`)**: Permite sincronizar em lote a contagem de tickets abertos de todos os repositórios cadastrados no Catálogo do Estado Gosplan (`catalog.json`) através de requisições diretas às APIs REST remotas (Codeberg, Forgejo, Gitea, GitHub e GitLab) via `ForgeClient`, sem nenhuma dependência de utilitários locais (`tea`, `gh`, `glab`) e mesmo para repositórios não clonados em disco. Implementa o método leve `fetch_open_issues_count` com consulta otimizada aos metadados das forjas, o subcomando `cccp catalog sync` (com execução global padrão ao ser chamado sem argumentos), suporte à flag `--catalog-tickets` no comando `cccp sync`, degradação graciosa com preservação do cache em caso de forjas offline e persistência atômica no catálogo. (closes #151)

### Safe Catalog Repository Removal with Clean Directory Deletion
- **Catalog Safe Removal with Clean Directory Purge (`cccp catalog remove --delete-dir` / `cccp catalog rm --rm-dir`)**: Estende o comando de remoção de repositórios do Catálogo Gosplan com suporte opcional à deleção física do diretório local do repositório em disco (`--delete-dir` ou `--rm-dir`). Implementa validação estrita de segurança e falha atômica: se o diretório local possuir arquivos modificados, alterações não rastreadas (*untracked*) ou commits pendentes de envio upstream (`ahead > 0`), a operação inteira aborta com erro, preservando intactos tanto o diretório em disco quanto a entrada no `catalog.json`. Em caso de sucesso, exclui o diretório e atualiza o catálogo de forma consistente com feedback formatado Gosplan. (closes #152)

### Dirty and Unpushed Repository Filters in Catalog Listing
- **Catalog Status Filters for Dirty and Pending Push (`cccp catalog list --dirty` / `cccp catalog list --unpushed`)**: Adiciona flags dedicadas na CLI para filtragem rápida dos repositórios do Catálogo Gosplan de acordo com o estado do git local: `--dirty` filtra exclusivamente repositórios com alterações não commitadas no working tree, enquanto `--unpushed` (ou `--pending-push`) filtra exclusivamente repositórios limpos com commits locais pendentes de envio upstream. Inclui validação de exclusão mútua (`conflicts_with`), suporte completo na mesclagem de opções de filtro e exibição detalhada com contagem de arquivos modificados (`dirty [n*]`) e commits ahead (`clean [↑n]`). (closes #129)

### Repository Open Tickets Synchronization in Catalog
- **Catalog Open Tickets Synchronization (`cccp catalog update-tickets` / `cccp catalog sync-tickets`)**: Permite consultar e atualizar a contagem de tickets abertos armazenada nos metadados dos repositórios do Catálogo do Estado (`open_tickets`), suportando atualização em lote de todo o catálogo com a flag `-a, --all` ou repositórios específicos por slug, nome ou alias. Integra com as APIs remotas de forges (Codeberg, GitHub, GitLab, Forgejo) através do `ForgeClient`, oferecendo fallback gracioso para inventários locais (`tickets.md` / `.antigravity/tickets.md`) ou contagens prévias em caso de ausência de rede, persistindo as contagens atualizadas no `catalog.json` e refletindo diretamente na listagem do catálogo. (closes #127)

### Repository Removal from Catalog
- **Catalog Repository Removal (`cccp catalog remove` / `cccp catalog rm`)**: Permite remover entradas do Catálogo do Estado Gosplan através do subcomando CLI `cccp catalog remove <repo>` (ou atalho `cccp catalog rm <repo>`), identificando o repositório de forma flexível por slug, nome canônico ou apelidos registrados (com comparação insensível a maiúsculas/minúsculas). Valida existência prévia retornando erro claro com código de saída 1 caso o alvo não exista, e atualiza o arquivo de catálogo persistido com feedback formatado. (closes #126)

### Ticket Comments Support in CLI, Divan, and Sync
- **Ticket Comments Lifecycle & Context Integration (`cccp ticket comment` / `cccp divan -c` / `cccp sync`)**: Adiciona suporte completo ao ciclo de vida de comentários em tickets nas forges suportadas (Codeberg, Forgejo, GitHub e GitLab). Implementa o subcomando CLI `cccp ticket comment <id> [-m "texto"]` com fallback para o editor interativo quando a mensagem não for especificada via flag. Adiciona o modo comentário no Divã (`cccp divan -c <id>`), estruturando o contexto da IA com o corpo da issue e histórico cronológico de comentários para propor respostas técnicas revisáveis no editor antes do envio. Enriquece a sincronização de tickets (`cccp ticket` / `cccp sync`), baixando e formatando comentários em seções estruturadas no `.antigravity/tickets.md` com respeito estrito ao teto de 10 KB. (closes #148)

### Manual Session Drafts and Repository Management in Divan
- **Manual Session Drafts & Repository Isolation (`cccp divan` / `cccp editor`)**: Permite salvar rascunhos manuais e intencionais durante a redação do prompt ou durante a revisão de tickets no editor do Divã acionando o atalho `Ctrl+R`, com feedback visual imediato na barra de status (`★ Rascunho salvo com sucesso [Ctrl+R]`) e atalho documentado no rodapé de ajuda. Isola a persistência em cache do usuário (`$XDG_CACHE_HOME/cccp/drafts/`) por caminho canônico do repositório alvo sem poluir o working tree git. Ao iniciar uma nova sessão em repositório com rascunho salvo, exibe menu de decisão estilo Vim swap file com opções `[c]` Continuar, `[m]` Manter e novo, ou `[d]` Descartar e novo. (closes #135)

### Local AI Instance & OpenAI-Compatible Endpoint Support in Divan
- **Generic Local AI Provider & OpenAI-Compatible Integration (`cccp divan` / `cccp diva`)**: Adiciona suporte completo ao provedor `AiProvider::Local` com interface HTTP padrão OpenAI (`/v1/chat/completions`) no Divã, permitindo utilizar instâncias locais como LocalAI, llama.cpp server, vLLM, LM Studio ou sidecars Gosplan. Inclui resolução de configuração via `.cccprc` e variáveis de ambiente (`LOCAL_AI_HOST`, `OPENAI_BASE_URL`, `LOCAL_AI_MODEL`, etc.), flag de atalho `--local` / `--pl` na CLI, validação de conectividade em `check_access()` e padronização do modelo local padrão para `llama3.2:3b`. (closes #149)

### Initial Cursor Position Below Comments in Divan Editor
- **Initial Cursor Positioning Below Header Comments (`cccp divan` / `cccp editor`)**: Posiciona o cursor do editor interativo automaticamente na primeira linha livre após os blocos de comentários de cabeçalho (`#`), poupando o usuário de navegar manualmente linhas de instruções iniciais antes de digitar o prompt de triagem. Reconcilia offsets de tela e viewport de rolagem vertical via `clamp_cursor()`, assegurando fallback seguro e livre de pânico em buffers sem comentários ou puramente comentados. (closes #131)

### Soft Wrap Default and Toggle in CCCP Editor
- **Default Soft Wrap and Dedicated Toggle (`cccp editor` / `cccp edit`)**: Garante que o editor abra com quebra automática de linha (*soft wrap*) ativada por padrão, fornecendo método e atalho dedicado `Alt+W` para alternar o modo sob demanda. Reconcilia os offsets de rolagem horizontal e vertical na transição (zerando `col_offset` ao ativar wrap e alinhando `row_offset` ao desativar), com sinalização do indicador `[WRAP]` na barra de status. (closes #132)

### Divan Session Checkpointing and Crash Recovery
- **Session Checkpoints & State Recovery (`cccp divan` / `cccp diva`)**: Adiciona mecanismo automático de checkpoint indexado por repositório em cache de usuário (`$XDG_CACHE_HOME/cccp/checkpoints/`), salvando prompts de triagem e tickets pendentes de revisão gerados pela IA. Detecta sessões não finalizadas na inicialização, permitindo restaurar exatamente do ponto da interrupção (pulando nova chamada à IA caso os tickets já tenham sido gerados) ou descartar, e limpando o checkpoint após conclusão bem-sucedida. (closes #134)

### Ticket Review & Submission in CCCP Editor
- **Direct Ticket Review and Submission (`cccp divan` / `cccp editor`)**: Integra a revisão e submissão de tickets do Divã diretamente no CCCP Editor, suprimindo cartões e listas prolixas de 9 fases no terminal. A primeira linha do buffer é interpretada como título (com validação estrita segundo Conventional Commits) e o restante como corpo. Adiciona barra contextual (`★ [CCCP TICKET EDITOR]`) e atalho contextual de envio (`Ctrl+S Submit` / `F2`), permitindo abortar sem efeitos colaterais com `Ctrl+Q` / `Esc`. (closes #138)

### Repository Alias Management and Collision Safety
- **Repository Aliases Management (`cccp catalog alias` / `cccp catalog add -a`)**: Adiciona suporte completo para criação, associação, listagem e remoção de apelidos (*aliases*) para repositórios registrados no catálogo Gosplan. Inclui validação estrita contra colisões de identificador (bloqueando sobreposições com nomes canônicos, slugs e apelidos de outros repositórios), suporte ao argumento `-a, --alias` na adição de repositórios remotos e resolução unificada e transparente em comandos como `divan`, `ticket` e `catalog`. (closes #136)

### Soft Wrap Support in CCCP Editor
- **Soft Wrap Rendering & Cursor Mapping (`cccp editor` / `cccp edit`)**: Implementa motor de quebra suave de linha (*soft wrap*) com mapeamento bidirecional de coordenadas entre buffer real e linhas visuais da tela. Linhas com largura superior às colunas do terminal são visualmente divididas sem inserir caracteres `\n` no buffer real ou disco. Suporta navegação vertical/horizontal fluida através dos chunks visuais, rolagem vertical contínua, indicador `[WRAP]` na barra de status e alternância via atalho `Alt+W`. (closes #139)

### GOST Version Column in Catalog Listing
- **GOST Governance Version Column (`cccp catalog list`)**: Exibe uma coluna dedicada `GOST` na tabela do catálogo Gosplan indicando a versão ativa declarada em `GOSTVER` (ex.: `1.4.0`) para repositórios locais gerenciados, ou `-` para repositórios remotos ou sem governança GOST ativa. Suporta ordenação ascendente e reversa por versão GOST via flag `--sort gost` (com alias `--sort gostver`). (closes #146)

### Automatic VERSION Synchronization in Pre-Commit and Post-Merge
- **Automatic VERSION Sync in Pre-Commit & Post-Merge**: Restaura a sincronização automática e contínua do arquivo `VERSION` no hook `pre-commit` com a versão calculada (`dev_version`) colocada em stage para o commit nascente, e adiciona reconciliação inteligente no comando `cccp merge`, gerando um commit dedicado `chore(version)` apenas quando a versão factual do novo HEAD divergir da gravada em disco (cenário de branches paralelas). (closes #147)

### Batch GOST Specification Assembly
- **Batch GOST Specification Assembly (`cccp catalog assemble` / `cccp profile assemble -a`)**: Varre os repositórios registrados no catálogo Gosplan e compila em lote a especificação canônica `.gost/GOST.md` para todos os repositórios locais que possuem `GOSTVER`. Respeita camadas de herança em cascata (`extends`), perfis ativos e sobreposições locais soberanas (`CUSTOM.md`). Suporta pré-visualização segura com `--dry-run` / `-n`. (closes #145)

### Batch Push for Clean Repositories
- **Safe Batch Push for Clean Repositories (`cccp catalog push` / `cccp push -a`)**: Varre os repositórios registrados no catálogo Gosplan e envia em lote (`git push`) todos os repositórios locais limpos (`clean`) com commits pendentes (`ahead > 0`). Ignora com aviso de segurança repositórios sujos com alterações não commitadas (`dirty`), repositórios já sincronizados e clones remotos ausentes. Suporta `--dry-run` / `-n` para pré-visualização segura. (closes #128)

### Multi-Repository Ticket Cache Isolation
- **Target Repository Ticket Cache Isolation**: Garante que tanto o `divan` quanto o comando `cccp ticket` sincronizem e atualizem `.antigravity/tickets.md` estritamente na raiz do repositório alvo (via catálogo ou caminho local), impedindo a poluição indevida do diretório corrente (`cwd`) ao operar sobre repositórios externos. (closes #140)

### Local CI Runner & Pre-Push Pause Optimization
- **Isolate Test Jobs in Local CI (`act`)**: Filtra e isola exclusivamente jobs de teste/verificação (`test`, `lint`, `check`), descartando jobs de deploy, release e publicação do workflow temporário passado ao `act`, limpando dependências `needs:` obsoletas e evitando falhas de validação de CD no pre-push hook. (closes #130)
- **Auto-pause forgejo-runner during local CI (`act`)**: Congela via `docker pause` o container daemon do `forgejo-runner` durante a execução do CI local (pre-push hook e `cccp act run`), descongelando-o deterministicamente ao término via guard RAII para evitar contenção de CPU e RAM no host. (closes #144)

### Automated Merge Ticket Closing & Merge Lock Gate
- **Merge Lock Gate (`cccp merge <branch>`)**: Bloqueia a integração de branches que não contêm commits com a tag canônica `(closes #<id>)`, garantindo conformidade irrestrita com a Fase 9 do GOST DD-TDD. (closes #133)
- **Post-Merge Git Hook (`post-merge`)**: Hook nativo de Git que inspeciona `ORIG_HEAD..HEAD`, identifica tags `(closes #<id>)` e encerra automaticamente as issues na API remota do forge. (closes #133)
- **Ticket Closure CLI (`cccp ticket close <ids...> [-m <comment>]`)**: Encerramento direto de tickets via API do forge (Codeberg/Forgejo/GitHub/GitLab) sem depender de ferramentas de terceiros. (closes #133)

## [v3.1.0] - 2026-10-03

### Multilingual Support, Divan Context Injection & Catalog Aliases

CCCP 3.1.0 expands the native standalone Rust binary with embedded internationalization, intelligent codebase context extraction for AI triage, enhanced terminal aesthetics, source installation tooling, and flexible repository catalog aliases.

#### Key Features & Improvements
- **Embedded Multilingual Internationalization (i18n)**: Zero-overhead embedded localization system supporting `en-US` (default fallback), `pt-BR`, `ru-RU`, and `es-ES` across the entire CLI with 4-tier precedence cascade (`CLI --lang < Config < ENV < Defaults`). (closes #110, #118)
- **Automatic Project Context & Symbols Injection**: Built-in manifest scanner (`Cargo.toml`, `package.json`, `pyproject.toml`, etc.), exported symbol extractor, and Gosplan token-budgeted prompt injection for Divan with `.git/cccp-context.cache` caching. (closes #116)
- **Constructivist Card Visual Formatting for Tickets**: High-aesthetic box-drawing cards with ANSI-aware padding and word-wrapping for ticket reviews in Divan. (closes #119)
- **CLI Commands Overhaul & Rationalization**: Unified commands under clear mnemonic English (`commit`, `version`, `lint`, `audit`, `sync`, `divan`), deprecating redundant aliases and unifying synchronization. (closes #115)
- **Soviet Constructivist Terminology Restored**: Reinstated ideological Party Line compliance, Order No. 227 directives, and full 9-point GOST compliance audit. (closes #120)
- **Compilation & Installation Infrastructure**: Root `install.sh` and POSIX `Makefile` with automatic toolchain verification, custom prefix/bindir, staging destdir, and shell PATH validation. (closes #121)
- **Repository Aliases in Catalog**: Support for short aliases (`alias` and `aliases`) in `catalog.json` with safety collision detection, enabling fast repo references in CCCP commands and Divan. (closes #123)
- **Comprehensive Test Suite & Quality Inspectorate**: 27 test targets covering linting, cultural themes, command parsing, E2E binaries, i18n dictionaries, and installer verification with 100% pass rate. (closes #111, #112, #113, #114)

## [v3.0.0] - 2026-09-30

### Complete Rust Migration & Multiplatform Release

CCCP 3.0.0 is a complete, ground-up rewrite in high-performance Rust, replacing the legacy POSIX shell implementation with a single, statically linked binary with zero external runtime dependencies (no libc, no curl, no OpenSSL).

#### Architecture & Multiplatform Support (32-bit & 64-bit)
- **First-Class 32-bit & 64-bit Binaries**: Official static builds for Linux and macOS via `cargo-zigbuild`:
  - `linux-386` (`i686-unknown-linux-musl`) — 32-bit x86 systems
  - `linux-armv7` (`armv7-unknown-linux-musleabihf`) — 32-bit ARM (Raspberry Pi, embedded devices)
  - `linux-amd64` (`x86_64-unknown-linux-musl`) — 64-bit x86_64 systems
  - `linux-arm64` (`aarch64-unknown-linux-musl`) — 64-bit ARM (AArch64)
  - `darwin-amd64` (`x86_64-apple-darwin`) — macOS Intel
  - `darwin-arm64` (`aarch64-apple-darwin`) — macOS Apple Silicon
- **Universal POSIX Installer**: Automated bootstrap script (`site/install.sh`) detecting all 6 targets with SHA-256 verification and zero external tools.

#### Local Offline AI & Forge Integrations
- **Local Ollama Integration**: Added offline AI-driven ticket triage via local Ollama daemon (`cccp diva --provider ollama --model <model>`) alongside Google Gemini API (`cccp diva --provider gemini`) and direct manual triage (`--direct`).
- **Multi-Forge Tickets Synchronizer**: Unified issue synchronization (`cccp tickets`) supporting Codeberg, Forgejo, GitHub, and GitLab with embedded TLS (`rustls`).

#### Developer Experience & Performance
- **Sub-3ms Git Hooks**: Lightning-fast git hook execution (`cccp hook commit-msg` and `cccp hook pre-push`) with automated hook installation (`cccp hook install`).
- **Interactive Commit Wizard**: Terminal commit wizard (`cccp cz`) built with `inquire`, offering intuitive type/scope selection, breaking change prompts, and GPG/SSH signing passthrough.
- **Strict Linting & SemVer Calculation**: Conventional Commits 1.0.0 validator (`cccp lint`) and predictive SemVer 2.0.0 release calculator (`cccp semver`) with tag inspection via `gix`.
- **Gosplan Cultural Aesthetics & Config**: 4-tier hierarchical configuration (`.cccprc`, XDG, ENV, CLI flags), authentic ASCII banners, and strict `NO_COLOR` standard compliance.

#### Legacy Features & Tooling

  - (triage) adicionar opcao para postagem direta sem intermedio do gemini (closes #55)
  - (cli) aceitar repositorio como parametro posicional unico no diva (closes #54)
  - (deps) implementar gerenciador de dependencias com list, check e install (#61)
  - (cli) implementar comando newrepo para scaffolding de repositórios no modelo gost (closes #57)
  - (cockpit) unificar interface grafica no comando cockpit com selecao de tabs (closes #59)
  - (matraca) integrar matraca como web repl movil tolerante a falhas no cccp (closes #58)
  - (act,pre-push) implement pre-push hook with act: local CI gate, live spinner and structured logs (closes #37)
  - (repo) implementar visão geral e gestão unificada de múltiplos repositórios (closes #38)
  - (serve) implementar comando unificado cccp serve (estático, dev e demo) (closes #39)
  - (gost,check) suporte a conformidade GOST (leitura de GOSTVER, cccp gost sync, check e upgrade) (closes #47) (closes #42)
  - (cockpit) adicionar seletor de repositorio ativo e suporte global a -C (closes #43)
  - (repo,diva,catalog) unify forge identity by domain and software engine by forgeapp (closes #46) (closes #34)
  - (web) adotar opt-in --open, resolucao de chave via env/flag e autenticacao no cockpit (closes #44) (closes #45)
  - (web) merge web interface and mayak command bridge (closes #41)
  - (diva) add cccp ticket/issue command with branch suggestion (closes #36)
  - (comrades) support mayak (mk) static micro-server in comrades manager
  - (web) implement cccp web command and mayak command bridge (ref #41)
  - (comrades) support ARM32 architectures, 1960s-style raw bootstrap downloaders, and auto-recruitment
  - (deps) implement pure POSIX token reader for gh and issue url parser without jq (#33)
  - (diva) integrar editor kvant (kv) para edicao de tickets com navegacao por teclado
  - (llm) permitir rediscutir parametros e refinar triagem com a Gemini no diva
  - (ui) permitir edicao manual de titulo e corpo dos tickets no diva
  - (cli) suportar nome parcial de repositorio na opcao -r do diva

### Bug Fixes
  - (apps) corrigir identificacao de forge e metadados no app tickets e subapps (closes #48)
  - (diva) usar comrade kv (kvant) como editor de texto padrao (closes #50)
  - (act) assegurar extensao yml em workflow temporario para o act (#61)
  - (act) corrigir flag pull e compatibilidade com workflows forgejo no runner (#61)
  - (diva) validar existência local do repositório informado via flag -r (closes #35)
  - (diva,tickets) suporte a forgejo, persistencia de drafts e correcao de hosts hardcoded

## Previous Releases

### [v2.3.0] - v2.3.0

### Features
  - (tag) recompile and stage cccp.sh in release commit
  - (catalog) suporte a diretório padrão via config global, env e flags
  - (diva) display target repository and forge in prompts and ticket preview
  - (diva) update default gemini model to gemini-3.6-flash
  - (diva) fallback to ~/.env for gemini api key resolution when unset in memory
  - (tooling) establish POSIX shell formatting standards and linting automation
  - (diva) integrate gemini-powered interactive ticket triage bot into posix shell
  - (catalog) port catalog metadata engine and forge synchronizer to posix shell
  - (web) implement hashbang polyglot installer and dedications section
  - (update) refactor cccp update and install routines for cccp.2lp.in channels (closes #17)
  - (installer) polyglot html/shell installer and loving tribute to Lisi (closes #15) (closes #16)
  - (dist) configure stable and nightly release publishing on cccp.2lp.in (closes #14)
  - (ci) migrate cloudflare deployment to workers static assets with custom domain
  - (site) update header to standalone svg badge, add author and unlicense links
  - (site) add programmer credit with trans flag to footer (closes #9)
  - (site) adicionar site estatico bilingue, design sovietico e servidor de preview (closes #7)
  - (tickets) implement generic forge detection and multi-driver ticket sync (#5)
  - (tickets) add GitHub issues sync and Gosplan rate limiter
  - (agent) add fill subcommand and generate template despite missing variables
  - (codeberg) add cccp codeberg setup command (#4)
  - (agent) standardise command to singular with plural fallback
  - (hooks) add sync-tickets and sync-agents to post-push hook (#3)
  - (cli) wire agents command and autocompletion into cccp (#2)
  - (agents) implement dynamic AGENTS.md generator and variable manager (#2)
  - (cli) add dynamic codeberg tickets synchronization command
  - (hooks) implement optional post-push hook automation and cccp push command

### Bug Fixes
  - (generator) prevent truncation of long/tall tickets in output
  - (generator) fix ASCII table right border and text wrapping in columns
  - (install) prevent same-file error and prioritize workspace script in install -g
  - (bin) include catalog and diva in unbundled development runner
  - (hooks) respect XDG_BIN_HOME and CCCP_INSTALL_DIR when running as root
  - (site) restore standard html5 document structure and use worker content negotiation
  - (hooks) keep only the latest hook backup on install (closes #6)
  - (tickets) handle empty issue lists gracefully without false error warning
  - (agent) persist --url to .cccprc automatically when passed via CLI
  - (agent) correct URL resolution precedence and add content validation
  - (agents) align confirmation output and JSON handling (#2)
  - (agents) support multi-line variables and exact awk substring replacement (#2)
  - (hooks) allow non-semver utility tags and bypass remote sync in reference-transaction

### [v2.2.0]

### Features
  - (hooks) implement pre-push and reference-transaction tag guards and CI check

### [v2.1.0] - v2.1.0 - Gosplan State Inspection & SemVer Protocol

### Features
  - (config) enforce global-only scope for tool update directives
  - (version) absorb breaking changes in X.0.0 pre-releases and promote on non-zero pre-releases
  - (hooks) provide detailed hooks audit, diffing, and intervention guide
  - (update) add check-update command and version pinning support
  - (help) display active version in all help headers

### Bug Fixes
  - (version) target base version when latest tag is a pre-release
  - (core) establish single-source version injection and eliminate recursive invocations

### [v2.0.0] - disable update check in spec_helper and cli_spec to prevent test warnings

### Features
  - (validation) enhance prerequisite checks without awk dependency
  - (core) introduce modular dispatcher and development runner

### Bug Fixes
  - (core) ensure safe sourcing and robust configuration hierarchy across modules
  - (changelog) omit empty section headings and add default release fallback

### [v1.7.0] - v1.7.0 - Teleprinter Telegraph (Shell Autocompletion)

### Features
  - (completion) add shell autocompletion for bash, zsh, and fish

### [v1.6.0] - v1.6.0 - Soviet Assembly (Interactive Commit Wizard)

### Features
  - (interactive) add pure shell terminal commit wizard

### [v1.5.0] - v1.5.0 - Party Line Compliance (Commit Linting)

### Features
  - (lint) add history and pull request commit linting

### [v1.4.0] - v1.4.0 - State Inspection (Repository Status)

### Features
  - (status) add repository status and health diagnostic command

### [v1.3.0] - v1.3.0 - Gosplan Directive (Custom Types and Scopes)

### Features
  - (config) support custom types, scopes, and descriptor lookup

### [v1.2.0] - v1.2.0 - Sputnik Protocol

### Features
  - (tag) support release titles with -t and --title flags
  - (update) add update checking, release channels and hook version synchronization
  - (install) support global CLI installation to PATH with shell profile integration
  - (config) add hierarchical configuration system and cccp config CLI command
  - (hooks) use standalone portable wrapper scripts with version tracking
  - (config) standardize default flags to 0 and set base version to 0.0.1

### Bug Fixes
  - (tag) exclude VERSION and CHANGELOG.md from pre-tag dirty check
  - (ci) configure git default branch to silence hints during spec tests
  - (ci) install shellspec to user space and export PATH
  - (ci) correct YAML escaping for multiline release notes and quote workflow name

### [v1.1.0] - v1.1.0

### Features
  - (tag) automate release commit creation and working tree validation
  - (cli) add command-specific help and release tagging command
  - (version) implement predictive semantic versioning, prerequisite checks and scope validation fixes

### [v1.0.2] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v1.0.1] - test

### Features
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - add updatescript function

### [v1.0.0] - first version

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.9] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.8] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.7] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.6] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.5] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.4] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.3] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.2] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.1] - test

### Features
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe

### [nightly] - v2.2.0 - Gosplan Tag Defense & Gatekeeper Bureau

### Features
  - (hooks) implement pre-push and reference-transaction tag guards and CI check
  - (config) enforce global-only scope for tool update directives
  - (version) absorb breaking changes in X.0.0 pre-releases and promote on non-zero pre-releases
  - (hooks) provide detailed hooks audit, diffing, and intervention guide
  - (update) add check-update command and version pinning support
  - (help) display active version in all help headers
  - (validation) enhance prerequisite checks without awk dependency
  - (core) introduce modular dispatcher and development runner
  - (completion) add shell autocompletion for bash, zsh, and fish
  - (interactive) add pure shell terminal commit wizard
  - (lint) add history and pull request commit linting
  - (status) add repository status and health diagnostic command
  - (config) support custom types, scopes, and descriptor lookup
  - (tag) support release titles with -t and --title flags
  - (update) add update checking, release channels and hook version synchronization
  - (install) support global CLI installation to PATH with shell profile integration
  - (config) add hierarchical configuration system and cccp config CLI command
  - (hooks) use standalone portable wrapper scripts with version tracking
  - (config) standardize default flags to 0 and set base version to 0.0.1
  - (tag) automate release commit creation and working tree validation
  - (cli) add command-specific help and release tagging command
  - (version) implement predictive semantic versioning, prerequisite checks and scope validation fixes
  - add updatescript function
  - first version

### Bug Fixes
  - (version) target base version when latest tag is a pre-release
  - (core) establish single-source version injection and eliminate recursive invocations
  - (core) ensure safe sourcing and robust configuration hierarchy across modules
  - (changelog) omit empty section headings and add default release fallback
  - (tag) exclude VERSION and CHANGELOG.md from pre-tag dirty check
  - (ci) configure git default branch to silence hints during spec tests
  - (ci) install shellspec to user space and export PATH
  - (ci) correct YAML escaping for multiline release notes and quote workflow name

