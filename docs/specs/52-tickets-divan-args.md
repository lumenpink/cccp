# Especificação Técnica: Parsing Robusto de Argumentos e Destino no Tickets Divan

**Ticket Associado:** #52  
**Branch:** `fix/52-tickets-divan-args`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Garantir que a invocação do Divã aceite parâmetros posicionais (`<repo>`, `owner/repo` ou descrição do problema) tanto após `cccp ticket divan` quanto através do plural canônico `cccp tickets divan` e atalhos diretos, aceitando flags adicionais (`-r`, `--repo`, `-p`, `-m`, `--dry-run`, `-c`, etc.) intercaladas sem erros de parsing pelo Clap, e resolvendo alvos `owner/repo` mesmo que o repositório não esteja pré-cadastrado no catálogo local.

## 2. Requisitos Funcionais e Critérios de Aceitação

1. **Aliases de Comandos no Clap (`src/cli.rs`):**
   - O comando `Commands::Ticket` deve possuir o alias `tickets`, permitindo invocações como `cccp tickets divan ...`.
   - `Commands` deve possuir o atalho direto `Divan(DivanArgs)` (com aliases `diva`), permitindo invocações diretas como `cccp divan <repo>`.

2. **Parsing de Argumentos Posicionais e Opções no DivanArgs (`src/cli.rs`):**
   - Declarar explicitamente os argumentos posicionais:
     - `target`: `Option<String>` (index = 1) com doc e help descritivo.
     - `prompt`: `Vec<String>` ou `Option<String>` (trailing_var_arg = true ou index = 2) de modo a permitir frases livres e opções intercaladas.
   - Suportar precedência clara em `resolve_repo_and_prompt()`:
     - Se `--repo <R>` ou `-r <R>` for passado, ele define o repositório-alvo soberano, e quaisquer argumentos posicionais são acumulados como `prompt`.
     - Se apenas argumentos posicionais forem fornecidos:
       - Se o primeiro argumento contiver formato `owner/repo` ou coincidir com nome de repositório/slug, é tratado como `target_repo` e o restante como `prompt`.
       - Se for uma frase com espaços e nenhum repo for especificado, é tratado como `prompt`.

3. **Resolução de Destino Desconhecido ou Remoto (`src/catalog.rs`):**
   - Se `input` contiver uma barra (`owner/repo`), e não constar no `catalog.json` local nem em diretórios locais:
     - Deve resolver dinamicamente criando `RepoForgeInfo` apontando para a forge padrão configurada (ex: `git.2lp.in`, `codeberg.org` ou a definida nas configurações), com provider apropriado (`forgejo` / `gitea` / `github`), permitindo triar e despachar sem erro de `Target repository not found`.

4. **Zero Erros de Parsing e Mensagens Informativas:**
   - Sintaxes válidas:
     - `cccp tickets divan meurepo`
     - `cccp tickets divan owner/meurepo`
     - `cccp ticket divan owner/meurepo "descrição do problema"`
     - `cccp ticket divan -r owner/meurepo --dry-run "descrição"`
     - `cccp divan owner/meurepo`
