# Especificação Técnica: Sobrescrita de Binário Nightly no Forgejo Generic Packages

**Ticket Associado:** #94  
**Branch:** `fix/94-sobrescrever-nightly-generic-packages`  
**Norma:** GOST 1.5.0

## 1. Objetivo
Garantir que toda execução da pipeline de CI na branch principal (`main`) publique e sobrescreva de forma determinística o binário `cccp-linux-amd64` no Forgejo Generic Packages (`https://gambi.2lp.in/api/packages/lumen/generic/cccp/nightly/cccp-linux-amd64`), prevenindo rejeições por duplicidade (`HTTP 409 Conflict`).

## 2. Requisitos Técnicos
1. **Exclusão Prévia Obrigatória (`DELETE`)**:
   - Antes do envio do binário via `PUT`, o workflow `.forgejo/workflows/ci.yml` deve emitir uma requisição `DELETE` autenticada contra o endpoint do arquivo no Forgejo Generic Packages:
     `DELETE https://gambi.2lp.in/api/packages/lumen/generic/cccp/nightly/cccp-linux-amd64`
   - O comando deve tolerar falha graciosa caso o pacote ainda não exista (`|| true`).
2. **Publicação via Envio (`PUT`)**:
   - Imediatamente após a remoção ou confirmação de ausência, enviar o novo binário compilado estático `target/x86_64-unknown-linux-musl/release/cccp` com `curl -s -f --retry 3 -X PUT`.
3. **Contratos e Testes de Pipeline (`tests/ci_pipeline_test.rs`)**:
   - Validar estritamente que a rotina de publicação contém tanto o método `DELETE` de pré-limpeza quanto o método `PUT` de upload com as devidas autorizações e targets.

## 3. Critérios de Aceitação
- Teste unitário e testes de mutação cobrindo a rotina de exclusão prévia no workflow.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou erros.

## 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**: Implementada requisição `DELETE` prévia com tolerância graciosa antes do upload via `PUT` no endpoint do Forgejo Generic Packages. Testes de contrato, testes de mutação, formatação rustfmt e clippy 100% aprovados sem erros ou avisos.


