# Especificação Técnica — Ticket #68: Autocompletion Dinâmico e Instalador no Shell

## 1. Visão Geral
O CCCP dispõe do comando `cccp completion <shell>` para emissão de scripts de completude sintática estática via `clap_complete`.
Entretanto, subcomandos que operam sobre repositórios do ecossistema Gosplan (como `cccp divan`, `cccp repo assemble`, `cccp tickets`) necessitam de completude contextual dinâmica dos slugs, nomes e apelidos cadastrados no catálogo local (`catalog.json`).
Além disso, a ativação do autocompletion exige que o usuário adicione comandos manuais ao seu arquivo de inicialização de shell (`.bashrc` ou `.zshrc`).

Esta especificação define o gerador de candidatos dinâmicos do catálogo e o instalador automático e idempotente de autocompletion.

---

## 2. Requisitos Técnicos

### 2.1 Emissão de Candidatos Dinâmicos do Catálogo
- Suportar a flag `cccp completion --candidates repos` (ou subcomando equivalente):
  - Carrega o catálogo Gosplan através de `load_catalog`.
  - Emite no `stdout`, linha a linha, todos os slugs (`owner/repo`), nomes e apelidos ativos.
  - Execução ultra-rápida e resiliente (ignora erros de catálogo inexistente retornando vazio).

### 2.2 Enriquecimento do Script de Autocompletion
- Ao emitir o script de completude para Bash (`cccp completion bash`):
  - Embutir a função auxiliar de busca dinâmica `_cccp_repo_candidates()`, que consulta `cccp completion --candidates repos`.
  - Vincular os argumentos de repositório (ex.: posicional do `divan`, flag `--repo`, e argumentos de `repo assemble`) à lista dinâmica de repositórios do catálogo.

### 2.3 Subcomando de Instalação Automática (`cccp completion install`)
- Suportar a sintaxe:
  - `cccp completion install --bash` (ou `cccp completion install bash`)
  - `cccp completion install --zsh` (ou `cccp completion install zsh`)
- Comportamento do instalador:
  - Identificar o arquivo RC de destino (`~/.bashrc` para Bash, `~/.zshrc` para Zsh), ou caminho customizado via `--path <file>`.
  - Injetar bloco delimitado por marcadores canônicos Gosplan:
    ```bash
    # >>> CCCP SHELL COMPLETION >>>
    source <(cccp completion bash)
    # <<< CCCP SHELL COMPLETION <<<
    ```
  - **Idempotência estrita**: se os marcadores já existirem no arquivo, atualizar o bloco existente sem duplicar linhas.
  - Suportar flag `--dry-run` para inspecionar a modificação sem alterar o disco.
  - Exibir mensagem de sucesso instruindo o usuário sobre como recarregar o shell (`source ~/.bashrc`).

---

## 3. Critérios de Aceitação
1. `cccp completion --candidates repos` lista corretamente slugs e aliases de um catálogo existente.
2. `cccp completion install --bash` (e com `--path`) insere o bloco canônico de forma idempotente em arquivo `.bashrc`.
3. Execuções repetidas do instalador não duplicam linhas nem corrompem o arquivo.
4. Suporte a `--dry-run`.
5. Bateria de testes automatizados em `tests/completion_install_test.rs`.
6. Conformidade com `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.
