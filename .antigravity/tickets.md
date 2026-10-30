# Active Tickets - cccp
> Synced at: 2026-10-30 15:04:21 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#93] fix(build): restaurar compilacao estatica musl para binario linux-amd64
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/93

### Description
### Descrição
Reverter a mitigação provisória do Ticket #89 e restaurar a compilação 100% estática e autocontida com x86_64-unknown-linux-musl para o binário Linux AMD64 de release e nightly do CCCP.

### Requisitos
1. Configurar variáveis estáticas no .cargo/config.toml (CC_x86_64_unknown_linux_musl = musl-gcc, OPENSSL_STATIC = 1, LIBZ_SYS_STATIC = 1, PKG_CONFIG_ALL_STATIC = 1).
2. Atualizar .forgejo/workflows/ci.yml para compilar e empacotar target/x86_64-unknown-linux-musl/release/cccp.
3. Atualizar contratos e testes de pipeline em tests/ci_pipeline_test.rs para validar target musl.
4. Executar fases DD-TDD 1 a 8 com testes de mutação e quality gate estrito.

---
## [#92] fix(hooks): instruir uso explicito de comandos cccp nas mensagens de bloqueio do git
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/92

### Description
### Descrição
Quando um usuário ou agente de IA executa comandos Git diretamente (como git commit, git push, git checkout, etc.), mensagens de bloqueio ou validação de hooks devem instruir explicitamente qual comando canônico do cccp deve ser utilizado no lugar do git (ex.: cccp commit, cccp push, cccp close, cccp sync).

Sem essa orientação explícita em todas as mensagens de barreira, usuários e agentes de IA podem se confundir e tentar contornos manuais com flags do git ou variáveis de ambiente de bypass (como CCCP_MAESTRO_ACTIVE=1), violando as salvaguardas da norma GOST v2.0.

### Requisitos
1. Revisar e padronizar todas as mensagens dos hooks (pre-commit, commit-msg, pre-push, post-commit, reference-transaction, post-merge) para indicar expressamente o subcomando cccp correspondente.
2. Alertar contra tentativas de contorno com comandos git manuais ou variáveis de bypass.
3. Assegurar formatação clara e comandos copiáveis para desenvolvedores humanos e agentes de IA.

---
