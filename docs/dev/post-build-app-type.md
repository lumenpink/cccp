# Suporte ao Escopo post-build e APP_TYPE no CCCP

## 1. Escopo post-build
O comando `cccp check` passa a suportar a flag `--scope <escopo>`:
- `cccp check --scope post-build`: Avalia exclusivamente regras declaradas com `scope = "post-build"`.
- Por padrão, `cccp check` executa regras de escopo `project`.

## 2. Leitura de APP_TYPE
O parser do `GOSTVER` extrai o valor de `APP_TYPE`:
- `web`: Webapps autorais
- `hybrid`: Aplicações híbridas (binário + webapp)
- `binary`: Utilitários binários puros
- `manifesto`: Normas e documentações puras
