# Especificação Técnica — Ticket #67: Subcomando cccp prompt para Integração de Status do Repositório no Shell

## 1. Visão Geral
O subcomando `cccp prompt` fornece um utilitário nativo em Rust de latência ultrabaixa para integração com o prompt de comandos do usuário (Bash, Zsh, Fish, Starship) via `PS1` ou `PROMPT_COMMAND`.

O comando inspeciona localmente o estado do repositório Git e o estado do ciclo GOST DD-TDD sem realizar chamadas de rede síncronas ou processos pesados, devolvendo uma representação informativa e esteticamente elegante do repositório no terminal.

---

## 2. Requisitos Técnicos

### 2.1 Informações Fornecidas
1. **Branch Atual**:
   - Nome da branch ativa (`main`, `feat/67-...`, `detached@<hash>`).
   - Se o diretório atual não for um repositório git, o comando deve retornar silenciosamente código de saída zero (sem poluir o prompt).
2. **Estado da Working Tree (Clean / Dirty)**:
   - Indicador de alterações não comitadas ou staged (ex: `*`).
3. **Ciclo GOST DD-TDD**:
   - Detecção automática de ticket e fase através da branch e histórico recente (`infer_ticket_from_branch`, `infer_next_phase`), exibindo ex: `[#67 ph3]`.
4. **Contadores e Metadados Locais**:
   - Quantidade de tickets locais abertos a partir de `.antigravity/tickets.md` (ex: `⚑ 10`).
   - Contagem de branches locais e status de CI se em cache no catálogo.

### 2.2 Opções e Modos de Saída da CLI
- `cccp prompt`:
  - Saída padrão formatada e colorida com ANSI.
- `cccp prompt --short` (`-s`):
  - Formato minimalista compacto (ex: `main` ou `feat/67* [#67 ph3]`).
- `cccp prompt --bash`:
  - Envolve sequências de escape ANSI em `\[` e `\]` para prevenir problemas de cálculo de largura de linha e navegação de cursor no Bash.
- `cccp prompt --zsh`:
  - Envolve sequências de escape ANSI em `%{` e `%}` para Zsh.
- `cccp prompt --json`:
  - Retorna objeto JSON com as chaves `{ "branch": "...", "dirty": bool, "ticket": int?, "phase": "..."?, "open_tickets": int }`.
- `cccp prompt --no-color`:
  - Remove qualquer sequência de escape de cor.

### 2.3 Desempenho e Latência
- Tempo de execução estritamente < 15ms em repositórios normais.
- Acesso exclusivo a metadados em disco local (Git HEAD, index, `.antigravity/tickets.md`).
- Nenhuma chamada de rede de qualquer tipo.

---

## 3. Critérios de Aceitação
1. Suporte a todas as flags de formatação (`--short`, `--bash`, `--zsh`, `--json`, `--no-color`).
2. Retorno silencioso e saída limpa com ExitCode 0 fora de repositórios git.
3. Testes automatizados em `tests/prompt_cli_test.rs` cobrindo cenários clean, dirty, branch com ciclo DD-TDD, contagem de tickets e flags de saída.
4. Conformidade total com `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.
