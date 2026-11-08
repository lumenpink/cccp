# Especificação Técnica: Runtime Lua Embutido e Primitivos da Biblioteca gost (Ticket #142)

## 1. Contexto e Motivação
A Metanorma Fundamental GOST v3.0.0 e os perfis derivados definem que as regras de governança e integridade sejam expressas declarativamente em arquivos `rules.toml` contendo validações executáveis em scripts Lua sob o namespace `gost.*`.
O motor `cccp check` (e `cccp gost check`) deve executar estes scripts em um interpretador seguro e embutido (`mlua`), mapeando retornos e mensagens de erro localizadas.

## 2. Requisitos Técnicos

### 2.1 Primitivos da Tabela `gost.*`
O ambiente de execução Lua de cada script de validação deve ter injetada a tabela global `gost` com os seguintes primitivos:
- `gost.file_exists(path: string) -> boolean`: Retorna se o arquivo ou diretório relativo à raiz do repositório existe.
- `gost.is_dir(path: string) -> boolean`: Retorna se o caminho é um diretório.
- `gost.read_file(path: string) -> string | nil`: Lê o conteúdo UTF-8 do arquivo relativo à raiz.
- `gost.count_lines(path: string) -> integer`: Conta a quantidade de linhas do arquivo.
- `gost.list_files(dir: string) -> table`: Lista arquivos e subdiretórios recursivamente a partir do diretório.
- `gost.git_branch() -> string | nil`: Retorna o nome da branch ativa no repositório.
- `gost.git_commits(limit: integer) -> table`: Retorna uma lista de tabelas representando os commits recentes (`{ hash = "...", header = "...", body = "...", author = "..." }`).
- `gost.parse_toml(content: string) -> table | nil`: Converte string TOML em tabela Lua.
- `gost.parse_json(content: string) -> table | nil`: Converte string JSON em tabela Lua.

### 2.2 Protocolo de Execução de Regras
Para cada regra definida em `rules.toml` ou `.gost/rules.toml`:
1. O motor executa o script Lua contido no campo `script`.
2. Se o retorno for `true` (ou se nenhum erro for retornado), a regra recebe status `CheckStatus::Pass`.
3. Se o retorno for `false, err_key, params`, a regra recebe status `CheckStatus::Fail`.
4. A mensagem de erro é resolvida buscando a chave `err_key` em `rules.<ID>.messages.<ERR_KEY>` ou no arquivo de localização correspondente (`locales/<lang>.toml`).
5. Variáveis declaradas no dicionário `params` (ex.: `{ header = "..." }`) são interpoladas no template da mensagem `{header}`.
