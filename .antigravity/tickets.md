# Active Tickets - cccp
> Synced at: 2026-10-31 14:41:22 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#111] fix(cli): retornar a edicao apos falha no envio ao gemini ou criacao na forja no divan
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/111

### Description
## Description
Atualmente, durante o fluxo interativo do comando `cccp ticket divan`, caso ocorra alguma falha na chamada à API Gemini ou durante o envio/criação do ticket na forja remota, o fluxo de recuperação pode tentar prosseguir com envio direto sem permitir a reavaliação dos dados. O comportamento desejado é que, diante de qualquer erro nessas fases, o operador retorne à etapa de edição manual (ou confirmação dos dados), permitindo ajustar o texto/parâmetros antes de nova tentativa.

## Expected Behavior & Acceptance Criteria
- [ ] Qualquer erro retornado na etapa de envio/inferência com a API Gemini deve redirecionar o usuário de volta à tela/fase de edição do prompt/relato.
- [ ] Qualquer erro retornado na etapa de criação do ticket na forja (Forgejo/Gitea/GitHub/GitLab) deve redirecionar o usuário de volta à tela/fase de edição dos dados do ticket.
- [ ] O fluxo não deve prosseguir com tentativas automáticas ou envio cego sem confirmação explícita do operador após a falha.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#110] fix(ui): implementar quebra de linha por palavra no editor de texto integrado
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/110

### Description
## Description
O editor de texto interativo/integrado quebra palavras no meio ao atingir a margem direita/final da linha, tornando o texto truncado e de difícil leitura durante a edição de descrições ou commits.

## Expected Behavior & Acceptance Criteria
- [ ] O editor deve implementar quebra de linha respeitando limites de palavras (word-wrap) ao invés de quebra abrupta por caractere.
- [ ] Palavras mais longas que a largura total da linha devem ser tratadas de forma graciosa sem travar a interface.
- [ ] Testes unitários para a rotina de quebra de linha e renderização do buffer de texto.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#109] fix(cli): aceitar opcao repo e argumentos posicionais no ticket divan sem exigir git local
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/109

### Description
## Description
Atualmente, ao executar `cccp ticket divan` fora de um repositório Git ou tentando especificar o repositório alvo via argumentos posicionais ou flag `-r` / `--repo`, o comando falha imediatamente alegando que o diretório atual não é um repositório Git válido, ignorando as opções fornecidas pelo usuário.

## Expected Behavior & Acceptance Criteria
- [ ] `cccp ticket divan` aceita `--repo <owner/repo>` ou `-r <owner/repo>` para direcionar os tickets para a forge/repo especificada sem depender do diretório atual ser um repositório Git.
- [ ] Suportar argumentos posicionais para definir repositório de destino se aplicável.
- [ ] Validação antecipada e mensagens claras de erro quando nenhuma informação de repositório for fornecida e o diretório atual não for um Git repo.
- [ ] Cobertura de testes automatizados garantindo o parsing correto de parâmetros e operação desvinculada de repositório local.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#108] fix(ai): respeitar precedencia de CCCP_GEMINI_API_KEY no parsing de arquivos de configuracao
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/108

### Description
### Descrição do Problema
O assistente de triagem (`cccp ticket divan`) falha com HTTP 401 ao comunicar com a API do Google Gemini quando `GEMINI_API_KEY` está desativada no ambiente/dotfile, mesmo havendo uma chave ativa definida em `CCCP_GEMINI_API_KEY` ou `CCCP_GEMINI_TOKEN` no `~/.env` ou `.cccprc`.

### Causa Raiz
1. Em `src/config.rs` (`set_key_value`), apenas `gemini_api_key` é mapeada explicitamente ao ler arquivos de configuração (`.env`, `~/.env`, `.cccprc`). Variantes como `cccp_gemini_api_key` e `cccp_gemini_token` caem em `self.extra`.
2. Em `src/ai.rs` (`resolve_gemini_api_key`), `config.gemini_api_key` é priorizada antes de avaliar chaves específicas do CCCP presentes em `config.extra`.
3. Em `merge_from_env()`, a função `resolve_gemini_env_key()` consulta apenas variáveis exportadas no processo (`std::env::var`), ignorando a precedência normativa de chaves CCCP quando declaradas em arquivos `.env`.

### Critérios de Aceitação
- Chaves `CCCP_GEMINI_TOKEN` e `CCCP_GEMINI_API_KEY` declaradas em arquivos de configuração (`.cccprc`, `.env`, `~/.env`) devem ter precedência estrita sobre `GEMINI_API_KEY` e `GOOGLE_API_KEY`.
- `set_key_value()` e `resolve_gemini_api_key()` devem honrar a ordem normativa do Ticket #79 independentemente da fonte (ambiente de processo ou arquivo de configuração).
- Testes unitários cobrindo cenários com arquivos `.env` e `.cccprc` contendo `CCCP_GEMINI_API_KEY` concorrendo com `GEMINI_API_KEY`.

---
