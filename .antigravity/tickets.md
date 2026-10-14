# Active Tickets - cccp
> Synced at: 2026-10-14 18:30:07 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#149] feat(api): suportar instancia local de IA para o Divan
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/149

### Description
## Description
Atualmente o módulo de inteligência artificial (`src/ai.rs`) interage com provedores remotos pré-definidos. É necessário possibilitar o uso de instâncias locais de IA (por exemplo, Ollama, LocalAI ou qualquer servidor HTTP com endpoint compatível com OpenAI) para processar as requisições do Divã do Gosplan sem dependência de chaves de API proprietárias externas.

## Expected Behavior & Acceptance Criteria
- [ ] Suporte a configuração de endpoint customizado de IA (URL base, modelo e chave opcional) via `src/config.rs` e variáveis de ambiente.
- [ ] `AiProvider` em `src/ai.rs` estendido para aceitar o modo local/customizado (ex: `Local` ou endpoint OpenAI-compatible).
- [ ] `AiClient::check_access()` valida conectividade com a URL local configurada antes de enviar prompts.
- [ ] Tratamento adequado de erros de conexão e timeouts quando a instância local não estiver respondendo.
- [ ] Compatibilidade mantida com os prompts de extração e formato de resposta `DecomposedTicket`.

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
## [#148] feat(tickets): suportar comentarios em tickets no cli, divan e sincronizacao
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/148

### Description
## Description
Adicionar suporte a comentários em tickets já existentes nos módulos de tickets e no Divan do CCCP, além de sincronizar os comentários ao baixar os tickets via `sync` (`cccp sync` / `cccp ticket`).

No Divan, ao interagir sobre um ticket existente para gerar ou refinar um comentário via IA, o contexto enviado ao modelo deve incluir a descrição original do ticket e o histórico de comentários já postados.

Na sincronização (`cccp ticket` / `cccp sync`), baixar os comentários existentes de cada issue e incluí-los no `.antigravity/tickets.md`, respeitando o teto de segurança (ceiling) configurado.

## Expected Behavior & Acceptance Criteria
- [ ] **Forge API Client**: Implementar `fetch_issue_comments` e `create_comment` no `ForgeClient` (suportando Codeberg/Forgejo, GitHub e GitLab).
- [ ] **CCCP Ticket Comment CLI**: Adicionar comando para submeter comentário a um ticket existente (`cccp ticket comment <id> "mensagem"`).
- [ ] **Divan Ticket Commenting**: Suportar modo de comentário no Divan direcionado a um ticket existente.
- [ ] **Divan AI Context**: Enviar o corpo do ticket e comentários anteriores como contexto estruturado no prompt para a IA.
- [ ] **Sync com Comentários**: Ao executar `cccp ticket` / `cccp sync`, baixar os comentários das issues e formatá-los no `.antigravity/tickets.md` dentro do limite de segurança.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de main
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
## [#131] feat(cli): posicionar cursor inicial abaixo do cabecalho no editor do divan
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/131

### Description
## Description
Atualmente, ao abrir o editor de texto do Divan, o cursor é posicionado no início do arquivo (linha 1, coluna 1), sobre o cabeçalho explicativo de 4 linhas comentadas. O comportamento desejado é que o cursor seja posicionado automaticamente logo abaixo desse cabeçalho (na linha 5) ao abrir a tela.

## Expected Behavior & Acceptance Criteria
- [ ] Ao inicializar o editor de texto com o cabeçalho padrão de 4 linhas comentadas, o cursor deve iniciar na linha 5 (ou na primeira linha livre após os comentários).
- [ ] Caso o arquivo/template não possua o cabeçalho de 4 linhas, o cursor deve inicializar de forma segura na primeira posição disponível sem causar erros de índice fora de limites (out of bounds).
- [ ] O viewport/scroll deve acompanhar a posição inicial do cursor adequadamente.

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
## [#129] feat(cli): adicionar filtros na listagem para repositórios dirty e com push pendente
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/129

### Description
## Description
Adicionar opções de filtragem na CLI para inspecionar repositórios de acordo com seu estado local de git, permitindo filtrar exclusivamente os repositórios dirty (com alterações locais) ou exclusivamente os limpos com commits pendentes de envio ao remote.

## Expected Behavior & Acceptance Criteria
- [ ] Suportar flag (ex: `--dirty`) para listar unicamente repositórios que possuem modificações staged ou unstaged no working tree.
- [ ] Suportar flag (ex: `--unpushed` ou `--pending-push`) para listar unicamente repositórios limpos com commits locais pendentes de push.
- [ ] Exibir saída estruturada e legível informando contagem de arquivos modificados ou commits ahead correspondentes.
- [ ] Suportar combinação coerente ou validação de exclusão mútua das flags de filtro.

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
