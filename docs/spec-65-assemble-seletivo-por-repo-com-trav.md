# Especificação Técnica — Ticket #65: Assemble Seletivo por Repo com Trava de Working Tree e Automação GOST

## 1. Visão Geral
O subcomando `cccp repo assemble` (e seu alias `cccp profile assemble`) compila a especificação em camadas da norma GOST (`.gost/GOST.md`), unindo a especificação base, os perfis aplicáveis e as customizações do projeto (`CUSTOM.md`). 

Atualmente, o comando em lote atua indiscriminadamente sobre os repositórios do catálogo sem checar se há modificações em andamento (working tree dirty), arriscando sobrepor ou misturar alterações locais. Além disso, não há suporte a execução em repositório único via flag `--repo <slug>`, e os arquivos gerados frequentemente permanecem em estado pendente (uncommitted), sujando o catálogo.

---

## 2. Requisitos Técnicos

### 2.1 Assemble Seletivo por Repositório
- Suportar a flag `--repo <slug>` (ou `-r <slug>`) tanto em `cccp repo assemble` quanto em `cccp profile assemble`.
- Quando informada a flag `--repo <slug>`:
  - Localizar a entrada no catálogo por slug, nome ou alias.
  - Abortar com erro caso o repositório não seja encontrado no catálogo.
  - Executar o assemble exclusivamente no workspace desse repositório.

### 2.2 Trava de Working Tree Limpa
- Antes de iniciar o assemble em qualquer repositório (individual ou no catálogo):
  - Inspecionar se a working tree está dirty (arquivos modificados, staged ou untracked conflitantes).
  - Caso haja modificações não salvas, abortar a operação para aquele repositório com mensagem explícita:
    `"Repositório possui modificações pendentes na working tree. Faça commit ou stash antes do assemble."`
  - No modo lote, repositórios dirty devem ser registrados como falha/ignorado com aviso claro, impedindo corrupção de trabalho local.

### 2.3 Automação de Ciclo GOST (Auto-Commit)
- Adicionar a flag `--auto-commit` (e opcionalmente `--push`):
  - Após montar `.gost/GOST.md`, se o arquivo tiver sofrido alteração de conteúdo em relação ao HEAD:
    - Adicionar `.gost/GOST.md` ao index git.
    - Criar commit canônico com mensagem padronizada:
      `docs(gost): atualizar especificacao gost compilada`
    - Utilizar o ambiente do Maestro (`CCCP_MAESTRO_ACTIVE=1`) para validação dos hooks.
    - Se a flag `--push` estiver ativa, despachar o envio seguro garantindo paridade com o tracking branch.

---

## 3. Matriz de Testes (Fases 4 & 7)
- `tests/repo_assemble_test.rs`:
  1. Teste de assemble seletivo com `--repo <slug>` em repositório único válido.
  2. Teste de rejeição/aborto quando repositório está com a working tree dirty.
  3. Teste de rejeição quando `--repo <slug>` não existe no catálogo.
  4. Teste de auto-commit após assemble quando `--auto-commit` está ativo e há mudanças.
  5. Teste de idempotência (nenhum commit quando o arquivo `.gost/GOST.md` já está em sincronia).
