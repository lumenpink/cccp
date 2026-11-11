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
3. **Template de Ajuda Estruturado (`ROOT_HELP_TEMPLATE`):**
   - No Clap 4, `subcommand_help_heading` nas variantes de um enum não altera o agrupamento no comando pai, aplicando-se apenas aos subcomandos filhos. Para categorizar a listagem do comando raiz sem acoplamento e com total controle de layout e espaçamento, foi adotado `help_template = ROOT_HELP_TEMPLATE` no struct `Cli`.
   - As descrições no menu raiz foram sintetizadas em 1 linha, enquanto a documentação aprofundada de cada subcomando segue preservada intacta na sua respectiva estrutura.

---

## 4. Status do Ciclo
- **Fase 3:** Especificação formalizada.
- **Fase 4:** Teste vermelho (`test_root_help_organized_into_categories`) atestou ausência de categorias na listagem plana anterior.
- **Fase 5:** Implementação verde com `ROOT_HELP_TEMPLATE` estruturando comandos por categorias lógicas e descrições sucintas.
- **Fase 7:** Mutante eliminado (`test_mutation_subcommand_individual_help_preserved`) validando preservação de opções detalhadas em subcomandos.
- **Fase 8:** Quality Gate aprovado com `cargo fmt` e `cargo clippy --all-targets -- -D warnings`.
- **Fase 9:** Conclusão e encerramento do ticket #132.
