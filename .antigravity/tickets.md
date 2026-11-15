# Active Tickets - cccp
> Synced at: 2026-11-15 10:21:28 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#176] feat(profile): implementar resolucao linear de upgrade com valid_since e replaces
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/176

### Description
## Contexto & Motivação
A metanorma GOST v3.0.0 (Seção 2.2) e os perfis imutáveis (como Lilya 2.1.0) normatizam o Contrato UPGRADE.md e o Princípio do Encadeamento Linear de Migração.
O Maestro CCCP necessita implementar o comando de migração declarativa cccp profile upgrade (com atalho canônico de primeiro nível cccp upgrade), consumindo o UPGRADE.md, validando o corte inferior valid_since (fail-fast), percorrendo os elos replaces recursivamente até a versão instalada e concatenando os deltas em ordem cronológica ascendente.

## Requisitos
1. Adicionar subcomando cccp profile upgrade e atalho canônico de primeiro nível cccp upgrade.
2. Baixar {base_url}/{alvo}/UPGRADE.md e parsear frontmatter YAML.
3. Validar valid_since: interromper com erro fatal e instrução clara caso a versão instalada seja inferior ao piso suportado.
4. Percorrer recursivamente a cadeia replaces até encontrar a versão instalada, concatenando os deltas em .gost/UPGRADE.md em ordem ascendente.
5. Atualizar .rules/profile.toml com a versão alvo e remontar regras via assemble_rules_specs.
6. Suportar flag --dry-run / -n para simulação sem modificações em disco.

---
