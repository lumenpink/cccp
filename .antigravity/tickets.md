# Active Tickets - cccp
> Synced at: 2026-09-24 01:07:31 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#32] feat(llm): permitir rediscutir parâmetros e refinar triagem com a Gemini
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/32

### Description
## Description

Adicionar um fluxo de refinamento interativo para que o usuário possa rediscutir o resultado da triagem e ajustar parâmetros com a IA Gemini.

## Steps to Reproduce

1. Realizar a triagem de um relato.
2. Tentar solicitar ajustes ou refinar o escopo com a IA.

## Expected vs Actual

- **Expected**: Interface que permita enviar novas instruções/ajustes de parâmetros para a Gemini atualizar a proposta de tickets.
- **Actual**: A triagem é executada em disparo único sem canal de refatoração ou discussão com o modelo.

---
## [#31] feat(ui): permitir edição manual do título e corpo dos tickets
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/31

### Description
## Description

Permitir que o usuário edite o título e o corpo do ticket diretamente na interface do O Divã do Gosplan antes de confirmar ou exportar.

## Steps to Reproduce

1. Realizar a triagem de um relato ou entrada.
2. Tentar alterar o título ou a descrição do ticket gerado.

## Expected vs Actual

- **Expected**: Os campos de título e corpo devem ser editáveis pelo usuário.
- **Actual**: Os dados gerados são exibidos como texto estático sem opção de edição manual.

---
## [#30] feat(diva): pesquisar e integrar novo editor de texto com suporte a navegação por teclado
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/30

### Description
## Description

O editor de texto do Divã atualmente é limitado e não suporta navegação por teclado, o que prejudica a acessibilidade e a experiência do usuário. É necessário pesquisar e implementar um novo componente de edição de texto que resolva essas limitações.

## Steps to Reproduce

1. Acessar a interface do Divã.
2. Tentar utilizar o editor de texto apenas com comandos e navegação do teclado (Tab, setas, atalhos).
3. Observar a impossibilidade de navegar livremente pelos elementos ou utilizar controles do editor.

## Expected vs Actual

- **Expected:** O editor de texto deve oferecer suporte completo a atalhos de teclado e navegação acessível.
- **Actual:** O editor é limitado e não aceita navegação adequada via teclado.

---
## [#29] feat(cli): suportar nome parcial de repositório na opção -r
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/29

### Description
## Descrição

Atualmente, ao utilizar a opção `-r` na CLI do divã, é necessário informar o nome completo do repositório no formato `proprietario/repositorio` (ex: `lumenpink/diva`).

Esta funcionalidade permite que o usuário passe apenas o nome do repositório (ex: `-r diva`). Caso a busca encontre um único repositório correspondente, ele deve ser selecionado automaticamente.

## Comportamento Esperado vs Atual

- **Atual:** Exige o nome completo no formato `proprietario/repositorio` ao passar `-r`.
- **Esperado:** Permitir `-r diva` e resolver para `lumenpink/diva` se for o único repositório correspondente. Se houver ambiguidade (mais de um repositório com o mesmo nome), deve exibir um mensagem de erro solicitando o nome completo.

---
## [#28] bug(diva,tickets): forge provider detection hardcoded to codeberg.org — forgejo/generic instances silently discarded
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/28

### Description
## Problema

O CCCP assume `codeberg.org` como host padrão para forjas do tipo `forgejo` em múltiplos lugares do código, mesmo quando o repositório está hospedado em uma instância Forgejo independente. Isso causa falhas silenciosas e **perda de dados** do usuário.

As forjas reais suportadas são: **github**, **sourcehut**, **gitlab**, **forgejo** (instâncias genéricas — incluindo codeberg.org, tildegit.org e self-hosted). `codeberg` não é um provider distinto no modelo de dados — é uma instância forgejo.

## Arquivos Afetados e Bugs Específicos

### 1. `src/utils/diva.sh` — Bug principal (reportado pelo usuário)

- **L360–L422** (`_diva_present_and_dispatch`): `case "$_forge"` só trata `codeberg|tildegit` e `github`. Quando o provider é `forgejo` (instância genérica), cai no `*)` com `[INFO] Publicação automática não suportada para ${_forge}.` — **sem erro, sem persistência do draft, sem exit code não-zero**. O ticket gerado pelo Gemini é silenciosamente descartado.
- **L552** (`cmd_diva`): quando `--repo` é passado manualmente, `_forge` é forçado para `"codeberg"` em vez de `"forgejo"`.
- **L362**: `_base_url` hardcoded `https://codeberg.org` para `tildegit` (errado — deveria usar o host real).

### 2. `src/utils/tickets.sh` — Defaults incorretos

