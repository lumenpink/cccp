# Active Tickets - cccp
> Synced at: 2026-09-30 22:54:52 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#105] refactor(legacy): remover scripts shell legados (bin/cccp, src/utils, src/hooks) apos paridade com Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/105

### Description
Remover todos os scripts legados em shell do repositorio (bin/cccp, src/utils/*.sh, src/hooks/*.sh, src/main.sh, src/config/*.sh) assim que todos os subcomandos e funcionalidades forem integralmente portados para Rust nativo e todos os testes estiverem passando com 100% de paridade.

---
## [#104] feat(update): implementar verificacao e auto-atualizacao do binario nativo em Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/104

### Description
Implementar os comandos cccp update e cccp check-update nativamente em Rust (src/update.rs), substituindo src/utils/update.sh. Suportar consulta a releases no Codeberg/forge configurada, comparacao SemVer da versao instalada versus disponivel, e atualizacao atomica do binario.

---
## [#103] feat(web): implementar interface web local e painel de controle Gosplan em Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/103

### Description
Implementar o comando cccp web / cccp cockpit nativamente em Rust (src/web.rs), substituindo src/utils/web.sh. Suportar servidor web embutido fornecendo dashboard com autenticacao por token para acompanhamento de status do repositorio, tickets, explorador e interface para Matraca e Diva.

---
## [#102] feat(matraca): implementar motor de filas e REPL offline/movel para Antigravity em Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/102

### Description
Implementar o comando cccp matraca nativamente em Rust (src/matraca.rs), substituindo src/utils/matraca.sh. Suportar fila persistente baseada no filesystem (~/.local/share/cccp/matraca/), comandos send, poll, worker, status, history e clean para comunicacao com Antigravity CLI (agy).

---
## [#101] feat(serve): implementar servidor HTTP local para preview e documentacao em Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/101

### Description
Implementar o comando cccp serve nativamente em Rust (src/serve.rs), substituindo src/utils/serve.sh. Suportar servidor HTTP estatico embutido sem dependencias externas para servir documentacao ou build local (site/, dist/, public/ ou diretorio atual) com suporte a definicao de porta, bind de interface e abertura no navegador.

---
## [#100] feat(newrepo): implementar motor de scaffolding de repositorios padrao GOST em Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/100

### Description
Implementar o comando cccp newrepo nativamente em Rust (src/newrepo.rs), substituindo src/utils/newrepo.sh. Suportar criacao de novos repositorios ou adequacao de diretorios existentes ao padrao GOST com geracao de .cccprc, .gitignore, VERSION, CHANGELOG.md, LICENSE, README.md, workflow de CI e instalacao de hooks.

---
## [#99] feat(agents): implementar sincronizacao de AGENTS.md e variaveis de agentes em Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/99

### Description
Implementar os comandos cccp agents e cccp set-av nativamente em Rust (src/agents.rs), substituindo src/utils/agents.sh. Suportar sincronizacao/download do template AGENTS_template.md, interpolacao de variaveis {{VAR}} configuradas em .cccprc, e comandos get/set/unset de variaveis de agentes.

---
## [#98] feat(deps): implementar verificador e catalogo de dependencias do sistema em Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/98

### Description
Implementar o comando cccp deps nativamente em Rust (src/deps.rs), substituindo src/utils/deps.sh. Suportar listagem, checagem e verificacao de versoes minimas de dependencias do ecossistema (git, act, docker, curl, etc.) com saida em tabela e formatacao amigavel.

---
## [#97] feat(act): implementar executor de CI local e pre-push gate com act em Rust
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/97

### Description
Implementar o comando cccp act nativamente em Rust (src/act.rs), substituindo src/utils/act.sh. Suportar deteccao de act/docker, execucao de workflows locais (.forgejo/workflows ou .github/workflows), exibicao de logs (.git/act-logs/) e checagem de status.

---
## [#69] feat(catalog): adicionar filtros e ordenação por estado e número de tickets
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/69

### Description
## Description

O comando `cataloge` precisa ser estendido para suportar filtragem e ordenação dos repositórios.

Critérios de aceitação:
- Permitir filtrar repositórios pelo estado.
- Permitir filtrar repositórios pelo número de tickets.
- Permitir ordenar a listagem pelo estado.
- Permitir ordenar a listagem pelo número de tickets.

## Steps to Reproduce

1. Executar o comando `cataloge` com opções de filtro ou ordenação (a definir).

## Expected vs Actual

- Expected: A listagem de repositórios é filtrada e/ou ordenada conforme os parâmetros informados.
- Actual: O comando atualmente não oferece essas opções de filtro e ordenação.

---
## [#68] feat(catalog): adicionar tabela de repositórios com status e contagem de tickets
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/68

### Description
## Description

Atualizar o comando `catalog` para que ele monte e exiba uma tabela formatada contendo a lista de repositórios gerenciados. A tabela deve incluir:
- Nome/caminho do repositório
- Número de tickets abertos
- Estado do repositório (`clean`, `dirty`, e indicador de commits pendentes para push).

## Steps to Reproduce

1. Executar o comando `cccp catalog` no terminal.

## Expected vs Actual

**Expected:** Uma tabela é exibida listando os repositórios, quantidade de tickets abertos e o estado do git/working tree de cada um.

**Actual:** O comando atual não exibe essas informações detalhadas ou não possui esta formatação em tabela.

---
## [#67] feat(catalog): adicionar comando cccp catalog add_new --from-src
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/67

### Description
## Description

Implementar um novo comando no catalog que permita adicionar todos os itens que estão em `src` com um único comando.

## Steps to Reproduce

1. Executar o comando `cccp catalog add_new --from-src`.

## Expected vs Actual

- Expected: Todos os itens de `src` devem ser adicionados ao catalog de uma só vez.
- Actual: O comando ainda não existe.

---
## [#66] fix(catalog): utilizar a lista de configurações em vez da pasta src
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/66

### Description
## Description

O catalog atualmente utiliza a lista que está em /src. O comportamento correto é utilizar a lista das configurações.

## Steps to Reproduce

1. Executar operações no catalog.
2. Observar que a fonte utilizada é a pasta `/src` em vez das configurações.

## Expected vs Actual

- Expected: O catalog deve utilizar a lista das configurações.
- Actual: O catalog está utilizando a lista em `/src`.

---
## [#65] feat(catalog): support adding remote repositories without prior cloning
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/65

### Description
## Description
Atualmente, o comando `catalog add_repo` exige que o repositório já esteja clonado localmente. Precisamos alterar essa lógica para suportar a adição direta de repositórios remotos utilizando apenas a URL (por exemplo, `codeberg.org/lumenpink/tralala`).

## Steps to Reproduce
1. Executar `cccp catalog add_repo codeberg.org/lumenpink/tralala` para um repositório que não existe localmente.

## Expected vs Actual
- **Expected:** O repositório é adicionado ao catálogo, baixado ou referenciado corretamente de forma remota.
- **Actual:** O comando falha por não encontrar o repositório clonado localmente.

---
