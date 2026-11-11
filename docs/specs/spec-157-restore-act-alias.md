# Especificação do Ticket #157: Restaurar Alias `act` para Subcomando `ci`

**Ticket:** #157  
**Fase:** Fase 3 (Spec)  
**Status:** Em Elaboração  
**Autor:** Lumen Pink / Antigravity  

---

## 1. Contexto e Diagnóstico

Durante a refatoração do menu de ajuda principal (`cccp --help`) no Ticket #132, a listagem de variantes do enum `Commands` em `src/cli.rs` foi reorganizada em categorias operacionais. Nessa modificação, a anotação `#[command(alias = "act")]` presente na variante `Ci(CiArgs)` foi acidentalmente omitida.

Como consequência:
- A esteira de CI no Forgejo falhou na execução de `tests/cli_domains_consolidation_test.rs`:
  ```text
  error: unrecognized subcommand 'act'
  ```
- Chamadas retrocompatíveis a `cccp act` deixaram de ser resolvidas para o subcomando canônico de esteira `ci`.

---

## 2. Requisitos Técnicos

1. **Restaurar Alias no Clap:**
   - Adicionar `#[command(alias = "act")]` na variante `Ci(CiArgs)` do enum `Commands` em `src/cli.rs`.
2. **Preservar Categorização no Root Help:**
   - O comando `ci` deve continuar listado na seção `Operações Git & Esteira` do menu raiz.
3. **Validação:**
   - O teste existente em `tests/cli_domains_consolidation_test.rs` (`test_ci_canonical_command`) deve voltar a passar em 100% dos casos.

---

## 3. Árvore de Decisões Arquiteturais (para a Natasha)

1. **Por que manter o alias `act` se o comando canônico é `ci`?**
   - *Decisão:* O comando `cccp act` foi o nome original de execução local da esteira via Act/Docker. Diversos scripts e testes de regressão de múltiplos repositórios ainda dependem desse alias. Quebrar o alias sem transição fere a estabilidade e a compatibilidade operacional da ferramenta.
2. **Por que o alias não polui o root help?**
   - *Decisão:* O alias no Clap não gera uma linha duplicada no menu principal; ele apenas aceita `cccp act` como equivalente transparente de `cccp ci` em tempo de execução.

---

## 4. Plano de Validação
- Fase 4 (Red Test): Já comprovado com a falha em `cargo test --test cli_domains_consolidation_test`.
- Fase 5 (Green Implement): Adicionar o atributo em `src/cli.rs` e reexecutar o teste.
- Fase 7 (Mutation Test): Validar que um mutante sem o alias falha o teste.
