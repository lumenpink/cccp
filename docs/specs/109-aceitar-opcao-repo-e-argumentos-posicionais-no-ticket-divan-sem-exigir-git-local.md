# Especificação Técnica: Ticket #109
## fix(cli): aceitar opção repo e argumentos posicionais no ticket divan sem exigir git local

### 1. Contexto e Problema
Ao executar `cccp ticket divan` (ex.: `cccp ticket divan -r owner/repo "prompt"` ou `cccp ticket divan owner/repo "prompt"`) em um diretório que não seja a raiz ou subdiretório de um repositório Git local, o comando falhava prematuramente com:
`Error: Not a git repository: ...`
Isso acontecia porque o manipulador de `Commands::Ticket` em `src/main.rs` exigia incondicionalmente a abertura de um repositório local (`GitRepo::open`) antes de avaliar a sub-ação (`TicketAction::Divan`), ignorando completamente os parâmetros `--repo` e argumentos posicionais fornecidos pelo operador.

### 2. Requisitos Técnicos
1. **Desacoplamento de `TicketAction::Divan` da Exigência de Git Local (`src/main.rs`)**:
   - No tratamento de `Commands::Ticket(tickets)`, interceptar `TicketAction::Divan` antes da chamada a `GitRepo::open(args.dir.as_deref())`.
   - Delegar a execução diretamente a `run_divan_cmd(divan_args, args.dir.as_deref(), config, theme)`.
2. **Resolução de Repositório e Validação Antecipada (`run_divan_cmd`)**:
   - Manter a resolução de repositório e prompt via `diva.resolve_repo_and_prompt()` (suportando `--repo`, `-r` e primeiro argumento posicional).
   - Caso nenhuma informação de repositório tenha sido fornecida (nem `--repo`, nem argumento posicional, nem seleção de catálogo) e o diretório corrente não for um repositório Git, emitir mensagem de erro clara e amigável:
     `Error: No target repository specified and current directory is not a Git repository.`
     retornando `ExitCode::FAILURE`.
3. **Cobertura de Testes**:
   - Teste automatizado verificando que `Commands::Ticket` com subcomando `Divan` pode ser acionado fora de um repositório git com a opção `-r` ou argumento posicional.
   - Teste de validação para falha clara quando nenhum repositório é informado fora de um git repo.

### 3. Critérios de Aceitação
- `cccp ticket divan -r <owner/repo>` e `cccp ticket divan <owner/repo>` não exigem que o diretório atual seja um repositório Git local.
- Erro claro e antecipado é exibido quando nenhum repositório for especificado fora de um repositório Git.
- Todos os testes de unidade e integração validam o parsing e o despacho do comando.
