# Especificação Técnica — Ticket #78

## Escopo
`fix(dx): gplv3 e ci no init, merge sem emenda ff e chores isentos no check`

## Contexto e Motivação
Identificaram-se três fricções no fluxo diário do CCCP:
1. `cccp init` gerava licença `CC0-1.0` por padrão em vez da soberana `GPL-3.0`, workflow em formato `ci.yaml` sem formatação e com `runs-on: native` em vez de `ubuntu-latest`, e fallback do GOST como `1.1.0`.
2. `cccp merge` sempre gerava um commit extra `chore(version)` na `main`, mesmo em merges *Fast-Forward*, poluindo o histórico e gerando divergências sem necessidade.
3. `cccp check --strict` exigia tags de ticket em commits `chore:`, inviabilizando manutenções rotineiras pequenas sem abrir tickets formais de 9 fases.

## Requisitos
1. **Scaffolding e Init (`src/newrepo.rs` / `src/cli.rs`)**:
   - `DEFAULT_LICENSE` alterado de `CC0-1.0` para `GPL-3.0`.
   - `DEFAULT_GOST_VERSION` padronizado para `2.0.0`.
   - Workflow gerado como `.forgejo/workflows/ci.yml` com `runs-on: ubuntu-latest` e estrutura YAML limpa com indentação canônica de 2 espaços.
2. **Merge Fast-Forward Limpo (`src/main.rs` / `src/hook_runner.rs`)**:
   - Se o merge resultar em Fast-Forward (ou seja, `HEAD` após o merge coincide com o commit da branch integrada), nenhuma reconciliação ou commit de emenda `chore(version)` deve ser criado.
3. **Isenção de Chores na Rastreabilidade (`src/audit.rs`)**:
   - A regra `GOST-04-TICKETS-COMMITS` deve filtrar commits com prefixo `chore:` e `chore(...):`, avaliando a rastreabilidade estrita de tickets apenas nos commits funcionais (`feat`, `fix`, `refactor`, etc.).
