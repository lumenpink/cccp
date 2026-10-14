# Especificação Técnica: Isolamento de Jobs de Teste no CI Gate Local (`act`)

**Ticket Associado:** #130  
**Branch:** `fix/130-isolate-ci-test-jobs`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Garantir que a execução local de CI via `act` (no pre-push hook e em `cccp act run`) execute e valide exclusivamente etapas de testes, linting e checagem estática, descartando jobs de deploy, release, publicação e CD (Continuous Delivery) presentes em workflows unificados de repositórios.

## 2. Motivação e Problema
Workflows de CI/CD que combinam testes e deploy (ex.: jobs `deploy`, `publish`, `release`, `pages`, `upload-artifacts`) frequentemente utilizam:
1. Segredos de produção (ex.: tokens de nuvem, chaves de API, credenciais de registro) indisponíveis no ambiente local.
2. Cláusulas condicionais complexas (`if: github.ref == 'refs/heads/main'`) ou dependências encadeadas (`needs: [test]`).
3. Expressões ou sintaxes específicas de runners de produção que causam erros de sintaxe no `act` local (*"workflow is not valid"*), abortando indevidamente o pre-push hook.

## 3. Critérios de Isolamento e Comportamento
1. **Identificação de Jobs de Teste / Verificação:**
   - Jobs elegíveis: qualquer job cujo identificador contenha `test`, `check`, `lint`, `fmt`, `audit`, `verify`, ou o job explicitamente configurado via `CCCP_ACT_JOB`.
2. **Exclusão de Jobs de Entrega/Deploy:**
   - Jobs cujo identificador contenha termos como `deploy`, `release`, `publish`, `upload`, `pages`, `docker`, `package` ou que dependam exclusivamente de segredos de publicação devem ser removidos do workflow temporário gerado para o `act`.
   - Cláusulas `needs:` nos jobs mantidos devem ser limpas caso referenciem jobs que foram removidos.
3. **Degradação Graciosa:**
   - Caso o workflow não possua múltiplos jobs ou nenhum job de deploy seja detectado, o workflow é mantido com a sanitização padrão de URLs.
   - Caso o parsing ou manipulação do YAML encontre erros inesperados, o CCCP recai com segurança para o workflow original sanitizado sem quebrar o fluxo.
