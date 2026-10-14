# Especificação Técnica: Push em Lote para Repositórios Limpos

**Ticket Associado:** #128  
**Branch:** `feat/128-batch-push-clean-repos`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Implementar funcionalidade de push em lote para repositórios locais gerenciados no catálogo do CCCP que possuam commits pendentes de envio ao remote (`ahead > 0`), garantindo que repositórios com alterações não commitadas (`dirty`) sejam rigorosamente ignorados por motivos de segurança.

## 2. Interface CLI
- Subcomando ou flag: `cccp push [--all|-a] [--dry-run|-n]` e/ou `cccp catalog push [--dry-run|-n]`.
- Opções:
  - `-n, --dry-run`: Pré-visualiza os repositórios elegíveis para push sem executar nenhuma operação de rede.
  - `-a, --all`: Dispara a varredura sobre todos os repositórios registrados no catálogo.

## 3. Critérios Estritos de Elegibilidade
Para que um repositório seja elegível ao push automático em lote, ele DEVE satisfazer todas as condições abaixo:
1. **Clone Local Existente:** O diretório local do repositório deve existir no disco.
2. **Estado Limpo (`clean`):** `git status --porcelain` deve estar 100% vazio (sem arquivos modificados, staged ou untracked).
   - Se `dirty`: o repositório é ignorado e um aviso de segurança é exibido.
3. **Commits Pendentes (`ahead > 0`):** O branch local deve possuir 1 ou mais commits à frente do branch upstream rastreado (`@{u}..HEAD`).
   - Se `ahead == 0`: o repositório está atualizado e é ignorado.

## 4. Execução e Relatório Construtivista
- Execução isolada em cada diretório de repositório elegível via `git push`.
- Exibição de sumário consolidado ao final:
  - Repositórios enviados com sucesso.
  - Repositórios ignorados por estarem sujos (`dirty`).
  - Repositórios ignorados por já estarem atualizados.
  - Repositórios com falha de envio (rede/permissão).
