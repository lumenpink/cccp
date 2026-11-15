# Especificação Técnica — Ticket #177
# Migração da Governança do Repositório CCCP para GOST v3.0.0 e Lilya v2.1.0

- **Ticket:** #177
- **Tipo:** `chore(profile)`
- **Escopo:** Transição normativa do Maestro CCCP para o padrão GOST v3.0.0 e Perfil Lilya v2.1.0.

---

## 1. Contexto e Motivação

O Maestro CCCP implementou no Ticket #176 o algoritmo linear de resolução de upgrade de perfis com `valid_since` e `replaces`. Com a Metanorma GOST v3.0.0 estabilizada e o Perfil Lilya v2.1.0 publicado com seu respectivo artefato `UPGRADE.md` em `https://lilya.2lp.in/2.1.0/UPGRADE.md`, o próprio repositório `cccp` deve ser migrado para o novo padrão.

Esta migração consolida os seguintes marcos arquiteturais:
1. Purgar o arquivo legado `GOSTVER` (eliminando manumissão de formatos obsoletos).
2. Adotar a configuração canônica `.rules/profile.toml` apontando para `url = "https://lilya.2lp.in"` e `version = "2.1.0"`.
3. Manter o laudo de migração gerado em `.gost/UPGRADE.md`.
4. Compilar o livro de regras `.gost/GOST.md` refletindo as regras ativas de Lilya v2.1.0.
5. Garantir conformidade estrita (`cccp check --strict`).

---

## 2. Desenho da Execução

```text
[Fase 4: Red]
  └── Script de teste / verificação apontando falha na ausência de .rules/profile.toml e persistência de GOSTVER.

[Fase 5: Green]
  ├── Execução de: cccp upgrade 2.1.0 (ou binário de depuração compilado)
  ├── Remoção de GOSTVER (git rm GOSTVER)
  ├── Atualização / compilação de .gost/GOST.md
  └── Validação de conformidade com cccp check --strict.

[Fase 6: Refactor]
  └── Limpeza de referências residuais a GOSTVER nos testes ou scripts caso existam.

[Fase 7: Mutation]
  └── Validação de robustez e integridade da árvore .gost e .rules.

[Fase 8: Quality Gate]
  └── Linters e testes nativos rápidos executados sem desvios.

[Fase 9: Close]
  └── Laudo de conclusão registrado no CHANGELOG.md e fechamento do ciclo.
```

---

## 3. Árvore de Decisões e Alternativas Descartadas (Para a Natasha)

- **Alternativa Descartada 1: Manter `GOSTVER` em paralelo com `.rules/profile.toml` como fallback.**
  - *Motivo do Descarte:* Fere o princípio constitucional de Fail-Fast (Seção 2.3 da GOST) e a Unicidade de Interface (Seção 2.4). Manter múltiplos arquivos declaratórios introduz ambiguidade sobre qual é a fonte da verdade e acumula débito técnico para o futuro.
- **Alternativa Descartada 2: Editar manualmente `.rules/profile.toml` sem rodar `cccp upgrade`.**
  - *Motivo do Descarte:* Desperdiça a oportunidade de validar o resolvedor linear em um cenário de produção real do ecossistema e não geraria o histórico formal em `.gost/UPGRADE.md`.

---

## 4. Critérios de Aceite e Matriz de Verificação

1. `.rules/profile.toml` existente contendo versão `2.1.0` e URL `https://lilya.2lp.in`.
2. `.gost/UPGRADE.md` gerado documentando a transição `2.0.0 ➔ 2.1.0`.
3. `GOSTVER` completamente removido do repositório.
4. `.gost/GOST.md` sincronizado e compilado com sucesso.
5. `cccp check --strict` aprovado com status de sucesso.

---

## 5. Resultados do Quality Gate (Fase 8)
- `cargo fmt --check`: Aprovado (0 inconsistências de estilo).
- `cargo clippy --test repo_upgrade_governance_test -- -D warnings`: Aprovado (0 warnings/erros).
- `cargo test --test repo_upgrade_governance_test`: Aprovado (4/4 testes aprovados).
- `cccp check --strict`: Aprovado com 100% de conformidade (24/24 regras).

