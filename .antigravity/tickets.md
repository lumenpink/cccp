# Active Tickets - cccp
> Synced at: 2026-10-14 22:31:01 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#149] feat(api): suportar instancia local de IA para o Divan
- **Status**: `closed`
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
## [#147] fix(hook): sincronizar arquivo VERSION automaticamente no pre-commit
- **Status**: `closed`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/147

### Description
### Descrição
Durante a migração do CCCP para Rust, a atualização automática do arquivo `VERSION` foi perdida. Originalmente, o arquivo `VERSION` era atualizado dinamicamente na fase de `pre-commit` com a versão calculada pelo SemVer 2.0.0 preditivo (ex.: `0.4.0-dev.46+20261014.7c0b89e`), sendo colocado em *stage* (`git add VERSION`) para ser incluído no próprio commit.

Atualmente, o `cccp version` calcula a versão corretamente via AST/grafo do Git, mas o arquivo `VERSION` em disco fica estagnado a menos que o usuário execute manualmente `cccp version -f dev -w`.

### Comportamento Esperado & Critérios de Aceite
1. O hook `pre-commit` (ou o despachador de hooks do CCCP) deve recalcular a versão com base nos commits e tags existentes e atualizar o arquivo `VERSION`.
2. Se o arquivo `VERSION` for atualizado durante o `pre-commit`, deve adicioná-lo ao index do Git (`git add VERSION`) de modo limpo e transparente.
3. Respeitar configurações de formato ou flags para repositórios com ou sem arquivo `VERSION` sob padrão GOST (GOST-03-VERSION-SEMVER).
4. Cobertura de testes unitários e de integração para a rotina de pre-commit e sincronização do `VERSION`.

### Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `fix/147-pre-commit-version-sync` criada
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação da sincronização no pre-commit
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional do sync no hook pre-commit
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter `-D warnings` e `cargo fmt`
- [ ] Fase 9 - Close: Changelog atualizado e commit com `(closes #<id>)`

---
## [#146] feat(catalog): exibir coluna de versao gost na listagem de repositorios do catalogo
- **Status**: `closed`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/146

### Description
## Description
A listagem do catálogo (`cccp catalog list`) exibe informações sobre Forge, Tickets, Status Git e Caminho Local. No entanto, não apresenta o status de governança GOST de cada repositório (ex.: versão ativa em `GOSTVER`, norma vigente ou status de conformidade). Adicionar uma coluna GOST permite identificar instantaneamente repositórios defasados ou sem governança ativa.

## Expected Behavior & Acceptance Criteria
- [ ] Incluir coluna `GOST` na tabela renderizada por `cccp catalog list`.
- [ ] Exibir a versão declarada em `GOSTVER` (ex.: `1.4.0`) para repositórios locais com GOST ativo.
- [ ] Exibir `-` ou indicador visual de ausência para repositórios sem `GOSTVER`.
- [ ] Suporte a ordenação por versão GOST via flag `--sort gost`.
- [ ] Testes unitários para renderização de colunas e extração de versão no catálogo.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch <tipo>/<id>-<slug> criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#145] feat(catalog): executar montagem de especificacoes gost em lote para repositorios do catalogo
- **Status**: `closed`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/145

### Description
## Description
Atualmente, a montagem da especificação GOST (`.gost/GOST.md`) via cascata de perfis (`cccp profile assemble`) é executada apenas no repositório corrente (`cwd`). Para manter múltiplos repositórios do catálogo sincronizados e em conformidade técnica após atualizações de normas ou perfis, é necessário suportar a execução de assemble em lote (ex.: `cccp catalog assemble` ou `cccp profile assemble --all`).

## Expected Behavior & Acceptance Criteria
- [ ] Implementar flag ou comando para montar `.gost/GOST.md` em lote para todos os repositórios locais registrados no catálogo.
- [ ] Ignorar repositórios sem clone local ou que não possuam arquivo `GOSTVER`.
- [ ] Exibir relatório consolidado de sucesso/falha por repositório com estética construtivista.
- [ ] Testes unitários para execução em lote e isolamento entre repositórios.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch <tipo>/<id>-<slug> criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
