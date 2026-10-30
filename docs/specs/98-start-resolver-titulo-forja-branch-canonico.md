# Especificação Técnica: Resolução Canônica de Branch e Consulta à Forja no CCCP Start

**Ticket Associado:** #98  
**Branch:** `fix/98-start-resolver-titulo-forja-branch-canonico`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Garantir que o comando `cccp start <id>` nunca crie branches genéricas sem ticket (`feat/<id>-ticket-<id>`), consulte obrigatoriamente a forja remota caso o ticket não esteja no cache local `.antigravity/tickets.md`, normalize caracteres acentuados da língua portuguesa via transliteração e unifique a nomenclatura de branches sob o padrão canônico `<tipo>/<id>-<slug>` tanto no `cccp ticket new` quanto no `cccp start`.

## 2. Requisitos Técnicos
1. **Função SSOT `canonical_branch_name(issue_num: u64, title: &str) -> String`**:
   - Extrai o tipo convencional do início do título (`feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `style`, `perf`, `build`, `ci`).
   - Normaliza caracteres acentuados (`á`->`a`, `é`->`e`, `í`->`i`, `ó`->`o`, `ú`->`u`, `ç`->`c`, etc.).
   - Converte para kebab-case com limite prudente de caracteres.
   - Retorna no formato `<tipo>/<id>-<slug>`.
2. **Unificação no `cccp ticket new`**:
   - Substituir `suggest_branch_name` pela função canônica `canonical_branch_name`, exibindo a sugestão no mesmo padrão do start.
3. **Resolução Remota e Proibição de Branch Genérica no `cccp start`**:
   - Inspecionar `.antigravity/tickets.md`. Se não encontrar o ticket, consultar a forja remota via `ForgeClient::fetch_single_issue`.
   - Se o ticket não existir na forja remota nem no cache local, abortar a execução com código de erro e mensagem explicativa:
     `Erro GOST (Fase 2): Ticket #<id> não encontrado na forja nem no cache local. Crie o ticket antes de iniciar o ciclo com 'cccp ticket new'.`
   - É terminantemente proibido criar fallback genérico `feat/<id>-ticket-<id>`.

## 3. Critérios de Aceitação
- Testes unitários comprovando que `canonical_branch_name(3, "docs: ajustar conteúdo do site")` gera `docs/3-ajustar-conteudo-do-site`.
- Teste unitário comprovando que `cccp start` falha com erro ao receber um ID inexistente.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou erros.

## 4. Status de Resolução e Validação
- **Fase 5 (Green):** Implementada transliteração com `transliterate_slug`, `canonical_branch_name` SSOT e integração remota `fetch_single_issue` no `ForgeClient`.
- **Fase 7 (Mutation):** Testes de mutação cobrindo todas as vogais acentuadas maiúsculas e minúsculas, cedilhas, símbolos e fallback de slug vazio.
- **Fase 8 (Quality Gate):** Validado com `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --test ddtdd_branch_resolution_test` (6 testes aprovados) e `cccp check --strict` com conformidade Gosplan de 100%.
- **Fase 9 (Close):** Ciclo finalizado com laudo de encerramento e conformidade plena com diretrizes GOST.
