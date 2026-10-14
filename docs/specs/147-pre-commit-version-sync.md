# Especificação Técnica: Sincronização Automática do VERSION no Pre-Commit e Pós-Merge

**Ticket Associado:** #147  
**Branch:** `fix/147-pre-commit-version-sync`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Restaurar e automatizar a sincronização contínua do arquivo `VERSION` com base no grafo Git e no SemVer 2.0.0 preditivo:
1. Durante a fase de **pre-commit** em branches de desenvolvimento, atualizando o arquivo `VERSION` com a versão de desenvolvimento prevista (`dev_version`) e adicionando ao stage (`git add VERSION`).
2. Imediatamente após o **merge** (`cccp merge`), inspecionando o novo HEAD da branch integradora (ex: `main`) e reconciliando o arquivo `VERSION` através de um commit dedicado caso a versão tenha divergido (cenário de branches paralelas), preservando o histórico intacto e gerando o commit extra apenas quando necessário.

## 2. Componentes e Comportamento

### 2.1. Hook Pre-Commit (`run_pre_commit`)
- Disparado pelo Git no ciclo de commit (`.git/hooks/pre-commit` -> `cccp hook pre-commit`).
- Verifica a existência do arquivo `VERSION` na raiz do repositório.
- Se o arquivo existir:
  - Inspeciona o estado do repositório Git via `GitRepo::open`.
  - Calcula a próxima versão de desenvolvimento prevista para o commit em elaboração:
    - O número de commits após a tag base é incrementado em 1 (`commits_count + 1`).
    - Formata a pre-release SemVer: `<target_semver>-dev.<commits_count+1>+<date>.<short_id>`.
  - Lê o conteúdo atual do arquivo `VERSION`:
    - Se for idêntico à nova versão: nenhuma ação necessária.
    - Se for diferente: grava a nova versão no arquivo `VERSION` e executa `git add VERSION` silenciosamente.

### 2.2. Reconciliação Pós-Merge (`reconcile_merge_version`)
- Disparado ao final de `cccp merge <branch>` após a execução bem-sucedida do `git merge`.
- Inspeciona o novo HEAD resultante do merge.
- Recalcula a versão preditiva de desenvolvimento atual da branch (`dev_version`).
- Lê o conteúdo do arquivo `VERSION` no disco:
  - **Cenário 1 (Idêntico):** Em branches sequenciais sem divergência no grafo, o `VERSION` já reflete exatamente o estado do HEAD. Nenhuma ação nem commit adicional é gerado.
  - **Cenário 2 (Divergente):** Em branches paralelas onde múltiplos caminhos convergiram, a distância até a tag anterior saltou. O arquivo `VERSION` é atualizado com o valor real e um commit extra de reconciliação é gerado:
    `chore(version): sincronizar VERSION para <dev_version>`

## 3. Critérios de Aceite
- Repositórios sem o arquivo `VERSION` ignoram a etapa sem falhas.
- O hook `pre-commit` é rápido (< 15ms) e adiciona o `VERSION` ao stage do commit corrente.
- O `cccp merge` só cria o commit `chore(version)` quando há divergência factual entre o arquivo `VERSION` e o cálculo real do novo HEAD.
