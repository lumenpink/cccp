# Especificação Técnica — Ticket #79
## feat(ai): precedencia de CCCP_GEMINI_TOKEN sobre GEMINI_TOKEN

### 1. Contexto e Motivação
O CCCP integra recursos de IA generativa para triagem automatizada de tickets, desdobramento de demandas e apoio via bot Divan (`cccp ticket divan`).
Atualmente, a resolução de credenciais para a API do Google Gemini depende de variáveis genéricas como `GEMINI_API_KEY` ou `GOOGLE_API_KEY`.
Em ambientes onde desenvolvedoras ou servidores executam múltiplos agentes e ferramentas que consomem o Gemini, o compartilhamento da mesma variável de ambiente pode ocasionar colisões de cotas, falta de granularidade de acesso e confusão de faturamento.

Para assegurar isolamento seguro e independência de credenciais no ecossistema CCCP/Gosplan, este ticket introduz a resolução com precedência explícita via `CCCP_GEMINI_TOKEN` e `CCCP_GEMINI_API_KEY`.

### 2. Regra de Precedência Normativa

A resolução da chave de API do Gemini deve seguir rigorosamente a seguinte ordem de prioridade decrescente:

1. **Específica do CCCP (Prioridade Máxima)**:
   - `CCCP_GEMINI_TOKEN`
   - `CCCP_GEMINI_API_KEY`
2. **Genérica do Gemini (Prioridade Secundária)**:
   - `GEMINI_TOKEN`
   - `GEMINI_API_KEY`
   - `gemini_api_key`
3. **Fallback Legado da Google (Prioridade Terciária)**:
   - `GOOGLE_API_KEY`
   - `google_api_key`

Qualquer valor encontrado deve ter espaços em branco limpos (`trim()`). Se a string resultante for vazia, deve ser ignorada em favor do próximo nível de precedência.

### 3. Componentes Afetados

#### 3.1 `src/config.rs`
- O método `Config::merge_from_env()` deve verificar a cadeia de variáveis respeitando a precedência acima e atribuir à `self.gemini_api_key`.

#### 3.2 `src/ai.rs`
- Função auxiliar pública `pub fn resolve_gemini_api_key(config: &Config) -> Option<String>`.
- O construtor `AiClient::new()` deve utilizar a cadeia hierárquica completa se `config.gemini_api_key` não estiver preenchida.
- As mensagens diagnósticas de erro em `check_access()` e `triage_gemini()` devem orientar sobre a existência de `CCCP_GEMINI_TOKEN` como opção recomendada para isolamento.

### 4. Critérios de Aceitação
- Se `CCCP_GEMINI_TOKEN` estiver definido, ele deve ser escolhido mesmo quando `GEMINI_API_KEY` e `GOOGLE_API_KEY` também estiverem presentes no ambiente.
- Se `CCCP_GEMINI_TOKEN` não estiver definido, `GEMINI_TOKEN` ou `GEMINI_API_KEY` deve prevalecer sobre `GOOGLE_API_KEY`.
- Se apenas `GOOGLE_API_KEY` estiver definida, ela deve ser utilizada como fallback.
- Valores em branco ou apenas com espaços devem ser descartados graciosamente.
- Cobertura de testes unitários e de integração validando cada nível de precedência de forma isolada.
