# Active Tickets - cccp
> Synced at: 2026-11-11 21:56:14 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#170] refactor(rules): desacoplar motor lua e delegar auditoria ao lunokhod
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/170

### Description
## Objetivo
Remover o runtime de regras Lua ('mlua') do CCCP, eliminando código C embutido e tornando o CCCP Rust puro. A auditoria estática ('cccp check') delega a execução diretamente ao executável 'lunokhod'.

## Escopo
- Purgar 'src/lua_rules.rs' e testes associados.
- Remover 'mlua' e 'lunokhod' como dependências de biblioteca no Cargo.toml.
- 'cccp check' invoca o binário 'lunokhod' via subprocesso.

---
## [#169] refactor(cli): delegar catalogo e scaffolding ao gossnab
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/169

### Description
## Objetivo
Desacoplar o CCCP do gerenciamento multi-repositório e criação de projetos, mantendo o CCCP estritamente focado na governança interna do repositório corrente.

## Escopo
- Purgar 'src/catalog.rs', 'src/newrepo.rs' e testes associados.
- Purgar subcomandos 'repo', 'new' e 'spool' da CLI.

---
## [#168] refactor(cli): delegar gestao de forjas e tickets ao gossnab
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/168

### Description
## Objetivo
Desacoplar o CCCP de interações com APIs remotas de forjas (Forgejo/GitHub/Codeberg), eliminando o subcomando 'ticket' e 'divan' e tornando 'close' e 'merge' operações estritamente locais do Git.

## Escopo
- Purgar 'src/forges/', 'src/ai.rs' e testes associados do CCCP.
- Purgar subcomandos 'ticket' e 'divan' da CLI.
- Adaptar 'cccp close' e 'cccp merge' para registrarem o commit de fechamento localmente sem chamadas de API externa.

---
## [#167] refactor(expurge): purgar gerador de site banner
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/167

### Description
## Objetivo
Remover a geração de banners HTML embutidos para sites de documentação ('src/site_banner.rs' e subcomando 'site-banner'), delegando a compilação visual para os repositórios dos próprios sites.

## Escopo
- Deletar 'src/site_banner.rs' e 'tests/site_banner_test.rs'.
- Purgar subcomando 'site-banner' da CLI em 'src/cli.rs' e chamadas em 'src/main.rs'.

---
## [#166] refactor(expurge): purgar cockpit web e matraca
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/166

### Description
## Objetivo
Remover completamente o servidor web embutido (cockpit), exportação de assets web e o protocolo de mensageria assíncrona (matraca), que representam overengineering no maestro de commits e hooks.

## Escopo
- Deletar 'src/cockpit.rs', 'src/web.rs', 'src/matraca.rs' e testes associados.
- Purgar subcomando 'cockpit' da CLI.
- Eliminar dependências não essenciais no Cargo.toml.

---
