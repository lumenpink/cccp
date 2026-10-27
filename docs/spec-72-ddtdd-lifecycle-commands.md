# Especificação Normativa: Subcomandos de Ciclo de Vida DD-TDD no Maestro CCCP

- **Ticket de Referência:** `#72`
- **Norma Base:** GOST v2.0.0 (Seções 5, 6, 7 e 8)
- **Perfil:** Kollontai v2.0.0 / Lilya v2.0.0
- **Fase DD-TDD:** Fase 3 — Doc/Spec (Zero código de produção)
- **Status:** Aprovado para Implementação

---

## 1. Escopo e Propósito

Esta especificação define o contrato de comportamento, argumentos de linha de comando, máquina de estados e validações determinísticas dos subcomandos de gerenciamento de ciclo de vida DD-TDD no Maestro `cccp`:

1. `cccp start <id> [--base <b>]` (Fase 2 — Ramificação Determinística)
2. `cccp commit "<resumo>" [--scope <s>] [--phase <ph>]` (Fases 3 a 8 — Rastreamento com Travas de Fase)
3. `cccp close "<resumo>" [--body <b>]` (Fase 9 — Conclusão com Laudo Textual Obrigatório)
4. `cccp status` (Fases 1 a 9 — Painel de Progresso das Fases)

---

## 2. Contrato do Subcomando `cccp start <id>`

### 2.1 Assinatura da CLI
```text
cccp start [OPTIONS] <ID>

ARGUMENTOS:
  <ID>                  Número identificador do ticket formalizado na forja primária (ex: 72)

OPÇÕES:
  -b, --base <BRANCH>   Branch base para ramificação (padrão: "main", "vN" ou epic ativo)
  -C, --dir <DIR>       Diretório do repositório de trabalho
```

### 2.2 Algoritmo de Execução
1. **Verificação de Working Tree Limpa:**
   - O comando DEVE abortar com erro se houver arquivos modificados ou unstaged no repositório ativo, prevenindo contaminação de branches.
2. **Resolução de Metadados do Ticket:**
   - Inspeciona o cache local de tickets (`.antigravity/tickets.md` ou `.git/cccp/tickets/<id>.json`).
   - Se ausente localmente, consulta a forja primária via API.
   - Extrai o tipo convencional (`feat`, `fix`, `docs`, `refactor`, `chore`, `test`) e o slug do título (normalizado em kebab-case, $\le 40$ chars).
3. **Formatação da Branch de Trabalho:**
   - Padrão canônico: `<tipo>/<id>-<slug>`.
4. **Checkout e Rastreamento:**
   - Se a branch já existir localmente, realiza o `checkout` e exibe o status de fases atual.
   - Se não existir, atualiza a branch base (`git fetch` / `git checkout <base>`) e cria a branch: `git checkout -b <tipo>/<id>-<slug>`.

---

## 3. Contrato do Subcomando `cccp commit`

### 3.1 Assinatura da CLI
```text
cccp commit [OPTIONS] [SUMMARY]

ARGUMENTOS:
  [SUMMARY]             Resumo executivo do commit (<= 72 caracteres)

OPÇÕES:
  -s, --scope <SCOPE>   Escopo opcional do Conventional Commit (ex: "core", "cli")
  -p, --phase <PHASE>   Sobrescrita explícita de fase (ex: "ph3", "ph4", "ph5", "ph6", "ph7", "ph8")
      --no-verify       Bypass de travas locais (proibido para agentes de IA)
```

### 3.2 Dedução Automática de Fase e Ticket
1. **Dedução do Ticket:**
   - O `<id>` é extraído da branch ativa via regex: `^(feat|fix|docs|refactor|test|chore)/([0-9]+)-([a-z0-9-]+)$`.
   - Se a branch ativa for `main` ou branch persistente sem `--phase`, o commit direto é terminantemente bloqueado (Regra GOST-04-B).
