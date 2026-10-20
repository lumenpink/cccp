# Active Tickets - cccp
> Synced at: 2026-10-20 13:07:21 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#158] fix(cli): exibir versão canônica do VERSION no cccp version e limpar exibição do GOSTVER
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/158

### Description
Garantir que cccp version, cccp -V e updater utilizem a versão canônica definida em VERSION (e não a tag da última versão do Cargo.toml). Além disso, extrair adequadamente apenas o GOST_VERSION de GOSTVER em vez de imprimir o arquivo multilinha inteiro.

---
