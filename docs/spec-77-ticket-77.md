# Especificação Técnica — Ticket #77

## Escopo
`fix(hooks): corrigir calculo de checksum e versao canonica nos hooks instalados`

## Contexto e Motivação
Ao instalar hooks via `cccp hook install`, o comando gravava uma versão estática (`3.1.0` de `CARGO_PKG_VERSION`) enquanto o runtime do binário valida contra `get_current_version()` (`3.2.0-dev...`).
Adicionalmente, o cálculo do checksum para detecção de customização local em `check_hook_version` filtrava linhas iniciadas por `#`, excluindo os comentários internos do script (`# 1. Prioritize cccp in PATH`, `# 2. Search common user...`). Como o hash original gravado no cabeçalho incluía essas linhas, a validação invariavelmente falhava, acusando errônea customização local em todos os repositórios gerenciados e emitindo advertências repetidas.

## Requisitos
1. **Versão Canônica na Instalação**:
   - `install_hooks` deve utilizar a versão canônica (`crate::update::get_current_version()`) ao invés do estático `CARGO_PKG_VERSION`.
2. **Extração Correta do Corpo do Hook**:
   - O corpo do script para cálculo de checksum deve ser extraído ignorando apenas o bloco de metadados do cabeçalho (linhas `#!/bin/sh`, `# cccp-hook-*` e o título Soviético). Linhas de comentário que fazem parte do script executável (`# 1. ...`, etc.) devem ser preservadas integralmente.
3. **Idempotência e Silêncio**:
   - Hooks intactos com a mesma versão de template não devem emitir avisos falso-positivos nem ser considerados modificados pelo usuário.
