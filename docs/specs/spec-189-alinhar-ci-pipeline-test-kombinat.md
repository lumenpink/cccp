# Especificação do Ticket #189: Alinhar `ci_pipeline_test.rs` com Delegação ao Kombinat via Generic Packages

## 1. Contexto e Motivação
Após a migração do `.forgejo/workflows/ci.yml` do `cccp` para publicar exclusivamente nos Generic Packages da Gambi (Ticket #188), dois testes legados em `tests/ci_pipeline_test.rs` falharam na CI:
1. `test_ci_workflow_overwrites_generic_packages_with_delete_before_put`: buscava o primeiro `-X PUT` do arquivo inteiro (que agora pertence aos pacotes `${VERSION}`) em vez do `-X PUT` do alias `nightly/cccp-linux-amd64` após o `-X DELETE`.
2. `test_ci_workflow_rotates_catalog_json_with_manifest`: exigia rotação direta na Banca (`git add site/downloads/cccp/index.json site/downloads/catalog.json`), responsabilidade agora delegada ao Kombinat.

## 2. Requisitos da Correção
1. Atualizar `test_ci_workflow_overwrites_generic_packages_with_delete_before_put` para validar que existe um `-X PUT` para `nightly/cccp-linux-amd64` posicionado **após** o `-X DELETE` no mesmo endpoint.
2. Atualizar `test_ci_workflow_rotates_catalog_json_with_manifest` para validar que `ci.yml` publica artefatos nos Generic Packages da Gambi e **não** contém `wrangler` nem rotação direta na Banca (`rotate-downloads.py`).
3. Validar `cargo test --test ci_pipeline_test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cccp check --strict`.

## 3. Árvore de Decisões (para a Natasha)
- **Inversão de Contrato no Teste:** Preservar a cobertura de testes de workflow invertendo as asserções legadas de rotação manual na Banca para proibição de `wrangler` e rotação direta, garantindo que regressões não reintroduzam deploys acoplados na CI.
