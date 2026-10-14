# Especificação Técnica: Atualização do Cache de Tickets no Repositório Alvo

**Ticket Associado:** #140  
**Branch:** `fix/140-target-repo-ticket-cache`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Garantir que a sincronização e atualização de tickets em `.antigravity/tickets.md` ocorram no diretório raiz do repositório alvo onde as issues foram criadas ou consultadas, e não no diretório de execução atual (`cwd`), tanto no fluxo do `divan` quanto no comando `cccp ticket`.

## 2. Resolução do Repositório Alvo

### 2.1 Fluxo do `divan`
1. Quando o `divan` é executado com um repositório alvo explícito (ex.: `cccp divan outro-repo`) ou quando um repositório é selecionado interativamente no catálogo:
   - O CCCP deve resolver o caminho local do repositório alvo através do catálogo (`find_local_repo_path`) ou por caminho relativo/absoluto em disco.
   - Ao concluir a publicação dos tickets gerados pela IA, a sincronização (`sync_tickets`) deve ser executada no contexto do repositório alvo (`target_repo`), gerando ou atualizando `<target_repo>/.antigravity/tickets.md`.
   - Se o repositório alvo for exclusivamente remoto (sem clone local na máquina), o CCCP **não** deve sobrescrever o `.antigravity/tickets.md` do diretório corrente (`cwd`).

### 2.2 Fluxo do comando `cccp ticket`
1. Quando `cccp ticket` é chamado com a flag `-r` / `--remote <REMOTE>`:
   - Se o remote especificado for diferente do remote do repositório corrente:
     - O CCCP consulta o catálogo e o sistema de arquivos local (`~/src/<repo>`) para verificar se existe um clone local daquele repositório.
     - Se existir clone local, atualiza o `.antigravity/tickets.md` daquele repositório específico.
     - Se não existir clone local e a flag `--output` não tiver sido passada explicitamente, não polui o `.antigravity/tickets.md` do repositório corrente com tickets de terceiros.
2. Quando `cccp ticket` é executado com `-C` / `--dir <DIR>`:
   - O repositório aberto e a escrita de `.antigravity/tickets.md` respeitam estritamente o diretório `<DIR>`.
