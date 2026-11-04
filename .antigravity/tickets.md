# Active Tickets - cccp
> Synced at: 2026-11-04 00:24:32 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#138] feat(repo-top): espelhar lista de repositorios recentes e status real de ci da forja
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/138

### Description
[Resumo]: Fazer o 'cccp repo top' espelhar a lista de repositorios da pagina inicial da forja por atividade recente com o status real da esteira (CI) na direita.
[Contexto]: O monitor estava desacoplado ao ler o catalogo estatico local e priorizar jobs pendentes velhos, exibindo status desatualizado em vez dos repositorios recentemente atualizados com o commit status real.
[Objetivo]:
1. Buscar repositorios ordenados por atividade recente da forja.
2. Resolver o commit status real do branch padrao na coluna direita.
3. Saneamento do catalogo contra estados pendentes fantasmas.

---
## [#137] fix(push): permitir envio de tags e repasse de git_args quando repo estiver sincronizado
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/137

### Description
### Descrição do Problema
No comando 'cccp push', a função 'check_and_reconcile_parity' verifica apenas a contagem de commits à frente ('ahead') e atrás ('behind') no tracking branch remoto de HEAD. Quando 'ahead == 0', a função retorna 'ParityAction::AlreadyUpToDate' e 'handle_push' encerra antecipadamente com sucesso exibindo 'Tudo sincronizado com o remote (nada a enviar).'.

Isso impede:
1. O envio de tags anotadas criadas via 'cccp tag' (ex.: 'cccp push origin v1.0.0' ou 'cccp push origin --tags'), pois argumentos passados em 'push.git_args' são ignorados quando a branch não possui novos commits.
2. O próprio comando 'cccp tag' instrui a rodar 'git push origin main && git push origin <tag>', mas o hook pre-push bloqueia a execução direta sem CCCP_MAESTRO_ACTIVE.

### Comportamento Esperado
- Se '!push.git_args.is_empty()', o 'cccp push' não deve abortar com AlreadyUpToDate, mas sim repassar os argumentos para 'git push' sob a guarda do Maestro.
- Opcionalmente verificar se existem tags locais pendentes de envio ao rodar 'cccp push'.

### Arquivo Afetado
- 'src/push.rs' (linhas 201-208)

---
## [#135] fix(catalog): resolver fqdn completo para hosts de forjas customizadas
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/135

### Description
[Resumo]: Garantir resolucao de FQDN (gambi.2lp.in) para forjas customizadas no catalogo e enriquecimento de entradas.

---
## [#132] refactor(cli): compactar e organizar exibição do menu de ajuda principal
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/132

### Description
## Description
O help principal do binário (`cccp --help`) tornou-se extenso e difícil de ler rapidamente. É necessário compactá-lo, limpando descrições verbosas e consolidando os comandos operacionais herdados diretamente do Git (`commit`, `merge`, `push`) em uma seção ou linha condensada explicativa, orientando o usuário a consultar `<comando> --help` para mais detalhes.

## Expected Behavior & Acceptance Criteria
- [ ] Estruturação do template de help raiz com seções claras e compactas.
- [ ] Comandos herdados do Git (`commit`, `merge`, `push`) apresentados de forma sucinta com indicação de contrapartes Git.
- [ ] Redução significativa do número de linhas visíveis no `cccp --help` sem perda de usabilidade.
- [ ] Ajuda individual dos comandos permanece intacta e rica em detalhes (`cccp commit --help`, etc.).

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
## [#131] feat(cli): criar atalhos ocultos no nível raiz para subcomandos com identificador exclusivo
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/131

### Description
## Description
Subcomandos com identificador não conflitante (exemplo: `cccp ticket divan`) devem poder ser invocados diretamente na raiz (exemplo: `cccp divan`). Para não poluir o help global da CLI, esses atalhos na raiz não devem ser exibidos no `cccp --help` (usando flags de ocultação do clap). Devem, contudo, constar explicitamente no help do comando pai (ex: `cccp ticket --help`) e na ajuda do próprio subcomando.

## Expected Behavior & Acceptance Criteria
- [ ] Permitir invocação de subcomandos exclusivos diretamente pela raiz (ex: `cccp divan`).
- [ ] Atalhos de nível raiz marcados como ocultos (`hide = true`) no menu `cccp --help`.
- [ ] O help do comando pai contextualiza que o comando possui atalho direto na raiz (ex: em `cccp ticket --help`).
- [ ] `cccp divan --help` ou `cccp ticket divan --help` exibem a documentação completa e menção ao atalho correspondente.

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
## [#130] feat(cli): adicionar flag --force para ignorar validações nos comandos operacionais
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/130

### Description
## Description
Todos os comandos que executam validações de conformidade de fluxo e qualidade (tais como `commit`, `push`, `merge`, entre outros) devem dispor da flag `--force`. Quando fornecida, a operação deve ser executada ignorando as validações automáticas do CCCP.

## Expected Behavior & Acceptance Criteria
- [ ] Adicionar flag `--force` (e `-f` se compatível) aos comandos `commit`, `push`, `merge` e correlatos.
- [ ] Quando `--force` for passado, pular verificações de conformidade prévias e prosseguir com a operação git subjacente.
- [ ] Registrar aviso visual no terminal informando que as validações foram ignoradas sob requisição explícita.
- [ ] Manter validações obrigatórias ativas por padrão na ausência da flag.

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
## [#129] feat(core): introduzir engine extensível para validação dinâmica de normas no cccp check
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/129

### Description
## Description
É necessário permitir que o `cccp check` avalie novas normas e regras dinamicamente sem acoplamento rígido ao binário. Deve-se avaliar e implementar uma camada de extensibilidade desacoplada (por exemplo, via engine de scripting integrada como Lua ou executores externos baseados no schema do `rules.json`) para que projetos definam e validem suas regras de conformidade sob demanda.

## Expected Behavior & Acceptance Criteria
- [ ] Definir e implementar mecanismo desacoplado para execução de verificações do `cccp check` com base nos artefatos `rules.json` e scripts de validação.
- [ ] Possibilitar inclusão de novas regras sem necessidade de alteração ou rebuild do binário CCCP.
- [ ] `cccp check` deve carregar e reportar violações baseadas nas regras das normas presentes na cadeia ativa do repositório.
- [ ] Tratamento gracioso de erros caso uma regra externa falhe ou apresente formato inválido.

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
