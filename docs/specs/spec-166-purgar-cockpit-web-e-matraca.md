# Especificação Técnica: Purgar Cockpit Web e Matraca (Ticket #166)

## 1. Contexto e Motivação
Historicamente, o `cccp` acumulou módulos experimentais que fugiam do escopo de uma ferramenta de automação de fluxo de engenharia e Conventional Commits:
- `cockpit`: um servidor HTTP/WebSocket local com interface gráfica web (SPA) para visualização de tickets, logs e status.
- `matraca`: um cliente e worker de mensageria FIFO em disco (`/tmp/.cccp_spool`) acoplado ao cockpit para notificações e web REPL assíncrono.

Ambos os módulos constituíam severo *overengineering*, aumentando a superfície de ataque, complexidade de manutenção, tempo de compilação e risco de quebras de compilação estática (ex.: dependências de rede e threads). Além disso, nunca foram adotados no fluxo de trabalho ativo.

## 2. Decisões Arquiteturais e Alternativas Descartadas (Para a Natasha)
- **Expurgo Total de Cockpit e Matraca:**
  - *Decisão:* Remover integralmente `src/web.rs`, `src/matraca.rs`, subcomandos `cockpit` e `spool` de `src/cli.rs`, branches de execução em `src/main.rs`, e suítes de testes associadas (`tests/web_test.rs`, `tests/matraca_test.rs`, e casos em `tests/cli_test.rs`).
  - *Alternativa descartada:* Manter os subcomandos como `deprecated` com mensagem de aviso. Descartada porque ninguém utiliza tais funcionalidades e manter código morto no binário apenas preserva dívida técnica e atrasa a simplificação para Rust estático puro.
  - *Alternativa descartada:* Mover para um repositório separado dedicado a web dashboard. Descartada por ausência de caso de uso real; se no futuro uma interface gráfica for necessária, ela deverá ser construída como um consumidor desacoplado das APIs e eventos de forja.

## 3. Resultados da Execução
- Remoção de mais de 1.500 linhas de código morto e testes legados.
- Eliminação dos subcomandos `cockpit` e `spool` da CLI.
- Resolução de concorrência em variáveis de ambiente nos testes de configuração através de mutex dedicado em `tests/config_test.rs`.
