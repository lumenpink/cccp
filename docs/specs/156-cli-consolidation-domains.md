# Especificação Técnica: Consolidação e Padronização da CLI por Domínios

**Ticket Associado:** #156  
**Branch:** `refactor/156-cli-consolidation-domains`  
**Norma:** GOST 1.4.0  

## 1. Objetivo
Revisar e consolidar todos os comandos da CLI (`cccp`), eliminando inconsistências na experiência do usuário e reduzindo radicalmente a poluição do `--help` de 31 para 14 comandos essenciais através de um design orientado a domínios (Domain-Driven CLI).

---

## 2. Requisitos Arquiteturais e Estrutura de Domínios

### 2.1. Domínio `ticket` (`cccp ticket`)
Centraliza todas as operações sobre tickets e demandas:
- `cccp ticket [list]` (ação padrão): Lista de forma tabular/Gosplan os tickets locais sincronizados em `.antigravity/tickets.md` (ID, status, título).
- `cccp ticket new`: Criação direta de ticket na forja remota via linha de comando (`-t, --title`, `-b, --body`, `-l, --label`, `-r, --remote`), sem necessidade de abrir editor interativo (compatível com scripts e agentes automatizados). Caso título/corpo não sejam fornecidos e haja terminal TTY interativo, permite entrada interativa.
- `cccp ticket divan`: Subcomando dedicado para triagem e geração inteligente de tickets assistida por IA (absorvendo a antiga raiz `cccp divan`).
- `cccp ticket sync`: Sincroniza atomicamente os tickets da forja remota para `.antigravity/tickets.md`.
- `cccp ticket close <ids>`: Fecha um ou mais tickets na forja com comentário opcional (`-m, --comment`).
- `cccp ticket comment <id>`: Adiciona comentário ao ticket (`-m, --message` ou buffer).

### 2.2. Domínio `repo` (`cccp repo`)
Centraliza o ciclo de vida e governança de repositórios:
- `cccp repo init`: Inicializa e estrutura novo repositório no padrão estrito GOST (absorvendo o antigo `cccp init`).
- `cccp repo [list]` (ação padrão): Lista os repositórios registrados no catálogo Gosplan com suporte a filtros (`--dirty`, `--unpushed`, `--ci`, `--tickets`) e exibição de esteiras de CI.
- `cccp repo add <repo>`: Cadastra repositório local ou remoto no catálogo.
- `cccp repo rm <repo>`: Remove repositório do catálogo, com suporte opcional a `--delete-dir`.
- `cccp repo status`: Inspeciona o status git, branches e conformidade do repositório ativo.
- `cccp repo sync-tickets`: Atualiza a contagem de tickets dos repositórios do catálogo.
- `cccp repo assemble`: Monta a especificação `.gost/GOST.md` nos repositórios.

### 2.3. Comando `sync` Unificado (`cccp sync`)
Mantido como a única exceção de comando de ação na raiz:
- Executa a sincronização completa e orquestrada de todo o ecossistema:
  1. Tickets da forja (`.antigravity/tickets.md`)
  2. Montagem em cascata da especificação GOST (`.gost/GOST.md`) via `GOSTVER`
  3. Diretivas e variáveis de agentes (`AGENTS.md`)
  4. Catálogo de repositórios do workspace (`~/src`)

### 2.4. Separação de `version` e `gen-version`
- **`cccp version`**:
  - Exibe informativamente as versões ativas do ambiente:
    - Versão instalada do binário CCCP.
    - Versão e perfil ativo do GOST no repositório (`GOSTVER`).
    - Versão atual do projeto (`VERSION` ou tag recente).
- **`cccp gen-version`**:
  - Executa o cálculo preditivo SemVer 2.0.0 a partir do histórico de commits convencionais (anteriormente em `cccp version`), suportando `--bump`, `--json`, etc.

### 2.5. Padronização Canônica de `check` e Limpeza do Root
- **`cccp check`**:
  - Comando canônico para verificação e auditoria de qualidade GOST (`cccp check [--strict]`), eliminando os aliases concorrentes `audit` e `gost`.
- **Comandos Removidos da Raiz**:
  - `editor`, `ci` e `serve`: Extirpados da CLI.
  - `divan`: Integrado em `cccp ticket divan`.
  - `init` e `catalog`: Integrados no domínio `cccp repo`.
  - `set-av`: Integrado em `cccp agent set <var> <valor>`.
  - `check-update`: Integrado em `cccp update --check`.
  - `comrades`: Integrado em `cccp soviet comrades`.

---

## 3. Critérios de Aceitação
1. `cccp --help` exibe a árvore enxuta com cerca de 14 comandos canônicos e sem duplicidades ou comandos obsoletos.
2. `cccp ticket` lista os tickets sincronizados e `cccp ticket new -t "..." -b "..."` cria tickets na forja sem invocar editor interativo.
3. `cccp ticket divan` inicia a triagem assistida por IA.
4. `cccp repo init` realiza o bootstrapping do repositório e `cccp repo list` opera sobre o catálogo.
5. `cccp check --strict` opera como comando canônico de qualidade.
6. `cccp version` exibe as versões instaladas e `cccp gen-version` calcula o SemVer preditivo.
7. Testes automatizados cobrem a nova estrutura de comandos e garantem que aliases obsoletos foram removidos.
