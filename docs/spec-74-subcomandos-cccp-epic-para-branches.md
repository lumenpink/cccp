# Especificação Técnica — Ticket #74
## feat(governance): subcomandos cccp epic para branches persistentes e cccp inbox para quarentena de prs

### 1. Contexto e Motivação
Conforme a Seção 15 (Regra GOST-25) e Seção 1.2 da norma Kollontai 2.0, o fluxo de desenvolvimento do ecossistema soberano exige:
1. **Governança de Branches Persistentes (`epic/*`)**: Suporte a ciclos de desenvolvimento que englobam múltiplos tickets relacionados através de branches persistentes com prefixo `epic/<slug>`, permitindo integração contínua antes do merge final na branch canônica (`main`).
2. **Quarentena de Contribuições Externas (`inbox/*`)**: Mecanismo para receber, listar, revisar e aceitar Pull Requests e contribuições de espelhos públicos (GitHub, Codeberg) isoladas no namespace `refs/heads/inbox/*`, garantindo que espelhos secundários estejam protegidos com `pushurl = no_push` contra envio acidental antes de homologação.

### 2. Requisitos e Subcomandos

#### 2.1 Subcomando `cccp epic`
- `cccp epic start <slug>`:
  - Cria e faz checkout da branch persistente `epic/<slug>` a partir da branch base atual (ou `main`).
  - Valida o slug (caracteres alfanuméricos e hífens).
- `cccp epic list`:
  - Lista as branches épicas ativas locais e remotas (`epic/*`).
- `cccp epic finish <slug>`:
  - Valida working tree limpa.
  - Alterna para a branch base (`main`) e integra a branch `epic/<slug>`.

#### 2.2 Subcomando `cccp inbox`
- `cccp inbox list`:
  - Lista branches e referências de quarentena sob `refs/heads/inbox/*` e `refs/remotes/*/pr/*`.
- `cccp inbox review <id>`:
  - Exibe o histórico de commits e diff resumido da branch de quarentena `inbox/<id>`.
- `cccp inbox accept <id>`:
  - Valida conformidade e integra a branch `inbox/<id>` na branch ativa.
  - Garante a configuração de segurança da Regra GOST-25 nos remotes secundários (`codeberg`, `github`) com `pushurl = no_push`.

### 3. Critérios de Aceitação
- Comandos CLI disponíveis sob `cccp epic` e `cccp inbox`.
- Testes automatizados cobrindo `start`, `list`, `finish` para `epic`, e `list`, `review`, `accept` para `inbox`.
- Proteção estrita contra push em espelhos não autorizados.
