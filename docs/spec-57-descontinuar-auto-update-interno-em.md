# Especificação Técnica — Ticket #57

## Escopo
`refactor(update): descontinuar auto-update interno em favor da jornaleira`

## Contexto e Motivação
A CLI CCCP possuía rotina autônoma de verificação periódica de atualizações (`check_auto_update`) executada no startup (`src/main.rs`). Na arquitetura do ecossistema Kollontai / 2lp.in, o ciclo de vida, compilação e entrega de ferramentas é centralizado no gestor de pacotes soberano **Jornaleira**. O auto-update embutido na CLI viola a previsibilidade de execução local, introduz latência potencial de rede por tentativas de conexão não solicitadas e duplica a responsabilidade da Jornaleira.

## Requisitos
1. **Inicialização Limpa e Offline**:
   - Eliminar a chamada a `check_auto_update` em `src/main.rs`. O arranque da ferramenta não deve disparar nenhuma requisição HTTP em background.
2. **Descontinuação da Checagem em Background**:
   - Tornar `check_auto_update` em `src/update.rs` uma função no-op deprecada que retorna `Ok(None)` de forma determinística e sem I/O de rede ou disco.
3. **Comandos Explícitos e Redirecionamento à Jornaleira**:
   - O comando explícito `cccp update` ou `cccp check-update`, se invocado, deve orientar que as atualizações são governadas centralmente pela Jornaleira (`jornaleira upgrade cccp`), sem mutação forçada de binários locais sem supervisão.
4. **Validação de Testes**:
   - Manter a compatibilidade da suíte de testes de `update`, garantindo que cenários de execução rotineira permaneçam 100% offline.
