# Especificação Técnica — Ticket #176: Implementar Resolução Linear de Upgrade com valid_since e replaces

- **Ticket:** [#176](https://gambi.2lp.in/lumen/cccp/issues/176)
- **Branch:** `feat/176-implementar-resolucao-linear-de-upgrade-com-valid-since-e-replaces`
- **Autoridade Normativa:** GOST v3.0.0 (Seção 2.2, 2.3 e 2.4) / Lilya v2.1.0

---

## 1. Contexto & Motivação

A Metanorma Fundamental GOST v3.0.0 consagra o **Princípio do Encadeamento Linear de Migração** (Seção 2.2) e a exigência do contrato `UPGRADE.md` exposto na raiz pública de cada versão imutável (`{base_url}/{version}/UPGRADE.md`).

O Maestro CCCP atua como o ponto único de entrada da governança operacional. Para que os repositórios regulados pelo ecossistema transitem de forma determinística e automatizada entre versões de perfis normativos, o `cccp` deve ser capaz de:
1. Identificar a versão instalada no projeto (`.rules/profile.toml` ou `GOSTVER` como baseline legado).
2. Descobrir a versão alvo (fornecida explicitamente ou obtida de `versions.json`).
3. Baixar e validar o contrato `UPGRADE.md` do alvo.
4. Aplicar o princípio de *Fail-Fast* estrito na validação do piso inferior `valid_since`: se a versão instalada for inferior ao `valid_since`, a execução é abortada no ato com erro canônico e orientação de auto-cura.
5. Percorrer recursivamente os elos declarados no campo `replaces` até atingir a versão instalada.
6. Concatenar os deltas de migração em ordem cronológica ascendente gerando o roteiro consolidado em `.gost/UPGRADE.md`.
7. Atualizar a declaração do perfil em `.rules/profile.toml` e acionar a montagem determinística de regras.

---

## 2. Decisões Arquiteturais

### 2.1 Interface de Linha de Comando (CLI)
Em conformidade com o **Princípio da Unicidade Canônica de Interface** (GOST v3.0.0, Seção 2.4):
- **Comando Canônico:** `cccp profile upgrade [OPTIONS] [TARGET_VERSION]`
- **Atalho de Primeiro Nível:** `cccp upgrade [OPTIONS] [TARGET_VERSION]` (alias transparente despachado para a mesma rotina interna).
- **Parâmetros e Opções:**
  - `TARGET_VERSION` (opcional): versão alvo semântica (ex.: `2.1.0`). Se omitida, adota a chave `latest` do `versions.json` remoto.
  - `-n, --dry-run`: Executa a descoberta e validação da cadeia sem gravar arquivos em disco.
  - `-C, --dir <DIR>`: Diretório base do repositório a inspecionar/migrar.

### 2.2 Parseamento do Contrato UPGRADE.md
O parser extrai o Frontmatter YAML delimitado por `---`:
```yaml
---
standard: "lilya"
tier: "operational"
version: "2.1.0"
replaces: "2.0.0"
valid_since: "2.0.0"
breaking: true
---
```
Campos obrigatórios:
- `standard`: String identificadora da norma/perfil.
- `version`: Versão correspondente à rota versionada.
- `replaces`: Versão anterior imediatamente substituída (ou `null` para versão primordial).
- `valid_since`: Versão mínima requerida da baseline instalada.
- `breaking`: Booleano indicando quebra de contrato.

### 2.3 Algoritmo de Resolução Linear e Fail-Fast
```text
[Início: cccp upgrade [alvo]]
  ├── Lê perfil atual em .rules/profile.toml (ou GOSTVER) -> (url, versao_instalada)
  ├── Se alvo ausente -> busca latest em {url}/versions.json
  ├── Se versao_instalada == alvo -> Encerra com sucesso ("Perfil já está na versão {alvo}")
  ├── Baixa {url}/{alvo}/UPGRADE.md
  ├── Parseia frontmatter
  ├── Se versao_instalada < valid_since:
  │     └── FALHA IMEDIATA (ERR_UPGRADE_BELOW_VALID_SINCE):
  │         "Versão instalada ({versao_instalada}) é inferior ao piso valid_since ({valid_since}).
  │          É necessária migração manual intermediária até a versão {valid_since}."
  ├── Loop recursivo:
  │     delta_list.prepend(alvo)
  │     atual = replaces
  │     Enquanto atual != versao_instalada e atual != null:
  │         Baixa {url}/{atual}/UPGRADE.md
  │         Parseia frontmatter
  │         delta_list.prepend(atual)
  │         atual = replaces_do_elo
  ├── Sintetiza deltas concatenados em ordem cronológica ascendente
  ├── Se dry-run: exibe plano e encerra
  └── Se execução real:
        ├── Grava .gost/UPGRADE.md
        ├── Atualiza .rules/profile.toml com versão alvo
        └── Invoca assemble_rules_specs(root)
```

---

## 3. Árvore de Decisões e Alternativas Descartadas (Para a Natasha)

- **Alternativa Descartada 1: Baixar todas as versões intermediárias presentes em `versions.json`.**
  - *Motivo do Descarte:* Viola o grafo causal linear do `replaces`. Um catálogo pode conter versões com tags descontinuadas ou ramificações táticas. O elo `replaces` reflete a exata sucessão planejada pela autoridade normativa.
- **Alternativa Descartada 2: Emitir apenas warning caso a versão seja inferior ao `valid_since`.**
  - *Motivo do Descarte:* Fere o princípio constitucional de Fail-Fast (Seção 2.3). Se o autor da versão estipulou que versões anteriores a `valid_since` não possuem trilha automatizada suportada, tentar interpolar deltas inexistentes causaria quebras silenciosas e deploys defeituosos.
- **Alternativa Descartada 3: Não gerar artefato local `.gost/UPGRADE.md`.**
  - *Motivo do Descarte:* O repositório migrado precisa registrar o laudo das quebras de contrato e as ações manuais pendentes para guiar a desenvolvedora nas fases subsequentes.

---

## 4. Critérios de Aceite e Matriz de Testes

1. `test_parse_upgrade_frontmatter`:
   - Valida extração correta de `version`, `replaces`, `valid_since`, `breaking` e corpo do documento.
2. `test_upgrade_fail_fast_below_valid_since`:
   - Assegura retorno de erro imediato caso `versao_instalada < valid_since`.
3. `test_upgrade_dry_run_leaves_disk_untouched`:
   - Garante que `--dry-run` não grava `.gost/UPGRADE.md` nem altera `.rules/profile.toml`.
4. `test_upgrade_linear_execution_writes_upgrade_md`:
   - Verifica execução real com gravação de `.gost/UPGRADE.md` e atualização do perfil para 2.1.0.
5. `test_mutation_parse_fails_without_valid_since`:
   - Rejeita frontmatter sem campo obrigatório `valid_since`.
6. `test_mutation_version_comparison`:
   - Valida asserções semânticas da comparação `is_version_below`.
7. `test_mutation_malformed_frontmatter`:
   - Rejeita documentos sem delimitadores `---`.

---

## 5. Resultados do Quality Gate (Fase 8)
- `cargo fmt --check`: Aprovado (0 inconsistências de estilo).
- `cargo clippy --test upgrade_resolution_test -- -D warnings`: Aprovado (0 warnings/erros).
- `cargo test --test upgrade_resolution_test`: Aprovado (7/7 testes aprovados).

