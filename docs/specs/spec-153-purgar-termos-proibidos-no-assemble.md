# Spec #153: Purgar Termos Proibidos na Montagem de Perfis em Cascata

## 1. Contexto e Motivação
O comando `cccp profile assemble` (`assemble_gost_specs`) compila as camadas de normas e perfis (`SPEC.md`, `profile.md`, `CUSTOM.md`) no documento canônico `.gost/GOST.md`.
Caso alguma camada externa ou cache legado de rede contenha jargões ou termos imperialistas proibidos (como "soberano", "soberania"), o documento consolidado herdaria esses termos, falhando os testes de conformidade.

## 2. Escopo
1. Em `src/profile.rs`, implementar uma higienização determinística no pipeline de geração de `assemble_gost_specs` que substitui termos imperialistas derivados de "soberan" por equivalentes canônicos ("autonomia", "canônico(a)").
2. Em `tests/assemble_rules_cascade_test.rs`, adicionar teste com mutante contendo camada com termo proibido e validar que a higienização é aplicada com sucesso.
