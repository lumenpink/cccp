# Especificação Canônica dos Comandos `cccp changelog` e `cccp tag` (RFC-150)

- **Ticket de Referência:** #150 (`feat(cli): implementar subcomandos canonicos tag e changelog`)
- **Status:** Aprovado / Em Implementação
- **Norma GOST:** GOST-03-VERSION-SEMVER, GOST-04-TICKETS-COMMITS, Ordem Nº 227

---

## 1. Contexto e Motivação
O ecossistema GOST impõe através de seu pre-push hook que tags de release oficial nunca apontem para commits que contenham versões de desenvolvimento (`-dev`) no arquivo `VERSION`. O próprio pre-push hook orienta o operador:
> `Run 'cccp tag' to generate clean release commits and synchronize CHANGELOG.`

Entretanto, as implementações de `Commands::Tag` e `Commands::Changelog` encontravam-se como stubs no `src/main.rs`. Esta especificação estabelece os contratos e o comportamento determinístico de ambos os comandos.

---

## 2. Subcomando `cccp changelog`

### 2.1 Interface
```bash
cccp changelog [--output <FILE>] [--dry-run] [--range <REV_RANGE>]
```
- `--output <FILE>`: Caminho de destino (padrão: `CHANGELOG.md`).
- `--dry-run`: Exibe o changelog gerado no stdout sem persistir no disco.
- `--range <REV_RANGE>`: Intervalo de revisão git (ex.: `v0.3.0..HEAD`). Se omitido, identifica a tag anotada mais recente e extrai até `HEAD`. Se não houver tags anteriores, processa todo o histórico do repositório.

### 2.2 Estrutura Gerada (Keep a Changelog)
Os commits convencionais são agrupados nas seguintes seções:
- `### Features` (`feat`)
- `### Bug Fixes` (`fix`)
- `### Documentation` (`docs`)
- `### Performance` (`perf`)
- `### Refactoring` (`refactor`)
- `### Testing` (`test`)
- `### Maintenance` (`chore`, `ci`, `build`)

Se o arquivo `CHANGELOG.md` já existir, o novo bloco é inserido de forma não-destrutiva logo abaixo do cabeçalho canônico do documento.

---

## 3. Subcomando `cccp tag`

### 3.1 Interface
```bash
cccp tag [VERSION] [-t/--title <TITLE>] [-m/--message <MESSAGE>] [--no-v] [-s/--sign]
```

### 3.2 Algoritmo de Execução
1. **Determinação de Versão:**
   - Se `VERSION` for fornecido explicitamente como argumento, extrai a versão e valida com SemVer 2.0.0.
   - Se omitido, executa a predição semântica com base nos commits desde a última tag (`semver::calc_next_version`).
2. **Normalização da Versão e Nome da Tag:**
   - `clean_version`: Versão sem prefixo `v`, sem `-dev` e sem metadados de build (ex.: `0.4.0`).
   - `tag_name`: `v{clean_version}` por padrão, ou `{clean_version}` se `--no-v` for declarado.
3. **Persistência de `VERSION`:**
   - Escreve `{clean_version}\n` diretamente no arquivo `VERSION` da raiz do repositório.
4. **Atualização de `CHANGELOG.md`:**
   - Gera o bloco de notas da versão atual com data UTC (`YYYY-MM-DD`) e prepende no `CHANGELOG.md`.
5. **Commit de Release Git:**
   - Executa `git add VERSION CHANGELOG.md`.
   - Executa `git commit -m "chore(release): v{clean_version}"` (ou título customizado).
6. **Criação da Tag Git:**
   - Constrói mensagem de anotação com título e data.
   - Se `-s/--sign` ou se `user.signingkey` estiver configurado no git, adiciona a flag `-s`.
   - Executa `git tag -a <tag_name> -m "<mensagem>"` (ou `git tag -s`).
7. **Saída Formatada:**
   - Imprime banner estilizado Gosplan confirmando a versão, tag gerada e instrução de push.
