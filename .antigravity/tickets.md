# Active Tickets - cccp
> Synced at: 2026-10-13 22:58:15 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#140] fix(cli): atualizar cache de tickets no repositorio alvo em vez do cwd
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/140

### Description
## Description

Atualmente, ao registrar um novo ticket utilizando as ferramentas `divan` ou `tickets`, o `cccp` realiza a atualização da lista local de tickets no diretório atual de execução (`cwd`). Isso gera inconsistência quando o comando é executado a partir de um repositório diferente do repositório alvo da criação dos tickets, fazendo com que o repositório correto não tenha sua listagem atualizada e o repositório corrente receba atualizações indevidas.

## Expected Behavior & Acceptance Criteria
- [ ] A sincronização/atualização da lista de tickets identifica e utiliza explicitamente o diretório/contexto do repositório alvo onde a issue foi criada.
- [ ] Caso executado fora da pasta do repositório alvo (ou apontando para outro remote/forge), a atualização afeta apenas o repositório correto e não o `cwd`.
- [ ] Comportamento consistente tanto no fluxo do `divan` quanto no comando `tickets`.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#139] feat(ui): implementar soft wrap no cccp editor
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/139

### Description
## Description
Implementar suporte a quebra de linha suave (soft wrap) no motor de renderização do CCCP Editor. O texto deve quebrar visualmente nas margens da janela/terminal para facilitar leitura e navegação, sem inserir quebras de linha (`\n`) reais no arquivo ou buffer subjacente.

## Expected Behavior & Acceptance Criteria
- [ ] Linhas que excedem a largura visível da janela do editor devem sofrer wrap visual nas linhas seguintes da tela.
- [ ] Nenhuma quebra de linha (`\n` ou `\r\n`) artificial deve ser gravada no texto persistido em arquivo ou passado para o buffer real.
- [ ] Navegação do cursor (cima, baixo, esquerda, direita) deve responder adequadamente através das linhas virtuais/quebradas.
- [ ] O comportamento de contagem de linhas reais (ex.: numeração na lateral) deve permanecer alinhado ao buffer real.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#138] feat(cli): integrar revisao e submissao de tickets diretamente no cccp editor
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/138

### Description
## Description
Atualmente, a exibição prévia de tickets no terminal ocupa espaço desnecessário mostrando blocos padronizados repetitivos (como as 9 fases GOST) e decoradores visuais. Esta funcionalidade migra a revisão para dentro do CCCP Editor, simplificando o fluxo: o título do ticket ocupa a primeira linha, seguido opcionalmente de uma linha em branco e o corpo do ticket. Ao disparar o comando de envio (ex.: Ctrl+S ajustado ao contexto), o formato é validado e o ticket submetido.

## Expected Behavior & Acceptance Criteria
- [ ] Suprimir a renderização detalhada/eyecandy das 9 fases do GOST no terminal antes da postagem.
- [ ] Abrir o ticket diretamente em uma instância do CCCP Editor para revisão e ajuste.
- [ ] Primeira linha do buffer interpretada como título da issue.
- [ ] O restante do buffer interpretado como corpo da issue (ignorando uma linha em branco opcional separadora entre título e corpo).
- [ ] Mapear atalho de envio contextual (ex.: `Ctrl+S` submete o ticket em vez de salvar arquivo local comum).
- [ ] Validar o formato do ticket (título não vazio seguindo Conventional Commits, corpo não vazio) antes de disparar o envio.
- [ ] Permitir cancelar/abortar a submissão sem publicar nada ao fechar ou rejeitar no editor.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#136] feat(core): permitir a criação de apelidos para repositórios
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/136

### Description
## Description
Implementar funcionalidade para definir e resolver apelidos (aliases) para repositórios configurados. Com isso, comandos ou rotinas que recebem o identificador de um repositório podem aceitar tanto o nome canônico quanto seus apelidos cadastrados (ex.: usar `iiab` em vez de `indieinabox`).

## Expected Behavior & Acceptance Criteria
- [ ] Suporte à configuração de um ou mais aliases para cada repositório no arquivo de configuração/registro de repositórios.
- [ ] Resolução unificada de nomes: ao buscar por um repositório via alias, o sistema deve resolver com sucesso para a entidade canônica correspondente.
- [ ] Detecção e rejeição de aliases duplicados ou que entrem em colisão com nomes canônicos de outros repositórios existentes.
- [ ] Mensagens de erro claras quando um alias não puder ser resolvido ou quando houver conflito de identificador.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#135] feat(cli): suportar salvamento manual de rascunhos com atalho e gerenciamento por repositório
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/135

### Description
## Description

Permitir que o usuário salve rascunhos intencionalmente durante a redação ou revisão de tickets no Divã para alternar entre diferentes repositórios sem perder o conteúdo digitado. O rascunho deve ser salvo via atalho de teclado (`Ctrl+R`) e associado ao repositório ativo. Ao reabrir o Divã no mesmo repositório, deve ser apresentado um menu de decisão semelhante ao comportamento do Vim para arquivos de swap.

## Expected Behavior & Acceptance Criteria
- [ ] Capturar o atalho `Ctrl+R` na interface de entrada de texto para salvar o rascunho atual com feedback visual.
- [ ] Isolar o armazenamento de rascunhos por repositório (ex.: rascunho de `cccp` não interfere no de `fuxico`).
- [ ] Ao iniciar o Divã em um repositório que possua rascunho salvo, apresentar as opções:
  - Continuar o rascunho anterior;
  - Manter o rascunho e gerar novo ticket;
  - Descartar o rascunho e gerar novo ticket.
- [ ] Permitir salvar como rascunho tickets já processados pelo Gemini que estejam aguardando aprovação.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
