# Especificação Técnica: Migração do Endpoint do Updater e Suporte a Canais no .cccprc

**Ticket Associado:** #157  
**Branch:** `feat/157-updater-banca-channels`  
**Norma:** GOST 1.4.0  

## 1. Objetivo
Migrar os endpoints de distribuição remota do CCCP para a infraestrutura soberana da Banca (`banca.2lp.in`) e introduzir suporte a canais de distribuição configuráveis (`stable` e `nightly`) declarados no arquivo de configuração do projeto (`.cccprc` ou `.cccprs`), garantindo governança previsível sobre atualizações de binários em todo o ecossistema.

## 2. Requisitos Funcionais e Critérios de Aceitação

1. **Novos Endpoints de Distribuição (`src/update.rs`):**
   - Substituir os endpoints anteriores pelo padrão:
     - Canal `stable`: `https://banca.2lp.in/downloads/cccp/cccp-stable`
     - Canal `nightly`: `https://banca.2lp.in/downloads/cccp/cccp-nightly`
   - Metadados de versão correspondentes:
     - `stable`: `https://banca.2lp.in/downloads/cccp/version`
     - `nightly`: `https://banca.2lp.in/downloads/cccp/nightly-version`
   - O método `get_distribution_endpoints(channel: &str)` e a lógica de download em `cmd_update` devem refletir esses endereços.

2. **Configuração de Canais (`src/config.rs`):**
   - Adicionar o campo `channel: String` na estrutura `Config`, com valor padrão `"stable"`.
   - Ler a chave `channel` ou `update_channel` a partir do `.cccprc` local ou global (com fallback de arquivo para `.cccprs`).
   - Precedência de configuração para o canal:
     1. Flag explícita na CLI (`--channel <canal>`) se fornecida.
     2. Variável de ambiente `UPDATE_CHANNEL` ou `CCCP_UPDATE_CHANNEL`.
     3. Configuração do repositório local (`.cccprc` / `.cccprs`).
     4. Configuração global do usuário (`~/.cccprc` ou `$XDG_CONFIG_HOME/cccp/config`).
     5. Padrão estrito: `"stable"`.

3. **Validação Estrita:**
   - Apenas os canais `"stable"` e `"nightly"` são permitidos.
   - Qualquer valor divergente informado no `.cccprc` ou via parâmetro deve resultar em erro explícito:
     `Invalid update channel '<canal>'. Choose 'stable' or 'nightly'.`

4. **Resolução no Atualizador (`src/update.rs`):**
   - `cmd_check_update` e `cmd_update` utilizam o canal resolvido da configuração em cascata quando nenhuma flag for passada.
   - Testes unitários e de integração mockando/validando ambos os canais (`stable` e `nightly`).
