# Especificação Técnica — Ticket #185
## Quebra de Recursão Infinita em Repositórios Grundnorm e Sem Perfil

### 1. Contexto e Motivação
A metanorma GOST (Fundamental) e certos repositórios estratégicos raiz não herdam de perfis operacionais ou táticos, declarando legitimamente `GOST_PROFILE_URL=""` ou omitindo `profile`.

Durante a auditoria normativa (`cccp check --strict`) nesses repositórios, observou-se uma falha crítica de estouro de pilha (*stack overflow* com `SIGABRT`), decorrente de recursão mútua infinita entre os módulos `audit` e `profile`.

### 2. Análise da Causa Raiz
1. Em `cccp::audit::read_gostver`:
   - Ao não encontrar perfil em `.gost/GOST.md` ou ler `GOST_PROFILE_URL=""` no `GOSTVER`, `prof` permanece vazio.
   - O fallback da linha 172 invoca `crate::profile::read_profile_config(root)`.
2. Em `cccp::profile::read_profile_config`:
   - Se `.rules/profile.toml` não existir, ele aciona o fallback para `get_active_profile(root)`.
   - Como `GOSTVER` existe no disco, `get_active_profile` lê o arquivo e, em seguida, a função invoca `crate::audit::read_gostver(root)` para tentar obter a versão.
3. Isso cria um ciclo fechado `read_gostver -> read_profile_config -> read_gostver`, consumindo os frames de pilha até o aborto fatal do processo.

### 3. Decisão de Engenharia
1. **Desacoplamento em `profile::read_profile_config`:**
   - Em vez de chamar `read_gostver(root)` dentro de `read_profile_config`, a versão do GOST deve ser lida diretamente do arquivo `GOSTVER` (se presente) ou assumir o padrão canônico `2.1.0` / `3.0.0` sem delegar para o `audit`.
2. **Guarda em `audit::read_gostver`:**
   - O fallback para `read_profile_config` só deve ser acionado se houver um arquivo declarativo `.rules/profile.toml` presente no repositório, ou se `GOSTVER` não tiver sido previamente inspecionado.
3. **Resiliência a `GOST_PROFILE_URL=""`:**
   - Quando `GOST_PROFILE_URL` for expressamente vazio ou `"none"`, deve ser tratado como sem perfil (`None`), sem acionar fallbacks infinitos.
