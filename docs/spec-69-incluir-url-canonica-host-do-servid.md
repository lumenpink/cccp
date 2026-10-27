# Especificação Técnica — Ticket #69: URL Canônica, Host do Servidor e Apelidos Customizáveis no Catálogo

## 1. Visão Geral
O catálogo Gosplan State Catalog (`catalog.json`) centraliza o inventário de repositórios do ecossistema. Para melhorar a navegação, interoperabilidade com forjas soberanas e comandos interativos (como Divã e Assemble), é necessário enriquecer os metadados de cada repositório com seu endereço canônico completo (`canonical_url`), o host do servidor de forja (`server_host`), além de atribuir apelidos automáticos derivados do domínio e permitir a fixação de apelidos arbitrários no arquivo de configuração do CCCP (`.cccprc`).

---

## 2. Requisitos Técnicos

### 2.1 Modelagem de Dados
- Em `CatalogEntry`:
  - `pub canonical_url: Option<String>`: URL completa canônica (ex.: `https://gambi.2lp.in/lumen/cccp`). Se omitida no JSON, é deduzida automaticamente a partir da forja (`forge`) e do slug (`slug`).
  - `pub server_host: Option<String>`: Nome do host da forja (ex.: `gambi.2lp.in`). Se omitido, é derivado da URL ou do identificador da forja.

### 2.2 Atribuição Automática de Apelidos por Domínio
- Implementar algoritmo determinístico de extração do domínio (`extract_domain_prefix`):
  - Divide o host em segmentos separados por ponto `.`.
  - Se o primeiro segmento for `git` (ex.: `git.2lp.in`, `git.subversao.su`), utiliza o segundo segmento (`2lp`, `subversao`).
  - Caso contrário, utiliza o primeiro segmento (ex.: `gambi` em `gambi.2lp.in`, `codeberg` em `codeberg.org`).
- Injetar automaticamente nos `aliases` do repositório as variações qualificadas:
  - `{domain}:{repo}` (ex.: `gambi:cccp`)
  - `{domain}/{repo}` (ex.: `gambi/cccp`)

### 2.3 Apelidos Customizados via Configuração
- Em `Config`:
  - Suportar mapeamento de apelidos customizados: `pub repo_aliases: HashMap<String, Vec<String>>`.
  - Carregar chaves `alias.<repo> = "apelido1, apelido2"` e `aliases.<repo> = "..."` no parser do `.cccprc`.
  - Durante o carregamento do catálogo (`load_catalog`), estender os `aliases` de cada entrada com os apelidos customizados correspondentes.

---

## 3. Matriz de Testes (Fases 4 & 7)
- `tests/catalog_metadata_aliases_test.rs`:
  1. Teste de resolução e armazenamento de `canonical_url` e `server_host`.
  2. Teste da regra de atribuição automática de apelido de domínio (`gambi.2lp.in` -> `gambi:repo`, `git.subversao.su` -> `subversao:repo`).
  3. Teste de injeção de apelidos customizados via configuração `.cccprc` / `Config`.
  4. Teste de busca no catálogo (`assemble_catalog_repo`) utilizando apelido de domínio e apelido customizado.
