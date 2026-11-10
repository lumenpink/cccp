# Especificação do Ticket #131: Atalhos Ocultos no Nível Raiz para Subcomandos com Identificador Exclusivo

## Objetivo
Implementar na interface CLI do Maestro CCCP atalhos diretos no nível raiz para subcomandos dotados de identificador semântico exclusivo (com foco no `cccp divan` como atalho para `cccp ticket divan`), mantendo tais atalhos ocultos (`hide = true`) no menu principal `cccp --help` para prevenir a poluição visual da listagem de comandos centrais.

---

## 1. Contexto e Requisitos

1. **Invocação Direta na Raiz:**
   - O comando `cccp divan [args...]` deve ser aceito e roteado diretamente para a mesma rotina de execução de `cccp ticket divan [args...]`.
2. **Ocultação Visual no Menu Principal:**
   - No enum `Commands`, o subcomando `Divan` deve conter a anotação `#[command(hide = true)]`.
   - O comando `cccp --help` NÃO deve exibir `divan` na lista pública de comandos disponíveis, mantendo a interface enxuta.
3. **Visibilidade Contextual no Help do Comando Pai:**
   - O comando pai `cccp ticket --help` e a ajuda do próprio `cccp divan --help` devem indicar explicitamente a disponibilidade do atalho de raiz `cccp divan`.

---

## 2. Árvore de Decisões e Registro Arquitetural (para a Natasha)

1. **Por que ocultar da raiz em vez de expor como comando de primeira classe?**
   - *Contexto:* O `cccp --help` já possui comandos vitais de governança (start, commit, close, hygiene, check, audit, sync, repo, push). Expor cada utilitário temático na raiz criaria sobrecarga cognitiva.
   - *Decisão:* Subcomandos especializados residem logicamente sob seus domínios (`ticket divan`), mas têm atalhos ocultos na raiz para máxima agilidade e ergonomia do operador frequente.
2. **Compatibilidade Total de Argumentos:**
   - O atalho reutiliza exatamente a mesma estrutura `DivanArgs` de `TicketAction::Divan`.

---

## 3. Status do Ciclo
- **Fase 3:** Especificação formalizada.
