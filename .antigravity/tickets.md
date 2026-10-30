# Active Tickets - cccp
> Synced at: 2026-10-30 21:35:21 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#103] feat(ci): consultar fila de ci consolidada via user actions runners jobs
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/103

### Description
### Descrição
Atualmente a sincronização e inspeção de CI exige múltiplas requisições HTTP sequenciais repo por repo (136 chamadas para 68 repositórios), gerando lentidão extrema no carregamento.
O Forgejo oferece o endpoint consolidado GET /api/v1/user/actions/runners/jobs que retorna toda a fila ativa e em execução em uma única requisição ultrarrápida (< 400ms).

### Requisitos
1. Adicionar método 'fetch_user_action_jobs' no ForgeClient para consumir GET /api/v1/user/actions/runners/jobs.
2. Integrar a consulta consolidada no monitor de CI / repo top para alimentar os status de execução em tempo real sem chamadas individuais repetitivas.
3. Executar o ciclo GOST DD-TDD fases 1 a 8 com testes de mutação e quality gate estrito.

---
## [#102] fix(top): ajustar layout responsivo 80 colunas corte utf8 e padding ansi
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/102

### Description
### Descrição
A visualização do 'cccp repo top' encontra-se quebrada em terminais de 80 colunas e telas divididas devido a colunas fixas de 93 caracteres, contagem incorreta de padding em strings com sequências de escape ANSI e fatiamento inseguro de strings UTF-8.

### Requisitos
1. Calcular o layout proporcionalmente à largura do terminal (mínimo 80 colunas), evitando quebras de linha automáticas (line wrap) que destroem o TUI.
2. Implementar padding visual imune a escapes ANSI para o status de CI, garantindo que as colunas seguintes permaneçam rigorosamente alinhadas.
3. Unificar ou redimensionar as colunas de nome e slug com corte seguro em UTF-8 usando reticências (…).

---
