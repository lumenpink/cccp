# Especificação Técnica — Ticket #76
## feat(ci): subcomandos de esteira cccp ci publish-artifacts e cccp ci deploy-site

### 1. Contexto e Motivação
Em conformidade com a Seção 10 da norma GOST 2.0 e a Seção 5 de Kollontai 2.0, as esteiras de integração e entrega contínua do ecossistema soberano Gosplan operam de forma isolada e declarativa (`.forgejo/workflows/ci.yml`).
Duas etapas críticas de release vinham sendo orquestradas por scripts de shell dispersos:
1. Publicação determinística de artefatos binários e manifestos para o Cloudflare R2 / Banca (`banca.2lp.in`).
2. Publicação e deploy da vitrine estática de documentação em branch órfã stateless `pages`.

Este ticket introduz os subcomandos oficiais `cccp ci publish-artifacts` e `cccp ci deploy-site`, integrando essas operações diretamente no CLI do CCCP com suporte a verificação criptográfica SHA-256 e operações idempotentes.

### 2. Especificação dos Subcomandos

#### 2.1 `cccp ci publish-artifacts`
- **Assinatura**:
  `cccp ci publish-artifacts [OPTIONS]`
- **Argumentos e Flags**:
  - `-d, --dist-dir <PATH>`: Diretório onde se encontram os artefatos compilados (padrão: `dist` ou `target/release`).
  - `-t, --target <TRIPLE>`: Triple alvo de compilação (ex.: `x86_64-unknown-linux-musl`).
  - `--bucket <NAME>`: Nome do bucket R2 (padrão via env `BANCA_BUCKET` ou `banca-artifacts`).
  - `--endpoint <URL>`: Endpoint S3/R2 (padrão via env `R2_ENDPOINT` ou Cloudflare endpoint derivado de `R2_ACCOUNT_ID`).
  - `--dry-run`: Calcula checksums e gera o manifesto localmente sem realizar upload na rede.
- **Fluxo de Execução**:
  1. Identifica artefatos no diretório alvo (binários, tarballs `.tar.gz`, zip).
  2. Gera arquivos `.sha256` para cada artefato.
  3. Gera manifesto estruturado `manifest.json` com versão, data ISO-8601, hashes e lista de arquivos.
  4. Na ausência de `--dry-run`, realiza o despacho HTTP PUT / S3 autenticado via `R2_ACCESS_KEY_ID` e `R2_SECRET_ACCESS_KEY`.

#### 2.2 `cccp ci deploy-site`
- **Assinatura**:
  `cccp ci deploy-site [OPTIONS]`
- **Argumentos e Flags**:
  - `-s, --source <PATH>`: Diretório com os arquivos do site estático (padrão: `site`).
  - `-b, --branch <BRANCH>`: Nome do branch órfão de páginas (padrão: `pages`).
  - `-r, --remote <REMOTE>`: Remoto para publicação (padrão: `origin`).
  - `--cname <DOMAIN>`: Cria/sobrescreve arquivo `CNAME` na raiz do site publicado.
  - `--dry-run`: Cria a árvore e commit local no branch de páginas sem executar push.
- **Fluxo de Execução**:
  1. Valida que o diretório `--source` existe e contém conteúdo publicável.
  2. Garante banner determinístico Open Graph (`site/assets/og_preview.png`) chamando o gerador interno se ausente.
  3. Constrói árvore git órfã isolada na branch `pages` com mensagem canônica `deploy: site showcase v<VERSION>`.
  4. Na ausência de `--dry-run`, executa push atômico para o remote configurado.

### 3. Critérios de Aceitação
- Subcomandos `cccp ci publish-artifacts` e `cccp ci deploy-site` registrados e acessíveis via CLI.
- `--dry-run` suportado em ambos os comandos com geração fiel de arquivos locais e hash SHA-256.
- Tratamento de erro robusto caso diretórios de entrada não existam ou faltem credenciais de esteira.
- Testes automatizados cobrindo geração de manifestos SHA-256, criação de árvore de site e flags de linha de comando.
