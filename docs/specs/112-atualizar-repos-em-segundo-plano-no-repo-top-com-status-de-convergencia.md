# Especificação Técnica: Ticket #112
## feat(cli): atualizar repos em segundo plano no repo top com status de convergência

### 1. Contexto e Problema
O painel interativo `cccp repo top` é uma ferramenta central de observabilidade para os repositórios gerenciados pelo catálogo Gosplan.
Entretanto, a exibição de commits pendentes de envio upstream (`clean [↑n]`) e status de CI pode ficar defasada em relação aos servidores remotos (por exemplo, após um `git push` executado por outro processo ou esteiras finalizadas na forja), exigindo reinicialização manual do comando.
Além disso, operações síncronas de rede e I/O travavam o loop de eventos da interface TUI, prejudicando a fluidez da navegação com teclas de seta e atalhos.

### 2. Requisitos Técnicos
1. **Thread de Sincronização em Segundo Plano Não-Bloqueante (`src/repo_top.rs`)**:
   - Disparar a verificação e atualização periódica em thread separada (`std::thread::spawn`), mantendo a navegação do usuário (scroll, filtros, alternância de ordenação e saída) responsiva e sem travamentos.
   - Para cada repositório com diretório local configurado, atualizar os refs remotos via git e reinspencionar o status local (`inspect_local_status`), refletindo a convergência de commits locais com o remote (ex.: reclassificando `clean [↑n]` para `clean`).
   - Consultar simultaneamente o status de esteiras de CI na API da forja associada.
2. **Indicador de Atividade (Spinner) e Status de Convergência no Cabeçalho**:
   - `RepoTopState` recebe os campos:
     - `is_syncing: bool`
     - `has_converged: bool`
     - `spinner_frame: usize`
   - Durante a sincronização ativa, o cabeçalho exibe um spinner discreto animado (ex.: `⠋ sincronizando`).
   - Ao término da checagem de todos os repositórios monitorados, o cabeçalho exibe o indicador de convergência (ex.: `✓ convergente`).
3. **Loop de Eventos Reativo**:
   - Enquanto `is_syncing` estiver ativo, o tempo de poll da interface se reduz para permitir avanço suave do spinner (~100ms) sem onerar CPU.
   - Comunicação via canal `mpsc` com eventos estruturados (`SyncStarted`, `RepoSynced`, `SyncCompleted`).

### 3. Critérios de Aceitação
- `render_top_view` exibe o spinner quando `state.is_syncing == true`.
- `render_top_view` exibe o indicador `✓ convergente` quando `state.has_converged == true` e `!state.is_syncing`.
- O status de commits pendentes de push reflete o estado remoto real após sincronização de background.
- Teclas de navegação da TUI permanecem 100% responsivas durante o ciclo de background sync.
- Testes automatizados cobrindo a renderização do spinner e badge de convergência, e recepção de eventos de sincronização.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**: Implementada sincronização periódica de repositórios em segundo plano em thread não-bloqueante no repo top, com spinner discreto de progresso e indicador visual de convergência no cabeçalho.
- **Encerramento**: Validado em conformidade com as 9 fases GOST DD-TDD.
- **Ciclo**: Concluído e pronto para integração.


