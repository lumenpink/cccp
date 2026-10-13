# Especificação Técnica: Detecção de Forge para TildeGit e Separação de Conceitos (Forge vs ForgeSoftware)

**Ticket Associado:** #137  
**Branch:** `fix/137-forge-detection-tildegit`  
**Norma:** GOST 1.4.0 (DD-TDD)

## 1. Contexto e Problema
Durante a execução de comandos como `cccp divan` e `cccp catalog scan`, repositórios hospedados em `tildegit.org` (por exemplo, `install_i3` com remote `git@tildegit.org:lumen/install_i3.git`) foram identificados incorretamente ou geraram alertas de token/configuração por falta de distinção conceitual entre:
1. **Forge (Instância/Provedor):** A instância canônica do serviço git (ex.: `tildegit`, `codeberg`, `github`, `gitlab`).
2. **ForgeSoftware (Motor/Protocolo API):** O motor de software executando a API REST remota (ex.: `forgejo` ou `gitea` compatível com `/api/v1`, `github` com `/api/v3` ou REST v3, `gitlab` com `/api/v4`).

Anteriormente, o enum `ForgeType` acumulava ambos os papéis e não conhecia `Tildegit`, resultando em fallbacks arbitrários ou inconsistências de nomenclatura no catálogo e na resolução de tokens de autenticação.

## 2. Decisão Arquitetural e Modelagem de Dados

### 2.1. Modelos
- **`ForgeSoftware` (Novo Enum):**
  - `Forgejo` (cobre Forgejo e Gitea v1 REST API em `/api/v1/...`).
  - `GitHub` (REST API v3).
  - `GitLab` (REST API v4).
- **`ForgeType` (Enum de Instância / Provedor Canônico):**
  - `Codeberg` (software: `Forgejo`, host padrão: `codeberg.org`, token env: `CODEBERG_TOKEN`, `CODEBERG_ORG_TOKEN`)
  - `Tildegit` (software: `Forgejo`, host padrão: `tildegit.org`, token env: `TILDEGIT_TOKEN`, `TILDEGIT_ORG_TOKEN`)
  - `Forgejo` (instâncias genéricas ou auto-hospedadas de Forgejo/Gitea)
  - `GitHub` (software: `GitHub`, host padrão: `github.com`, token env: `GITHUB_TOKEN`, `GH_TOKEN`)
  - `GitLab` (software: `GitLab`, host padrão: `gitlab.com`, token env: `GITLAB_TOKEN`, `GL_TOKEN`)

`ForgeType` implementará `pub fn software(&self) -> ForgeSoftware`.

### 2.2. Estrutura `RepoForgeInfo`
- `pub provider: ForgeType` (compatibilidade retroativa com código existente).
- `pub software: ForgeSoftware` (software que dita os endpoints e payload de API).
- `pub host: String` (FQDN canônico, ex.: `tildegit.org`, `codeberg.org`).
- `pub owner: String`.
- `pub repo: String`.

Adicionalmente, `RepoForgeInfo` terá métodos auxiliares:
- `pub fn forge_name(&self) -> &str`: Retorna o identificador amigável da forge (`"tildegit"`, `"codeberg"`, `"github"`, `"gitlab"`, `"forgejo"`).
- `pub fn software_name(&self) -> &str`: Retorna o identificador do software (`"forgejo"`, `"github"`, `"gitlab"`).

## 3. Heurística de Detecção de URL Remota (`parse_remote_url`)
A função `parse_remote_url` analisará o host:
- Se `host_lower.contains("codeberg.org")` -> `ForgeType::Codeberg` (software `Forgejo`).
- Se `host_lower.contains("tildegit.org")` -> `ForgeType::Tildegit` (software `Forgejo`).
- Se `host_lower.contains("github.com")` -> `ForgeType::GitHub` (software `GitHub`).
- Se `host_lower.contains("gitlab") || host_lower.contains("framagit")` -> `ForgeType::GitLab` (software `GitLab`).
- Demais hosts -> `ForgeType::Forgejo` (software `Forgejo`).

## 4. Resolução de Tokens (`resolve_token_for_host`)
Para `ForgeType::Tildegit`:
- Prioridade 1: Variável de ambiente baseada no domínio (`TILDEGIT_ORG_TOKEN`).
- Prioridade 2: Chave em `.cccprc` / `.env` (`tildegit_org_token`).
- Prioridade 3: Variáveis genéricas da instância: `TILDEGIT_TOKEN`, `FORGEJO_TOKEN`, `GITEA_TOKEN`, `TEA_TOKEN`.
- Prioridade 4: Chaves genéricas em `.cccprc` / `.env`.

## 5. Integração com o Catálogo
No `catalog.rs`:
- Entradas de catálogo com `forge: "tildegit"` mapeiam para `host: "tildegit.org"`, `provider: ForgeType::Tildegit`, `software: ForgeSoftware::Forgejo`.
- Ao sincronizar ou salvar entradas no catálogo originadas de `tildegit.org`, o campo `forge` será gravado como `"tildegit"` (e não `"forgejo"` ou fallback para `"codeberg"`).