- **L41**: `codeberg|forgejo) resolved_host="codeberg.org"` — forgejo genérico sempre recebe host de codeberg.
- **L44, L387**: `*) resolved_host/host="codeberg.org"` — fallback universal para codeberg para providers desconhecidos.
- **L74**: Fallback de provider não-identificado assume `forgejo` com host `codeberg.org`.
- **L79**: `codeberg|forgejo) resolved_host="${resolved_host:-codeberg.org}"` — pode substituir host correto pelo de codeberg.
- **L384**: `forgejo) host="codeberg.org"` — explicitamente sobrescreve o host detectado do remote para codeberg.
- **L434**: Chama `resolve_forge_token "codeberg"` para qualquer instância forgejo — retorna token errado para self-hosted.

### 3. `src/utils/catalog.sh` — L500–L501

- `codeberg|tildegit` compartilham `_base_url="https://codeberg.org"` — tildegit tem URL distinta (`https://tildegit.org`).

## Comportamento Esperado

1. **`diva.sh`**: Quando forge é `forgejo` (qualquer instância Gitea-compatible), publicar via API REST usando o `$_host` real detectado. Se publicação falhar, **salvar draft** em `.antigravity/diva_drafts/<timestamp>_<forge>_<slug>.json` e **falhar ruidosamente** no stderr (exit ≠ 0).
2. **`tickets.sh`**: Usar o host real detectado via `detect_repo_info()` — nunca sobrescrever com `codeberg.org` se o host já foi resolvido. Fallback deve ser vazio ou erro explícito.
3. **`catalog.sh`**: Separar `codeberg` e `tildegit` em cases distintos com suas URLs reais.

## Procedimento de Correção Proposto

### diva.sh
- Adicionar `forgejo` como case explícito em `_diva_present_and_dispatch`, usando `$_host` para montar `_base_url` (API Gitea-compatible).
- Implementar `_diva_save_draft()`: salva o ticket em `.antigravity/diva_drafts/<timestamp>_<forge>_<owner>_<repo>.json`.
- No `*)` fallback: chamar `_diva_save_draft`, imprimir erro no stderr, retornar 1.
- Corrigir L552: `_forge="forgejo"` para repos passados via `--repo`.
- Passar `$_host` como 6º argumento para `_diva_present_and_dispatch`.

### tickets.sh
- Quando host já foi detectado via git remote, não sobrescrever com `codeberg.org`.
- L434: Usar `resolve_forge_token "forgejo"` para instâncias genéricas (com fallback para `codeberg` se host = codeberg.org).
- Fallback de host desconhecido: emitir warning no stderr, não assumir codeberg.

### catalog.sh
- Separar `tildegit` em case próprio: `_base_url="https://tildegit.org"`.


---
## [#27] feat(catalog): suporte a diretório padrão via config global, env e flags
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/27

### Description
## Descrição  A função `catalog` atualmente não possui um diretório padrão definido. É necessário implementar a resolução do diretório alvo utilizando a seguinte ordem de precedência:  1. Flag passada pela linha de comando 2. Variável de ambiente 3. Configuração global do `cccp` 4. Valor fallback: `~/src`  ## Passos para Reproduzir  1. Executar a função/comando `catalog` sem especificar um diretório por parâmetro/flag. 2. Verificar a ausência de um fallback ou busca na configuração global.  ## Esperado vs Atual  - **Esperado**: A função `catalog` deve determinar o diretório utilizando a hierarquia (Flag > Env > Config Global > `~/src`). - **Atual**: A função não possui suporte a diretório padrão.

---
## [#26] fix(generator): prevent truncation of long/tall tickets in output
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/26

### Description
## Description  Ao gerar tickets com grande volume de texto ou muitas linhas, o resultado final está sendo exibido apenas parcialmente (truncado).  ## Steps to Reproduce  1. Criar ou gerar um ticket com conteúdo extenso (altura/linhas elevadas). 2. Exibir a saída do gerador. 3. Observar que o ticket é cortado e apenas uma parte é apresentada.  ## Expected vs Actual  - **Esperado:** O ticket deve ser exibido integralmente, independentemente da altura/quantidade de linhas. - **Atual:** Apenas uma parte do ticket é exibida quando ele excede uma determinada altura.

---
## [#25] fix(generator): fix ASCII table right border and text wrapping in columns
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/25

### Description
## Description  A representação em tabela ASCII do gerador de tickets não está fechando a borda da margem direita e não aplica quebra de linha (text wrapping) no conteúdo da coluna X, prejudicando a apresentação visual.  ## Steps to Reproduce  1. Gerar um ticket que contenha texto longo na coluna X. 2. Visualizar a tabela ASCII gerada. 3. Observar que a borda lateral direita está ausente/desalinhada e o texto não quebra linha.  ## Expected vs Actual  - **Esperado:** A tabela ASCII deve ser exibida com a borda direita devidamente fechada e com quebra de linha automática nas colunas para manter a largura consistente. - **Atual:** A borda direita fica aberta e o texto da coluna X é impresso sem quebras de linha.

---
