# Active Tickets - cccp
> Synced at: 2026-09-30 13:55:05 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#87] feat(ai): adicionar suporte ao provedor local Ollama
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/87

### Description
## Description
Na branch rust, implementar suporte para utilizar a IA local através do Ollama como alternativa à API do GEMINI.

## Steps to Reproduce
1. Mudar para a branch rust
2. Tentar configurar a aplicação para usar uma IA local (Ollama)
3. Constatar que apenas o GEMINI é suportado

## Expected vs Actual
Expected: Poder configurar e utilizar o Ollama localmente na branch rust.
Actual: Apenas o provedor GEMINI está disponível.

---
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
- **Status**: `closed`
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
## [#64] fix(deps): corrigir erro de shift na linha 12991
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/64

### Description
## Description
O comando `cccp deps` está apresentando um erro de execução na linha 12991 com a mensagem `shift: cant shif that many`.

## Steps to Reproduce
1. Executar o comando `cccp deps` no repositório.
2. Observar o erro impresso no terminal na linha 12991.

## Expected vs Actual
- Expected: O comando `cccp deps` deve ser executado com sucesso sem erros de shift no shell.
- Actual: O script falha com o erro `shift: cant shif that many` na linha 12991.

---
## [#63] feat(editor): integrar microeditor embutido
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/63

### Description
## Description

Adiva ainda não está utilizando o microeditor embutido conforme esperado.

## Steps to Reproduce

1. Abrir a aplicação/diva
2. Tentar editar um arquivo ou texto
3. Observar que o editor padrão utilizado não é o microeditor embutido.

## Expected vs Actual

Expected: O sistema deve utilizar o microeditor embutido.
Actual: O sistema está utilizando outro editor ou o editor padrão antigo.

---
