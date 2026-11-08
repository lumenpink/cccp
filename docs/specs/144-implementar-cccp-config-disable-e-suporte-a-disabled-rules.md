# Especificação Técnica: Implementar cccp config disable e Suporte a DISABLED_RULES (Ticket #144)

## 1. Contexto e Motivação
A gestão de conformidade em projetos complexos ou durante migrações requer mecanismos formais e auditáveis para dispensar temporariamente determinadas regras ou forçar aprovação em situações emergenciais devidamente justificadas.

## 2. Requisitos Técnicos

### 2.1 Gestão de Regras Dispensadas via `.gost/config`
- A variável `DISABLED_RULES` em `.gost/config` declara a lista de identificadores tridigitais dispensados (ex.: `DISABLED_RULES="GOST-020, PREF-001"`).
- O comando `cccp config disable <REGRAS...>` adiciona os identificadores à variável `DISABLED_RULES` em `.gost/config` (criando o arquivo/diretório se inexistente).
- O comando `cccp config enable <REGRAS...>` remove os identificadores especificados da lista `DISABLED_RULES`.

### 2.2 Exibição de Status `[SKIP]` no `cccp check`
- Toda regra dispensada por `DISABLED_RULES` deve ser avaliada com o status `CheckStatus::Skip`.
- Na tabela do Laudo de Auditoria, regras dispensadas são exibidas com a badge `[SKIP]` e não computam como falhas ou avisos bloqueantes.

### 2.3 Flag Universal `--force` e Justificativa Obrigatória
- O comando `cccp check` e `cccp audit` suportam as flags `--force` e `--reason <JUSTIFICATIVA>`.
- O uso de `--force` sem justificativa formal (`--reason`) é rejeitado.
- Quando invocado com `--force --reason "<texto>"`, inconformidades não causam encerramento com código de erro não-zero, registrando um aviso formal no relatório.
