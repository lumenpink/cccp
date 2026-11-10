# Especificação do Ticket #130: Suporte Universal à Flag `--force` nos Comandos Operacionais

## Objetivo
Garantir paridade e disponibilidade da flag `--force` (e `-f`) em todos os comandos operacionais do Maestro CCCP, completando a cobertura em `cccp push` para permitir o envio forçado com bypass explícito de verificações de paridade quando solicitado conscientemente pelo desenvolvedor.

---

## 1. Contexto e Problema
A norma GOST preconiza automações rigorosas com verificações obrigatórias por padrão. No entanto, em cenários de emergência, migração histórica ou reescrita intencional de histórico em branches de trabalho isoladas, o operador deve poder ignorar as validações automáticas utilizando a flag explícita `--force` / `-f`.

Comandos que já dispunham de `--force`:
- `cccp commit -f`: bypass de guards e validação de formato.
- `cccp merge -f`: bypass da exigência de tag canônica de fechamento.
- `cccp hook -f`: sobrescrita forçada de hooks sem confirmação interativa.
- `cccp sync -f`: sincronização e sobrescrita compulsória.
- `cccp check -f`: aprovação com waiver/justificativa.

Faltava paridade estrita no comando `cccp push`, que não disponibilizava `--force` para contornar divergências locais deliberadas.

---

## 2. Requisitos Técnicos
1. **Definição em `PushArgs`:**
   - Adicionar o argumento `#[arg(short = 'f', long = "force")] pub force: bool`.
2. **Comportamento em `handle_push`:**
   - Se `push.force == true`:
     - Emite aviso visual no terminal: `! Flag --force detectada: ignorando verificações prévias de paridade.`
     - Pula a execução de `check_and_reconcile_parity`.
     - Adiciona a flag `--force` à invocação do `git push` executado sob a guarda de `CCCP_MAESTRO_ACTIVE=1`.
   - Se `push.force == false`:
     - Mantém integralmente o fluxo padrão de auditoria de paridade (`check_and_reconcile_parity`) e bloqueios normativos.

---

## 3. Árvore de Decisões e Registro Arquitetural (para a Natasha)

1. **Aviso visual obrigatório:**
   - Toda execução com `--force` deve deixar rastro claro no `stdout`/`stderr` de que as salvaguardas foram dispensadas a pedido explícito do usuário, evitando que operações destrutivas passem despercebidas.
2. **Não dispensar `CCCP_MAESTRO_ACTIVE`:**
   - Mesmo com `--force`, a variável de ambiente do Maestro continua ativa para que o hook de `pre-push` saiba que a invocação veio da ferramenta oficial e não de um comando cru sem contexto.

---

## 4. Status do Ciclo
- **Fase 9 Concluída:** Flag --force / -f adicionada ao cccp push com bypass de paridade, aviso visual e repasse de flag ao git push sob a guarda do Maestro. Testes de unidade, mutação e quality gate 100% aprovados.
