# Especificação Técnica: Ticket #124
## feat(ci): configurar cancelamento de esteiras redundantes cancel-in-progress

### 1. Contexto e Problema
Em instâncias Forgejo com capacidade de execução serial (runner único com concorrência = 1), múltiplos pushes sucessivos na mesma branch enfileiram sucessivas esteiras completas.
Quando a configuração de concorrência não cancela esteiras em andamento ou obsoletas (`cancel-in-progress: false` ou ausente), dezenas de jobs acumulam-se na fila por horas ou dias, executando código já superado e impedindo que versões mais recentes sejam validadas prontamente.

Para assegurar eficiência operacional e evitar reincidência:
1. O workflow de CI do próprio CCCP (`.forgejo/workflows/ci.yml`) deve possuir `cancel-in-progress: true`.
2. O gerador de novos projetos GOST (`cccp init` em `src/newrepo.rs`) deve gerar workflows de CI contendo o bloco `concurrency` com `group: ${{ github.workflow }}-${{ github.ref }}` e `cancel-in-progress: true` por padrão.

### 2. Requisitos Técnicos
1. **Configuração de CI do Repositório (`.forgejo/workflows/ci.yml`)**:
   - Atualizar o bloco `concurrency`:
     ```yaml
     concurrency:
       group: ${{ github.workflow }}-${{ github.ref }}
       cancel-in-progress: true
     ```
2. **Molde Padrão de CI para Novos Repositórios (`src/newrepo.rs`)**:
   - Na função `create_project`, a variável `ci_workflow_content` deve incluir a seção `concurrency`:
     ```yaml
     name: CI

     on:
       push:
         branches: [main]
       pull_request:

     concurrency:
       group: ${{ github.workflow }}-${{ github.ref }}
       cancel-in-progress: true

     jobs:
       check:
         runs-on: ubuntu-latest
         steps:
           - uses: actions/checkout@v4
           - name: GOST Compliance Audit
             run: |
               if command -v cccp >/dev/null 2>&1; then
                 cccp check --strict
               fi
     ```
3. **Verificação e Cobertura de Testes (`tests/newrepo_test.rs`)**:
   - Assegurar testes unitários verificando a presença de `concurrency:` e `cancel-in-progress: true` no `.forgejo/workflows/ci.yml` gerado por `cccp init`.

### 3. Critérios de Aceitação
- `.forgejo/workflows/ci.yml` possui `cancel-in-progress: true`.
- Repositórios gerados via `create_project` possuem `cancel-in-progress: true` no seu workflow de CI.
- A suíte de testes unitários valida a presença das diretivas de concorrência no arquivo gerado.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou erros.
