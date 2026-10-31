# Especificação Técnica: Ticket #111
## fix(cli): retornar à edição após falha no envio ao Gemini ou criação na forja no Divan

### 1. Contexto e Problema
Durante o ciclo interativo do assistente `cccp ticket divan` (ou `cccp divan`):
1. Caso a inferência de IA (`client.triage()`) falhe por erro na API do Google Gemini (ex.: HTTP 401 Unauthorized, rate limit, timeout ou erro de rede) ou Ollama/LocalAI, o comando atualmente aborta imediatamente com `ExitCode::FAILURE`. O operador perde a oportunidade de ajustar o prompt recém-digitado no editor interativo.
2. Em nenhum caso deve ocorrer envio direto cego ou tentativas automáticas sem confirmação expressa do operador.
3. Se a criação de um ticket individual na forja remota falhar (ex.: token expirado, erro 403, indisponibilidade da forja), o operador deve poder reabrir o editor com os dados do ticket (`[r] Retry in editor`) para retificar o conteúdo antes de tentar novamente, sem que o sistema pule silenciosamente o ticket por padrão.

### 2. Requisitos Técnicos
1. **Loop de Triagem com Retorno ao Editor (`src/main.rs`)**:
   - Em caso de falha em `client.triage(&prompt, ...)` durante o fluxo interativo, capturar o erro, exibir o diagnóstico ao usuário e oferecer o menu de recuperação `[e] Reopen in editor to adjust prompt  [q] Quit`.
   - Se o operador escolher `[e]` (ou pressionar Enter/default), o editor deve ser reaberto preenchido com o texto do prompt anterior (`last_prompt_text`), evitando perda de digitação.
   - Em ambientes não-interativos (sem terminal), preservar a saída com erro (`ExitCode::FAILURE`).
2. **Tratamento de Falha na Criação na Forja**:
   - Diante de falha em `forge_client.create_issue()`, o menu de opções deve priorizar `[r] Retry in editor` por padrão em vez de pular o ticket (`[s] Skip ticket`).
   - Ao selecionar `[r]`, o editor do CCCP deve reabrir imediatamente com o título e corpo atuais do ticket para revisão e retentativa de envio.

### 3. Critérios de Aceitação
- Falha na API Gemini em sessão interativa do Divan não encerra o processo abruptamente; permite reabrir o editor com o texto anterior preservado.
- Nenhuma criação direta de tickets ocorre automaticamente sem a anuência expressa do usuário.
- Falha na API da forja permite reabrir o editor para o ticket em questão mantendo título e descrição editados.
- Testes automatizados cobrindo o fluxo de erro e recuperação.
