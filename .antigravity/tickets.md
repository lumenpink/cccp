# Active Tickets - cccp
> Synced at: 2026-10-29 20:41:41 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#86] fix(ci): injetar no_proxy apenas nos passos de teste com acesso local
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/86

### Description
## Description
Injetar explicitamente as variaveis de ambiente no_proxy e NO_PROXY ('localhost,127.0.0.1,::1') exclusivamente no passo 'Run Unit & Integration Tests' do workflow de CI, prevenindo interceptacao indevida de conexoes locais pelo proxy do runner.

## Acceptance Criteria
- [ ] Definir no_proxy e NO_PROXY pontualmente no passo de testes em .forgejo/workflows/ci.yml.
- [ ] Tornar o tratamento de erro em tests/ai_test.rs tolerante a diferencas de formato de erro de conexao.
- [ ] Validar execucao com cargo test --test ai_test sob presenca de http_proxy.
- [ ] Passar quality gates rigorosos (cargo fmt, cargo clippy -D warnings).

---
