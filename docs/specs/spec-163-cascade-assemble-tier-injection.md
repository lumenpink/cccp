# Especificação Técnica - Ticket #163: Injeção de Tier Hierárquico no Assemble em Cascata

## 1. Contexto e Motivação
Com a implementação do suporte a `tier` no motor do Lunokhod (Ticket #12), a ordem de execução da auditoria passa a ser governada pelo nível hierárquico da regra (`tier ASC`).

Para que todas as regras compiladas a partir de normas remotas reflitam automaticamente essa hierarquia sem exigir alterações manuais em todas as normas históricas, o compilador do Maestro (`assemble_rules_specs`) deve injetar o `tier` derivado da posição de cada camada na árvore de herança em cascata.

## 2. Decisão Arquitetural e Rastreabilidade (Para a Natasha)

### Ordem da Árvore de Herança
A função `resolve_cascading_chain` resolve as camadas da base/raiz até a folha mais especializada:
* Camada 0 (Fundação/Constitucional, ex: GOST) -> `tier = 1`
* Camada 1 (Metodologia/Processo, ex: GOSPLAN) -> `tier = 2`
* Camada 2 (Perfil Organizacional, ex: KOLLONTAI) -> `tier = 3`
* Camada 3 (Perfil Operacional/Fábrica, ex: LILYA) -> `tier = 4`
* Camada Local (`custom-rules.toml`) -> `tier = 99` (ou última prioridade)

### Injeção de Metadados no `RULES.toml`
* Durante a iteração de cada camada na função `assemble_rules_specs`:
  - Se a regra individual (`[rules.CODE]`) **não** possuir o campo `tier` definido, o Maestro injeta `tier = (index + 1)`.
  - Se a regra já definir explicitamente um `tier`, seu valor nativo é preservado.
  - Para regras customizadas locais (`custom-rules.toml`), caso não tenham `tier`, recebem `tier = 99`.

### Consumo do Lunokhod Atualizado
* Atualizar a dependência `lunokhod` no `Cargo.toml` do `cccp` para a versão `0.1.0-dev.51+20261111.0fa4b41` (publicada no registry Gambi).

## 3. Plano de Fases (GOST DD-TDD)
- **Fase 3**: Especificação viva registrada neste documento.
- **Fase 4**: Teste vermelho em `tests/profile_test.rs` validando que `assemble_rules_specs` injeta `tier = 1`, `tier = 2`, etc. no `.rules/RULES.toml` gerado.
- **Fase 5**: Implementação da injeção de tier em `src/profile.rs` e bump de dependência do Lunokhod.
- **Fase 7**: Testes de mutação cobrindo preservação de tiers pré-existentes e regras customizadas locais.
- **Fase 8**: Linters (`cargo fmt`, `cargo clippy`), testes unitários e de integração.
- **Fase 9**: Fechamento do ticket #163, merge na `main` e push.

## 4. Laudo de Validação e Quality Gate (Fase 8)
- `cargo fmt --check`: 100% aprovado.
- `cargo clippy --all-targets -- -D warnings`: 0 warnings, 0 erros.
- `cargo test --test assemble_rules_cascade_test`: 5 testes aprovados, validando injeção automática de tier por camada e preservação de tiers explícitos.

