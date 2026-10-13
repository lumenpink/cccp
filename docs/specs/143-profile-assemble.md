# Especificação Técnica: Montador de Perfis em Cascata (`cccp profile assemble`)

**Ticket Associado:** #143  
**Branch:** `feat/143-profile-assemble`  
**Norma:** GOST 1.4.0 (Perfil Lumen / Herança Kollontai)

## 1. Objetivo
Implementar o comando `cccp profile assemble` e integrá-lo ao `cccp sync` para compilar o arquivo canônico e único `.gost/GOST.md` resolvendo em ordem de precedência:
1. **Norma Central GOST:** Base universal (`SPEC.md`).
2. **Cadeia em Cascata de Perfis:** Resolve recursivamente as diretivas `extends:` no frontmatter dos perfis (ex: `Lumen` -> `Kollontai` -> `GOST base`).
3. **Sobreposições Locais de Repositório (`CUSTOM.md`):** Se um arquivo `CUSTOM.md` existir na raiz do repositório, ele é anexado ao final como a autoridade máxima soberana local.

## 2. Estrutura do Documento Gerado (`.gost/GOST.md`)
O arquivo compilado congrega:
- Cabeçalho Gosplan com sumário da árvore de herança resolvida.
- Conteúdo integral de cada camada separada por divisores semânticos com indicação de precedência.
- Camada final `CUSTOM.md` (quando existente).
