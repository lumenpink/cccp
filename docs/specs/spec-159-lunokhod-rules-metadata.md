# Especificação Técnica: Atualização do Motor Lunokhod e Metadados .rules/ (Ticket #159)

## 1. Contexto e Motivação
Com a introdução da arquitetura desacoplada `.rules/`, os repositórios passam a abrigar suas normas em `.rules/RULES.md` e `.rules/RULES.toml`, com configuração em `.rules/profile.toml`.

O `cccp check` apresentava duas inconformidades ao auditar repositórios nesse padrão:
1. Delegação ao motor `lunokhod`: o motor não buscava em `.rules/`, retornando zero verificações e acionando o erro sintético `NO-RULES-FOUND`.
2. Cabeçalho de relatório: a função `read_gostver` buscava exclusivamente pelo arquivo `GOSTVER` na raiz. Na sua ausência, o cabeçalho reportava `Norma: Não declarado | Perfil: Nenhum`, ignorando a declaração explícita de `.rules/profile.toml`.
3. Além disso, a compilação gerava apenas `.rules/RULES.toml`. Para máxima resiliência e evitar falhas por sensibilidade a caixa de qualquer ferramenta ou script legado, o `assemble_rules_specs` deve gerar tanto `RULES.toml` quanto `rules.toml`.

## 2. Decisões Arquiteturais e Alternativas Descartadas (Para a Natasha)
- **Atualização da dependência `lunokhod`:**
  - *Decisão:* Atualizar no `Cargo.lock` para a versão `0.1.0-dev.39+...` recém-publicada no registro `gambi`, que inclui suporte nativo a `.rules/RULES.toml`, `.rules/rules.toml` e o primitivo `gost.git_remotes()`.
  - *Alternativa descartada:* Reimplementar o runtime de Lua internamente no `cccp`. Descartada porque o Maestro é um orquestrador desacoplado e a responsabilidade de execução Lua pertence unicamente ao `lunokhod`.
- **Extração de Metadados em `read_gostver`:**
  - *Decisão:* Se `GOSTVER` não existir na raiz, inspecionar `.rules/profile.toml` (via `read_profile_config(root)`). Usar `config.version` como versão e `config.url` como perfil.
  - *Alternativa descartada:* Exigir que repositórios migrados mantenham um arquivo `GOSTVER` vazio ou dummy na raiz. Descartada pois violaria a diretriz de eliminar nomes acoplados e poluição na raiz.
- **Geração de alias em minúsculas (`rules.toml`):**
  - *Decisão:* Em `assemble_rules_specs`, além de gravar `.rules/RULES.toml`, gravar também `.rules/rules.toml` como cópia/alias idêntico.
  - *Alternativa descartada:* Criar symlink simbólico. Descartada para evitar problemas de portabilidade entre sistemas de arquivos ou clonagens em ambientes sem suporte a symlinks.

## 3. Plano de Fases (DD-TDD)
- **Fase 4 (Red Test):** Testes em `tests/audit_test.rs` comprovando a leitura de `.rules/profile.toml` por `read_gostver` e a resolução do cabeçalho.
- **Fase 5 (Green Implement):** Atualização do `Cargo.lock`, ajuste de `read_gostver` em `src/audit.rs` e geração de `.rules/rules.toml` em `src/profile.rs`.
- **Fase 7 (Mutation Test):** Garantir fallback gracioso caso `profile.toml` esteja ausente ou malformatado.
- **Fase 8 (Quality Gate):** Verificação de formatação e linters (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`).
- **Fase 9 (Close):** Laudo de fechamento e merge na base principal.
