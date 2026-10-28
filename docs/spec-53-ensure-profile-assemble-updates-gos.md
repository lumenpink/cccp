# Especificação Técnica — Ticket #53
## fix(core): ensure profile assemble updates gost and gostver across repositories

### 1. Contexto e Problema
Durante a execução do comando de montagem de especificação (`cccp profile assemble` e `cccp repo assemble`), os repositórios gerenciados pelo catálogo e diretórios locais compilam o `.gost/GOST.md`, porém não sincronizam nem atualizam os artefatos GOST subjacentes (como especificações de linguagem e regras) nem garantem a atualização e consistência do arquivo `GOSTVER` com a versão canônica ativa do GOST (ex.: `2.0.0`) nos metadados do repositório.

### 2. Critérios de Aceitação
1. **Atualização do `GOSTVER`**: Durante `profile assemble` (seja individual, por catálogo ou em lote `--all`), o arquivo `GOSTVER` do repositório deve ter sua versão canônica atualizada e sincronizada (padrão `DEFAULT_GOST_VERSION`, ex.: `2.0.0`).
2. **Sincronização de Artefatos em `.gost/`**: O fluxo de montagem deve verificar e sincronizar os recursos normativos GOST em `.gost/` (especificações de linguagem em `.gost/languages/`, `rules.json` e manifesto de metadados se aplicável).
3. **Tolerância a Falhas Parciais**: Em operações em lote (`batch_assemble_gost_repos`), falhas parciais em repositórios isolados não interrompem nem corrompem os repositórios íntegros.
4. **Auto-commit**: Quando `auto_commit` estiver ativo, modificações em `GOSTVER` e `.gost/` devem ser adicionadas ao stage e commitadas com mensagem semântica `docs(gost): atualizar especificacao gost compilada`.
5. **Rigor e Validação**: Suítes de testes unitários e de integração comprovando a atualização de `GOSTVER` e sincronização de `.gost/` em repositórios individuais e múltiplos.

### 3. Arquitetura da Solução
- `src/profile.rs`:
  - `pub fn update_gostver_version(root: &Path, version: &str) -> io::Result<()>`: Atualiza a chave `GOST_VERSION` no arquivo `GOSTVER` preservando `GOST_PROFILE_URL` e `GOST_LANGUAGES`.
  - Integrar a sincronização dos artefatos normativos (`sync_gost_specs`) dentro do pipeline de `assemble_gost_specs` ou como etapa coordenada de `assemble_repo` e `batch_assemble_gost_repos`.
- `src/catalog.rs`:
  - `assemble_repo`: Atualizar `GOSTVER` e artefatos GOST além de compilar `.gost/GOST.md`. Adicionar ao auto-commit eventuais alterações no `GOSTVER` e `.gost/`.
  - `batch_assemble_gost_repos`: Propagar a atualização e sincronização em lote sobre todos os repositórios catalogados válidos.
