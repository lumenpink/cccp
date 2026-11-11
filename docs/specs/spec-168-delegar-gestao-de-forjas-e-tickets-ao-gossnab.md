# Especificação Técnica: Delegar Gestão de Forjas e Tickets ao Gossnab (Ticket #168)

## 1. Contexto e Motivação
O `cccp` acumulava integrações diretas com APIs remotas de forjas Git (Forgejo/GitHub/Codeberg), necessitando de credenciais de privilégio elevado em `.cccprc` e tornando o fechamento de tickets durante o `cccp merge` acoplado à conectividade de rede e dependente de chamadas síncronas manuais.

Com o `gossnab` consolidado como o ecossistema e autoridade de forjas (`gossnab ticket`), o `cccp` delega essa atribuição, concentrando-se em sua missão de maestro de desenvolvimento local.

## 2. Decisões Arquiteturais e Alternativas Descartadas (Para a Natasha)
- **Fechamento Autônomo e Desacoplado no Merge:**
  - *Decisão:* No `cccp merge`, tentar delegar o fechamento via `gossnab ticket close` quando o utilitário estiver disponível, mas sem bloquear a operação em caso de falha de rede; a forja remota fechará automaticamente as issues associadas através da convenção canônica `(closes #ID)` no corpo do commit integrado durante o `cccp push`.
  - *Alternativa descartada:* Falhar o `cccp merge` se a API da forja remota não responder. Descartada pois quebra o fluxo de trabalho offline e impede integrações locais em ambientes sem conexão.
  - *Alternativa descartada:* Remover imediatamente o subcomando `cccp ticket`. Descartada em favor de manter a CLI retrocompatível apontando e delegando para a autoridade canônica `gossnab`.

## 3. Resultados da Execução
- Desacoplamento de `close_tickets` em `src/tickets.rs` com suporte a execução delegada e resiliência offline.
- Atualização semântica das descrições de `Ticket` e `Forge` em `src/cli.rs`.
- Validação completa com suíte de testes passando em `tests/tickets_test.rs`.
