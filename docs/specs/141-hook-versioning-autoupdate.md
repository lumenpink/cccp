# Especificação Técnica: Versionamento e Auto-Atualização Segura de Git Hooks

**Ticket Associado:** #141  
**Branch:** `feat/141-versionar-auto-atualizar-hooks`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Evitar alertas constantes e espúrios sobre defasagem de versão nos git hooks (`commit-msg`, `post-commit`, etc.) quando a lógica interna do hook não sofreu alteração, e permitir auto-atualização segura dos hooks não customizados localmente.

## 2. Metadados Padronizados nos Hooks
Cada hook gerado ou gerenciado pelo CCCP deve conter metadados declarativos em seu cabeçalho:
```sh
# cccp-hook-version: {version}
# cccp-hook-template-version: 1
# cccp-hook-checksum: {sha256}
```
- `cccp-hook-version`: Versão do pacote cccp que gerou ou verificou o hook (ex.: `3.1.0`).
- `cccp-hook-template-version`: Número serial incremental da arquitetura do template do hook (ex.: `1`). Só é incrementado quando a lógica interna do script mudar.
- `cccp-hook-checksum`: Hash SHA-256 do corpo do template sem os metadados dinâmicos.

## 3. Matriz de Decisão na Verificação / Auto-Atualização
Ao executar `cccp` (via `check_hook_version` ou comando explícito `cccp hook --install`):
1. **Hook Ausente:** Criado com template canônico atual.
2. **Hook na versão atual (`template_version == CURRENT_TEMPLATE_VERSION`):** Nenhuma ação, nenhum alerta.
3. **Hook com versão de template defasada (`template_version < CURRENT_TEMPLATE_VERSION` ou sem `template_version`):**
   - Se o checksum do corpo atual coincidir com qualquer template canônico conhecido de versões anteriores:
     - **Não modificado:** O `cccp` atualiza automaticamente o hook para a versão atual e informa o usuário amigavelmente.
   - Se o checksum diferir:
     - **Modificado manualmente pelo desenvolvedor:** O `cccp` **não** sobrescreve o arquivo; preserva a customização local e emite alerta de advertência.
