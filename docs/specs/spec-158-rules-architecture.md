# Especificação do Ticket #158: Migração da Arquitetura de Normas para `.rules/` e Catálogo `versions.json`

**Ticket:** #158  
**Fase:** Fase 3 (Spec / Contrato Arquitetural)  
**Status:** Concluído (Fase 9)  
**Autor:** Lumen Pink / Antigravity  

---

## 1. Contexto e Motivação

O Maestro `cccp` foi concebido originalmente como a ferramenta oficial de governança do padrão GOST. Com a recentíssima migração para o motor de auditoria declarativo agnóstico `lunokhod` (Ticket #155), tornou-se imperativo desvincular estruturalmente o `cccp` de qualquer acoplamento nominal a "GOST", "Lilya" ou "Kollontai".

Além disso, identificou-se a poluição por artefatos legados da era GOST 1.0 (`GOSTVER`, `.gost/PROFILE.md`, `.gost/rules.json`, `.gost/languages/`) e a ausência de versionamento explícito nas URLs de perfil.

Esta especificação formaliza a nova arquitetura universal **`.rules/`**:
- **Pasta SSOT:** `.rules/` na raiz do projeto.
- **Configuração:** `.rules/profile.toml` com declaração canônica de `{ url = "...", version = "..." }`.
- **Artefatos Compilados Únicos:**
  1. `.rules/RULES.md`: Texto humano e diretrizes normativas agregadas de todas as camadas.
  2. `.rules/RULES.toml`: Regras declarativas compiladas executadas pelo Lunokhod.
- **Catálogo de Versões:** Consumo de `{url}/versions.json` para auto-descoberta e fixação imutável da versão estável `latest`.

---

## 2. Requisitos Técnicos

### 2.1 Configuração Local (`.rules/profile.toml`)
A raiz do repositório deixa de utilizar o arquivo `GOSTVER`. Todas as diretrizes de perfil passam a residir em `.rules/profile.toml`:
```toml
url = "https://lilya.2lp.in"
version = "2.1.0"
```

### 2.2 Descoberta de Versão via `versions.json`
Quando um repositório é inicializado com apenas a URL base (ex.: `cccp repo init --profile https://lilya.2lp.in`), ou quando se invoca verificação de atualização (`cccp profile check-update`):
1. O `cccp` requisita `GET {url}/versions.json`.
2. O payload JSON deve conter:
   ```json
   {
     "standard": "<id>",
     "latest": "<semver>",
     "versions": ["<semver>", ...]
   }
   ```
3. O `cccp` extrai o valor de `latest`, grava em `.rules/profile.toml` como `version = "<latest>"` e procede com o download determinístico.

### 2.3 Resolução Canônica de Endpoints Versionados
A partir de `url` e `version`, o `cccp` constrói a cadeia de precedência para download:
- **Norma Textual:**
  1. `{url}/{version}/RULES.md`
  2. `{url}/{version}/profile.md`
  3. `{url}/RULES.md`
  4. `{url}/profile.md`
- **Regras Declarativas:**
  1. `{url}/{version}/RULES.toml`
  2. `{url}/{version}/rules.toml`
  3. `{url}/RULES.toml`
  4. `{url}/rules.toml`

### 2.4 Montagem e Geração dos Artefatos Compilados (`assemble`)
1. O comando `assemble` gera estritamente dois artefatos na pasta `.rules/`:
   - `.rules/RULES.md`
   - `.rules/RULES.toml`
2. **Purga Automática:** Se existirem no diretório `.rules/` ou `.gost/` resquícios legados (`rules.json`, `PROFILE.md`, `GOST.md`), o processo de montagem e migração purga esses arquivos.

### 2.5 Auditoria Agnóstica no Lunokhod
O módulo de auditoria (`src/audit.rs`) passa a buscar preferencialmente `.rules/RULES.toml` (com fallback para `.rules/rules.toml` ou `.gost/rules.toml`).

---

## 3. Árvore de Decisões Arquiteturais (para a Natasha)

1. **Por que eliminar o arquivo `GOSTVER` da raiz?**
   - *Alternativa Descartada:* Manter `GOSTVER` na raiz e apenas alterar os caminhos internos.
   - *Motivo do Descarte:* O `GOSTVER` é específico da GOST. Ferramentas e usuários externos que queiram usar o `cccp` com outras normas teriam que manter um arquivo nomeado `GOSTVER`. Centralizar tudo em `.rules/profile.toml` isola as regras em um único diretório.
2. **Por que nomear os artefatos compilados como `RULES.md` e `RULES.toml`?**
   - *Decisão:* Simetria e simplicidade. O desenvolvedor ou IA sabe que em `.rules/` há `profile.toml` (configuração), `RULES.md` (o texto) e `RULES.toml` (o executável de regras).
3. **Por que corte limpo em vez de tolerância eterna com código zumbi?**
   - *Decisão:* Os repositórios do ecossistema estão sob nosso controle no nó central. Manter branches de código legadas para ler `.gost/` e `GOSTVER` por anos traria dívida técnica e fragilidade de testes para a Natasha.

---

## 4. Plano de Fases DD-TDD
- **Fase 4 (Red Test):** Testes unitários em `tests/rules_domain_test.rs` comprovando a leitura de `.rules/profile.toml`, montagem de `.rules/RULES.md` / `.rules/RULES.toml` e resolução de `versions.json`.
- **Fase 5 (Green Implement):** Refatoração em `src/profile.rs`, `src/newrepo.rs`, `src/audit.rs` e `src/main.rs`.
- **Fase 7 (Mutation Test):** Mutantes testando fallback de endpoints versionados e resolução de `latest`.
- **Fase 8 (Quality Gate):** `cargo fmt`, `cargo clippy`, `CHANGELOG.md`.
- **Fase 9 (Close & Merge):** Fechamento e publicação.
