# Especificação Técnica — Ticket #174
## Preservar Status Skip para Regras Desabilitadas e Assegurar GOSTVER no Scaffolding

- **Ticket:** #174
- **Branch:** `fix/174-preservar-status-skip-de-regras-desabilitadas-e-assegurar-gostver`
- **Data:** 2026-11-13
- **Autor:** Lumen Pink (Baquara)

---

## 1. Contexto e Motivação
A esteira de integração contínua do CCCP apresentou duas falhas de auditoria e testes:
1. Em `src/lua_rules.rs`: A função `evaluate_lua_rules_with_lang` descartava regras desabilitadas via `.filter()`. Como consequência:
   - As regras desabilitadas não apareciam no relatório final com o status canônico `CheckStatus::Skip`.
   - Se o conjunto de regras avaliado consistisse exclusivamente de regras desabilitadas, o retorno era uma lista vazia, fazendo com que o `audit.rs` emitisse o falso positivo `[FAIL] NO-RULES-FOUND`.
2. Em `src/newrepo.rs`: O comando `cccp repo init` delega o provisionamento ao Gossnab (`gossnab new`). O Gossnab moderno gera apenas `.rules/profile.toml`, deixando de criar o arquivo canônico `GOSTVER` (exigido para auditoria da versão da norma GOST) e inicializando repositórios git mesmo quando a flag `--no-git` é passada.

---

## 2. Requisitos e Implementação
- **`src/lua_rules.rs`:**
  - Em vez de filtrar e descartar regras presentes em `disabled_rules`, mapear essas regras diretamente para `CheckStatus::Skip`.
  - Definir a mensagem de detalhe como `"Rule <CODE> disabled in configuration"`.
- **`src/newrepo.rs`:**
  - Após a execução com sucesso do `gossnab new`, verificar se `target_root/GOSTVER` existe; caso não exista, criar com o conteúdo padrão `GOST_VERSION="1.1.0"\n`.
  - Se `opts.git == false` e `.git` foi gerado, remover a pasta `.git` do repositório provisionado.
- **`tests/config_disable_test.rs`:**
  - Criar o catálogo declarativo em `.rules/rules.toml` para conformidade com a descoberta de regras do Lunokhod.

---

## 3. Critérios de Aceite e Validação
- `cargo test --test config_disable_test` passa com 100% de sucesso e status `CheckStatus::Skip` validado.
- `cargo test --test e2e_cli_test` passa com 100% de sucesso, confirmando a presença de `GOSTVER` e o respeito à flag `--no-git`.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` limpos com zero advertências.
