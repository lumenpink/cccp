# Especificação — Ticket #181: Manifesto Declarativo de Entrega no CCCP

## 1. Contexto e Motivação
Conforme estabelecido na arquitetura de Continuous Deployment do ecossistema, o Kombinat opera lendo o manifesto declarativo `.gost/delivery.toml` com convenção de presença ativa.
O CCCP, sendo a ferramenta de linha de comando central da fábrica, produz binários executáveis estáticos (`cccp`) que devem ser despachados para a Banca (Cloudflare R2) e disponibilizados para download e auto-atualização (`jornaleira` / `cccp update`).

## 2. Escopo Técnico
1. **Manifesto `.gost/delivery.toml`:**
   Declarar a seção `[binaries]` ativa:
   ```toml
   # Manifesto Declarativo de Continuous Deployment (CD) — CCCP
   [binaries]
   target = "banca"
   ```
2. **Validação:**
   - Garantir que `.gost/delivery.toml` seja reconhecido pelo motor de delivery e catálogo do ecossistema sem erros.

## 3. Árvore de Decisão Técnica (Natasha)
- **Por que apenas `[binaries]`?**
  O CCCP não possui APK mobile (`[mobile]`) nem documentação em site separado (`[sites]` usa repositório dedicado ou vitrine central). O princípio da presença ativa dispensa declarar seções desativadas.
