# Especificação Técnica: Carriage Return CRLF e Limpeza de Linha no Modo Raw do CCCP Repo Top

**Ticket Associado:** #104  
**Branch:** `fix/104-aplicar-carriage-return-crlf-e-limpeza-de-linha-no-modo-raw`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Eliminar definitivamente o "efeito escada" (*staircase effect*) no monitor interativo `cccp repo top`, garantindo que todas as quebras de linha emitidas para o terminal em modo raw utilizem o par Carriage Return + Line Feed (`\r\n`) e escape de limpeza até o final da linha (`\x1b[K`), retornando o cursor à coluna 0 a cada linha renderizada.

## 2. Requisitos Técnicos
1. **Quebra de Linha em Modo Raw (`\r\n`)**:
   - O `render_top_view` deve emitir `\r\n` em vez de `\n` isolado em todas as quebras de linha (cabeçalho, divisor, linhas da tabela e rodapé).
   - Isso garante conformidade com o driver TTY em modo raw (`enable_raw_mode`), onde o processamento de saída `ONLCR` fica desativado.
2. **Limpeza de Linha com Escape ANSI (`\x1b[K`)**:
   - Cada linha renderizada na tabela e no rodapé deve incluir `\x1b[K` antes de `\r\n`, garantindo que caracteres remanescentes de redesenhos anteriores não persistam na tela.
3. **Compatibilidade com Testes Existentes**:
   - Manter as funções utilitárias `pad_visual` e `truncate_utf8_ellipsis`.
   - Atualizar a suite de testes para verificar explicitamente a presença de `\r\n` nas linhas renderizadas.

## 3. Critérios de Aceitação
- Testes unitários comprovando que `render_top_view` termina suas linhas com `\r\n` (ou `\r`).
- Testes unitários comprovando ausência de `\n` desacompanhado de `\r`.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e bateria de testes 100% verde.
