# Especificação do Ticket #132: Compactação e Organização do Menu de Ajuda Principal (`cccp --help`)

## Objetivo
Refatorar a apresentação do menu de ajuda principal (`cccp --help`), compactando descrições prolixas e estruturando os subcomandos por agrupamentos lógicos de fácil escaneamento visual, garantindo que os comandos individuais mantenham suas explicações ricas e completas quando consultados diretamente (`cccp <cmd> --help`).

---

## 1. Contexto e Diagnóstico
O `cccp --help` acumulou dezenas de comandos em uma única lista plana desestruturada, com descrições longas que empurram os argumentos e opções para fora da primeira rolagem do terminal.
Além disso, comandos operacionais que estendem rotinas usuais do Git (como `commit`, `merge`, `push`) devem ser sumarizados com objetividade, remetendo seus detalhes avançados para seus respectivos subcomandos.

---

## 2. Requisitos Técnicos

1. **Separação Categórica de Comandos (`subcommand_help_heading`):**
   - **Ciclo DD-TDD & Governança:** `start`, `commit`, `close`, `mutate`, `hygiene`, `check`, `audit`
   - **Operações Git & Esteira:** `merge`, `push`, `tag`, `hook`, `status`, `ci`
   - **Gestão de Repositórios & Catálogo:** `repo`, `sync`, `ticket`, `epic`, `inbox`, `deps`, `agent`
   - **Ferramental & Utilitários:** `version`, `gen-version`, `changelog`, `config`, `profile`, `site`, `update`, `completion`, `prompt`, `cockpit`, `spool`, `soviet`, `forge`
2. **Compactação de Descrições:**
   - Reduzir textos redundantes no menu raiz para títulos sucintos de 1 linha.
   - Preservar o detalhamento integral em `long_about` ou na docstring do subcomando correspondente.
3. **Template de Ajuda Enxuto:**
   - Adicionar `help_template` no comando raiz para formatar seções limpas sem espaçamento vertical excessivo entre comandos.

---

## 3. Árvore de Decisões e Registro Arquitetural (para a Natasha)

1. **Por que agrupar por headings em vez de esconder comandos?**
   - *Contexto:* Comandos importantes não devem ser escondidos se o desenvolvedor precisa saber que eles existem.
   - *Decisão:* Usar headings (`subcommand_help_heading`) organiza visualmente a tela por intenção (TDD, Git, Catálogo, Utils), reduzindo drasticamente o esforço cognitivo de busca sem perder a descoberta de recursos.
2. **Preservação de Subcomandos:**
   - `cccp commit --help`, `cccp push --help`, etc., permanecem intocados com seus exemplos e documentação aprofundada.

---

## 4. Status do Ciclo
- **Fase 3:** Especificação formalizada.
