# Relatório Consolidado de Migração (GOST DD-TDD)

- **Perfil:** `lilya`
- **Baseline Instalada:** `2.0.0`
- **Versão Alvo:** `2.1.0`
- **Cadeia Percorrida:** 2.0.0 ➔ 2.1.0
- **Quebras de Contrato:** Sim (atenção redobrada requerida)

---

## Delta de Migração para 2.1.0

# Guia de Migração: Perfil Lilya 2.0.0 ➔ 2.1.0

Este documento formaliza as alterações contratuais e os passos operacionais para elevar um repositório regulado da versão `2.0.0` para a versão `2.1.0` do perfil operacional Lilya.

---

## 1. Quebras de Contrato (*Breaking Changes*)

1. **Subordinação Estrita a Kollontai e Metanorma GOST v3.0.0:**
   - O perfil Lilya deixa de atuar como norma autônoma e passa a estender formalmente o perfil de ecossistema Kollontai, ancorando-se na Metanorma Fundamental GOST v3.0.0.
2. **Proibição de Manifestos Estáticos Locais (Regra LILYA-005):**
   - É expressamente vedada a permanência de `catalog.json` e `kombinat.toml` nos repositórios. O inventário de aplicações e dependências é governado dinamicamente via Gossnab.
3. **Extensão Mandatória `.yaml` para Docker Compose (Regra LILYA-006):**
   - O uso da extensão `.yml` para orquestração Docker é proibido. Todo arquivo deve ser nomeado exclusivamente como `compose.yaml` ou `docker-compose.yaml`.
4. **Manifesto de Metadados Canônico (Regra LILYA-007):**
   - Todo repositório deve manter `.gost/metadata.json` válido, declarando compulsoriamente o campo `schema_version`.
5. **Conformidade de Linha de Comando Post-Build (Regras LILYA-008 e LILYA-009):**
   - Binários executáveis compilados devem responder a `--help` com texto estruturado informativo e a `--version` com paridade estrita em relação ao arquivo `VERSION`.
6. **Eliminação de Rotas Desversionadas e Aliases Legados:**
   - As regras da norma são servidas exclusivamente no padrão imutável `{base_url}/{version}/RULES.md` e `RULES.toml`. Rotas legadas desversionadas na raiz não são suportadas (postura fail-fast).

---

## 2. Mutação de Regras Declarativas (`.rules/rules.toml`)

- **Adicionadas (+):**
  - `LILYA-005`: Proibição de manifestos estáticos `catalog.json` e `kombinat.toml`.
  - `LILYA-006`: Exigência estrita de extensão `.yaml` para compose.
  - `LILYA-007`: Validação de esquema do manifesto `.gost/metadata.json`.
  - `LILYA-008`: Validação de texto informativo em `--help` no escopo `post-build`.
  - `LILYA-009`: Validação de paridade estrita de versão em `--version` no escopo `post-build`.
- **Modificadas (~):**
  - `LILYA-002`: Afirmação de identidade transfeminista obrigatória no `profile.md` ou `README.md`.
  - `LILYA-003`: Declaração de `primary_forge_domain = "gambi.2lp.in"` no `.cccprc`.

---

## 3. Ações Manuais Necessárias

1. **Remover Manifestos Estáticos Legados:**
   ```sh
   rm -f catalog.json kombinat.toml
   ```
2. **Renomear Orquestração Compose:**
   ```sh
   [ -f compose.yml ] && mv compose.yml compose.yaml
   [ -f docker-compose.yml ] && mv docker-compose.yml docker-compose.yaml
   ```
3. **Garantir Manifesto de Metadados:**
   - Criar ou atualizar `.gost/metadata.json` contendo `"schema_version": "1.0.0"`.
4. **Verificar Forja Canônica em `.cccprc`:**
   - Assegurar que `primary_forge_domain = "gambi.2lp.in"` consta na configuração.

---

## 4. Ações Automatizáveis via Ferramenta

Execute os comandos do Maestro local para atualizar as definições normativas e recompilar o catálogo de regras:
```sh
cccp profile set lilya
cccp profile assemble
```
Após a montagem, execute a auditoria rigorosa de conformidade:
```sh
cccp check --strict
```

---

