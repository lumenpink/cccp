# Active Tickets - cccp
> Synced at: 2026-10-14 16:07:49 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

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
## [#134] feat(cli): implementar recuperação de estado de sessão após falhas na geração e envio de tickets
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/134

### Description
## Description

Atualmente, se ocorrer qualquer falha durante a execução do Divã (erro na API do Gemini, timeout, falha de rede ao submeter à forge ou encerramento inesperado do processo), todo o progresso do usuário é perdido. Faz-se necessário implementar um mecanismo automático de checkpoint (estilo swap do Vim) indexado por repositório, garantindo que o último estado com sucesso seja salvo e oferecido para recuperação na próxima inicialização.

## Expected Behavior & Acceptance Criteria
- [ ] Salvar automaticamente checkpoints de estado vinculados ao repositório alvo antes e após chamadas de rede (envio ao Gemini e envio à Forge).
- [ ] Persistir tickets retornados pelo Gemini que ainda estão pendentes de aprovação pelo usuário.
- [ ] Ao inicializar o Divã para um repositório com estado não finalizado, exibir prompt perguntando se o usuário deseja restaurar a sessão interrompida.
- [ ] Ao confirmar a restauração, retomar exatamente do ponto da interrupção (ex.: pular nova chamada ao Gemini se os tickets já haviam sido gerados).
- [ ] Limpar o checkpoint automático após a conclusão bem-sucedida do envio à Forge.

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
## [#132] feat(cli): implementar quebra automatica de linha padrao e atalho de alternancia no editor
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/132

### Description
## Description
O editor de texto do Divan necessita de suporte a quebra automática de linha (soft wrap) ativada por padrão, facilitando a leitura e edição de textos longos no terminal sem rolagem horizontal indesejada, além de fornecer um atalho de teclado para alternar (toggle) o recurso sob demanda.

## Expected Behavior & Acceptance Criteria
- [ ] O editor deve abrir com a quebra automática de linha (word wrap / soft wrap) ativada por padrão.
- [ ] Um atalho de teclado dedicado deve ser disponibilizado para alternar o modo de quebra de linha (ligado/desligado).
- [ ] A interface/barra de status do editor deve indicar claramente o estado atual da quebra de linha e informar o atalho disponível.
- [ ] A navegação do cursor vertical e horizontal deve respeitar corretamente as linhas lógicas e visuais quando o wrap estiver habilitado ou desabilitado.

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
