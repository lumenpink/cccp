# Active Tickets - cccp
> Synced at: 2026-10-13 20:55:03 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#141] feat(cli): versionar e auto-atualizar git hooks com deteccao de modificacao local
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/141

### Description
## Description
Atualmente, a execução do `cccp` emite alertas constantes sobre os hooks do repositório, mesmo quando estes não necessitam de intervenção ou quando o próprio hook não sofreu alterações upstream. Como o binário do `cccp` é atualizado com muito mais frequência do que a lógica dos hooks, é necessário implementar um sistema de controle de versão dos hooks com detecção de integridade e auto-atualização segura.

### Mecanismo Proposto para Detecção de Alterações
Para identificar se o hook foi modificado pelo desenvolvedor ou se permanece o template original da versão antiga:
1. Incluir metadados no cabeçalho do hook gerado:
   - `CCCP_HOOK_VERSION`: número serial inteiro incremental (ex: `1`, `2`, `3`).
   - `CCCP_HOOK_CHECKSUM`: hash SHA-256 gerado a partir do conteúdo do hook no momento da instalação (ou comparação contra uma tabela de hashes de templates conhecidos das versões anteriores).
   - `CCCP_LAST_VERIFIED`: versão do cccp que fez a checagem/instalação.
2. Ao rodar o `cccp`:
   - Se `CCCP_HOOK_VERSION < VERSAO_ATUAL`:
     - Verifica se o conteúdo atual confere com o checksum registrado (ou com o template canônico daquela versão):
       - **Não modificado:** Atualiza o hook automaticamente e emite mensagem informativa amigável no terminal.
       - **Modificado manualmente:** Mantém o hook intacto e exibe alerta alertando sobre a defasagem e a customização local preservada.

## Expected Behavior & Acceptance Criteria
- [ ] Cabeçalho dos hooks gerados/instalados inclui metadados padronizados (`CCCP_HOOK_VERSION`, `CCCP_HOOK_CHECKSUM`, `CCCP_LAST_VERIFIED`).
- [ ] Execução do `cccp` detecta se o hook instalado é de versão inferior à versão embutida.
- [ ] Hooks desatualizados e não modificados são sobrescritos automaticamente para a versão mais recente, informando o usuário via stdout/stderr.
- [ ] Hooks que sofreram modificações locais não são sobrescritos às cegas; um alerta é emitido indicando a customização.
- [ ] Hooks já na versão atual não geram avisos espúrios.
- [ ] Testes unitários e de integração cobrindo cenários: hook ausente, hook idêntico, hook desatualizado sem alteração, hook desatualizado modificado.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `feat/hook-versioning-autoupdate` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
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
