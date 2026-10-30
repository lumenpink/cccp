# Active Tickets - cccp
> Synced at: 2026-10-30 19:53:04 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#101] feat(gost): isentar commits chore de exigencia de branch dedicada e ticket
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/101

### Description
### Descrição
Commits com tipo chore possuem propósito estritamente de manutenção de rotina e housekeeping. Pela norma GOST, commits do tipo chore devem ser isentos da exigência de branch dedicada, de ticket associado e de travas de commit direto na main, permitindo execuções ágeis de manutenção.

---
## [#100] feat(commit): adicionar flag --force ao cccp commit
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/100

### Description
### Descrição
Adicionar a opção -f / --force ao comando cccp commit para permitir forçar a gravação de commits em situações administrativas ou excepcionais quando explicitamente solicitado pelo usuário, desativando validações impeditivas locais.

---
## [#99] feat(ddtdd): simplificar option --phase do cccp commit para aceitar apenas numeros 3 a 8
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/99

### Description
### Descrição
Na CLI do cccp commit, a flag -p / --phase atualmente exige ou aceita prefixos como ph3, ph4. O usuário deve poder passar apenas o número da fase (ex.: -p 3 ou -p 8).

### Regras de Validação
1. Aceitar apenas números de 3 a 8 (ex.: 3, 4, 5, 6, 7, 8).
2. Não aceitar fases 1 e 2 (pois são Ticket e Branch, sem commits no repositório).
3. Não aceitar fase 9 (pois a Fase 9 deve ser obrigatoriamente executada através do comando cccp close).

---
