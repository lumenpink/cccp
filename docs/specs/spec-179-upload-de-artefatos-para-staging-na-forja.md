# Especificação — Ticket #179: Upload de Artefatos para Staging na Forja (`cccp package`)

## 1. Contexto e Motivação
Nas esteiras de CI da Gambi e no desenvolvimento local, a publicação de artefatos efêmeros no Generic Package Registry da forja atualmente depende de comandos `curl` manuais, formatação de URLs proprietárias e injeção direta de tokens.
O CCCP, sendo o utilitário universal de ciclo de vida do desenvolvedor e do CI, deve fornecer uma interface canônica e declarativa: `cccp package upload <alvo>`.

## 2. Escopo e Funcionalidades
1. **Subcomando:**
   `cccp package upload <CAMINHO> [--app <NOME>] [--version <VERSAO>]`
2. **Autodetecção de Contexto:**
   - **Forja, Host e Owner:** Detectados automaticamente a partir do remote `origin` via `detect_forge_from_repo`.
   - **App / Pacote:** Caso `--app` não seja informado, assume o nome do repositório (`info.repo`).
   - **Versão:** Caso `--version` não seja informada, resolve através do arquivo `VERSION` ou `Cargo.toml`.
   - **Token:** Resolvido automaticamente via `resolve_token_for_host` (`FORGEJO_TOKEN`, `GAMBI_2LP_IN_TOKEN`, etc.).
3. **Tratamento de Alvo:**
   - **Arquivo único:** Envia diretamente via HTTP PUT para `/api/packages/{owner}/generic/{app}/{version}/{filename}`.
   - **Diretório (ex: `site/` ou `dist/`):** Compacta automaticamente em tarball `.tar.gz` temporário (ex: `site.tar.gz`) e realiza o upload.
4. **Resiliência e Idempotência:**
   - Confirmação de status HTTP 201 (Created) ou 200 (OK).

## 3. Árvore de Decisão Técnica (Natasha)
- **Por que `cccp package upload` no CCCP e não script bash na esteira?**
  Para eliminar código bash duplicado em dezenas de repositórios, padronizar a autenticação com a forja e manter os manifests de CI enxutos (`cccp package upload site/`).
- **Por que no CCCP e não no Kombinat?**
  O Kombinat é o agente de CD (egress/consumo), confinado no servidor ou contêiner de orquestração. O CCCP é o utilitário de CI (ingress/produção), já instalado e presente nos runners de compilação da Gambi.

## 4. Laudo de Resolução (Fase 9)
- **Status:** Concluído com 100% de conformidade GOST.
- **Implementação:** Módulo `src/package.rs` com funções `upload_package`, `package_dir_to_tarball`, `resolve_package_upload_config`.
- **CLI:** Subcomando `cccp package upload <CAMINHO> [--app] [--version] [--filename]`.
- **Verificação:** Cobertura de testes unitários e de integração em `tests/package_spec.rs`, `cargo clippy --all-targets -- -D warnings` limpo e `cccp check --strict` aprovado com índice de 100%.