2. **Máquina de Estados de Fases:**
   - O histórico de commits da branch em relação à base (`git log <base>..HEAD`) é inspecionado procurando o último marcador `ph[3-8]`:
     - Sem commits na branch $\to$ Próxima fase: **`ph3`** (Doc/Spec).
     - Último commit foi `ph3` $\to$ Próxima fase: **`ph4`** (Red Test).
     - Último commit foi `ph4` $\to$ Próxima fase: **`ph5`** (Green Implement).
     - Último commit foi `ph5` $\to$ Próxima fase: **`ph6`** (Refactor).
     - Último commit foi `ph6` $\to$ Próxima fase: **`ph7`** (Mutation Test).
     - Último commit foi `ph7` $\to$ Próxima fase: **`ph8`** (Quality Gate / Hygiene).
3. **Travas Determinísticas de Fase (Phase Guards):**
   - **Trava de `ph3` (Doc/Spec):**
     - O diff do stage (`git diff --cached --name-only`) NÃO PODE conter arquivos sob `src/`, `lib/`, `tests/`, `bin/`, `R/`. Apenas arquivos em `docs/`, `SPEC.md`, `README.md`, etc., são permitidos.
   - **Trava de `ph4` (Red Test):**
     - O executor nativo de testes da linguagem é disparado em modo restrito. O comando EXIGE falha nos testes para validar o teste vermelho. Se os testes passarem sem falha, o commit é abortado.
   - **Trava de `ph5` (Green Implement):**
     - O executor nativo de testes é disparado. O comando EXIGE que todos os testes passem com sucesso.
   - **Trava de `ph7` (Mutation Test):**
     - Exige que o commit anterior ou arquivos incluam testes antes da refatoração.
4. **Formatação Padronizada da Mensagem:**
   - Formato gerado: `<tipo>(<escopo>): <resumo> (ref #<id> - ph<N>)` ou `<tipo>: <resumo> (ref #<id> - ph<N>)`.

---

## 4. Contrato do Subcomando `cccp close`

### 4.1 Assinatura da CLI
```text
cccp close [OPTIONS] <SUMMARY>

ARGUMENTOS:
  <SUMMARY>             Resumo executivo de fechamento (<= 72 caracteres)

OPÇÕES:
  -b, --body <BODY>     Corpo do Laudo Textual de Resolução
      --editor          Abre o $EDITOR para redação do Laudo
```

### 4.2 Requisitos Obrigatórios do Laudo Textual (Fase 9)
O corpo do commit DEVE conter rigorosamente os 3 cabeçalhos normativos com comprimento total $\ge 80$ caracteres:
```text
<tipo>(<escopo>): <resumo> (closes #<id> - ph9)

[Resumo]:
Descrição clara do problema resolvido e escopo atendido.

[Solução]:
Detalhes técnicos das mudanças implementadas no código.

[Validação]:
Relação de testes automatizados, mutação e análise estática executados.
```

### 4.3 Validações Pré-Commit
1. **Rastreabilidade de Ticket:** O `<id>` do ticket é obtido da branch e conferido com a tag `(closes #<id> - ph9)`.
2. **Atualização de Documentação/Changelog:** Verifica se `CHANGELOG.md` ou `site/` foi modificado nos commits da branch.
3. **Persistência e Fechamento no Merge:** Ao executar `cccp merge`, o Laudo presente no corpo do commit `ph9` é extraído e postado como comentário oficial de encerramento na forja primária.

---

## 5. Contrato Visual do `cccp status`

Quando executado dentro de uma branch de trabalho `<tipo>/<id>-<slug>`, o `cccp status` DEVE renderizar:
```text
★ CCCP DD-TDD Phase Inspector:
  Ticket Ativo : #72 (feat(ddtdd): subcomandos cccp start...)
  Branch       : feat/72-start-commit-close-ddtdd (base: main)
  Progresso    : [1:Ticket ✓] [2:Branch ✓] [3:Spec ✓] [4:Red ...] [5:Green] [6:Refactor] [7:Mutate] [8:Hygiene] [9:Close]
  Fase Atual   : ph4 (Red Test)
  Próximo Passo: cccp commit "adicionar testes falhos de ciclo" (ph4)
```
