# Especificação Técnica: Pausa Automática do Forgejo Runner Durante CI Local (`act`)

**Ticket Associado:** #144  
**Branch:** `feat/144-pause-runner-during-act`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Evitar contenção de recursos de hardware (CPU, barramento de memória e concorrência no Docker daemon) durante a execução de testes locais de CI via `act` (tanto no hook `pre-push` quanto no comando `cccp act run`), congelando temporariamente o container do `forgejo-runner` via `docker pause` e restaurando-o via `docker unpause` ao final da execução.

## 2. Escopo & Mecanismos
1. **Detecção:**
   - O CCCP verifica se o Docker está acessível.
   - Consulta containers ativos cujo nome seja `forgejo-runner` (ou containers com label/nome correspondente).
   - Se a variável de ambiente `CCCP_NO_PAUSE_RUNNER=1` estiver definida, pula a pausa.
2. **Ciclo de Vida RAII (`RunnerPauseGuard`):**
   - Criação do guard: Executa `docker pause <container>` nos containers encontrados e exibe feedback visual informativo via stdout/stderr (ex: `[pre-push] ⏸ Pausando forgejo-runner...`).
   - Implementação de `Drop`: Ao sair do escopo (seja por sucesso do CI, falha, erro de I/O, `panic!` ou interrupção), o destrutor executa `docker unpause <container>` e exibe feedback amigável (ex: `[pre-push] ▶ Retomando forgejo-runner...`).
3. **Degradação Graciosa:**
   - Se o Docker não estiver instalado ou não estiver rodando: Nenhuma ação é tomada (no-op), sem causar falha.
   - Se nenhum container do runner estiver em execução: Operação continua normalmente (no-op).
   - Falhas no comando `docker pause` ou `docker unpause` não devem abortar a execução do `act_ci_gate`.
