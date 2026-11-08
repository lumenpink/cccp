# Especificação Técnica: Implementar cccp assemble para Combinação em Cascata de rules.toml (Ticket #143)

## 1. Contexto e Motivação
A Tríade Normativa Fechada estabelece uma hierarquia estrita de 4 camadas:
`Fundamental (GOST)` -> `Estratégica (GOSPLAN)` -> `Tática (Kollontai)` -> `Operacional (Lilya)` -> `Local (Customizado)`.
O comando `cccp assemble` deve realizar a compilação determinística tanto do texto normativo unificado (`.gost/GOST.md`) quanto do manifesto consolidado de regras declarativas (`.gost/rules.toml`).

## 2. Requisitos Mandatórios
1. **Resolução em Cascata por `inherits_from.base_url`:**
   - Suporte à extração canônica de `inherits_from.base_url` do frontmatter universal, além de compatibilidade retroativa com `extends:`.
   - Busca recursiva de pacotes normativos a partir do perfil configurado em `GOSTVER` ou `.gost/config`.
2. **Compilação de `.gost/rules.toml`:**
   - Para cada camada resolvida na cadeia, carregar as regras de `rules.toml` (seja por URL pública, cache local ou repositórios locais).
   - Mesclar as seções `[rules.<ID>]` preservando precedência estrita: camadas inferiores sobrepõem definições anteriores.
   - Caso exista `custom-rules.toml` na raiz do projeto, mesclar suas regras no `.gost/rules.toml` final com precedência máxima local.
3. **Purga Terminológica:**
   - Substituição de "Camada Local Soberana" por "Camada Local Autônoma" nos comentários e sumários gerados no `.gost/GOST.md`.
