# Spec #152: Consumir Lunokhod via Registry Gambi e Expurgar Path Dependency

## 1. Contexto e Motivação
A regra tática `KOLLONTAI-016` (`FORBIDDEN-PATH-DEPENDENCIES`) proíbe que manifestos de pacotes utilizem dependências declaradas via caminhos de sistema de arquivos local (`path = "..."`).
Ademais, a presença de `lunokhod = { path = "../lunokhod" }` no `Cargo.toml` do `cccp` quebra a esteira de CI do Forgejo, pois o checkout em ambiente de automação isola apenas o diretório do repositório em `/workspace/lumen/cccp`.

## 2. Escopo de Alterações
1. **`Cargo.toml`**:
   - Substituir `lunokhod = { path = "../lunokhod" }` por `lunokhod = { version = "0.1", registry = "gambi" }`.
2. **`.forgejo/workflows/ci.yml`**:
   - Adicionar o registro `[registries.gambi]` no setup de configuração do Cargo nos steps de auditoria e compilação.
   - Purgar ocorrências de termos imperialistas ("Sovereign") nos nomes das etapas.
3. **Auditoria**:
   - Garantir compilação limpa e hermética através do registro `gambi`.
