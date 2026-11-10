# Especificação do Ticket #137: Repasse de Git Args e Envio de Tags no `cccp push`

## Objetivo
Corrigir o comportamento de `handle_push` no comando `cccp push` para que argumentos explícitos (`git_args`), como tags de release (`origin v1.0.0`) ou `--tags`, sejam devidamente encaminhados ao comando `git push` mesmo quando o tracking branch de commits estiver em paridade (`ahead == 0`). Além disso, alinhar a instrução pós-criação de release tag em `cccp tag` para recomendar comandos coordenados pelo Maestro (`cccp push`).

---

## 1. Contexto e Problema Identificado
Atualmente, `handle_push` em `src/push.rs` invoca `check_and_reconcile_parity`. Se o repositório estiver sincronizado com o upstream remoto (`ahead == 0` e `behind == 0`), a função retorna `Ok(ParityAction::AlreadyUpToDate)`.
Nesse ponto, `handle_push` encerra imediatamente com `ExitCode::SUCCESS`, emitindo *"Tudo sincronizado com o remote (nada a enviar)."*.

Isso acarreta dois bloqueios operacionais críticos:
1. **Envio de tags impedido:** Quando uma nova tag de release é gerada via `cccp tag`, a branch já está limpa e no mesmo commit do remoto. O usuário tenta rodar `cccp push origin v1.0.0` ou `cccp push --tags`, mas o Maestro aborta antecipadamente sem executar o `git push`.
2. **Incompatibilidade com o hook de pré-envio:** O hook `pre-push` do CCCP bloqueia execuções manuais de `git push origin <tag>` sem a variável `CCCP_MAESTRO_ACTIVE=1`, gerando um impasse no qual o desenvolvedor fica impossibilitado de enviar a tag por vias canônicas.

---

## 2. Requisitos Técnicos

1. **Condição de Saída Rápida em `AlreadyUpToDate`:**
   - Se `ParityAction::AlreadyUpToDate` for retornado:
     - Se `push.git_args.is_empty()`, encerra antecipadamente com `ExitCode::SUCCESS` ("Tudo sincronizado com o remote (nada a enviar).").
     - Se `!push.git_args.is_empty()`, NÃO encerra antecipadamente. Emite mensagem informativa indicando que a branch está em paridade e prossegue com a execução do `git push` repassando os argumentos especificados sob a variável `CCCP_MAESTRO_ACTIVE=1`.
2. **Alinhamento da Mensagem de Instrução em `cccp tag`:**
   - Em `src/main.rs`, substituir a instrução legada:
     ```text
     git push origin main && git push origin <tag>
     ```
     por:
     ```text
     cccp push && cccp push origin <tag>
     ```
     assegurando conformidade com a proibição estrita de invocações diretas ao `git push`.

---

## 3. Árvore de Decisões e Registro Arquitetural (para a Natasha)

1. **Por que não alterar o retorno de `check_and_reconcile_parity`?**
   - *Contexto:* `check_and_reconcile_parity` afere estritamente a relação entre branches locais e remotas (`ahead`/`behind`).
   - *Decisão:* Manter a responsabilidade de `check_and_reconcile_parity` pura. A decisão de bypassar a saída prematura cabe a `handle_push`, que tem visibilidade das opções da linha de comando (`PushArgs`).
   - *Alternativa Descartada:* Misturar checagem de tags e argumentos dentro do verificador de paridade de branches causaria acoplamento desnecessário e efeitos colaterais em outras partes do sistema.

2. **Segurança do Maestro:**
   - A invocação continuará sendo feita sob a guarda de `CCCP_MAESTRO_ACTIVE=1`, garantindo que o hook `pre-push` seja atendido e não ocorram bypasses não auditados.

---

## 4. Status do Ciclo
- **Fase 9 Concluída:** Repasse de git_args em push com paridade implementado, instrução pós-release tag em cccp tag atualizada, testes e mutantes validados com sucesso.
