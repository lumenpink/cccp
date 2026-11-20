# Especificação do Ticket #186: Normalizar Fallback de Linguagem para "None" no read_gostver

## 1. Contexto e Motivação
A função `read_gostver` é responsável por extrair a versão do GOST, a URL canônica do perfil e as linguagens declaradas ou inferidas do repositório inspecionado.
Em repositórios onde nenhuma linguagem foi explicitamente configurada (ausência de `languages` no frontmatter de `.gost/GOST.md` ou `GOST_LANGUAGES` em `GOSTVER`), a rotina `autodetect_languages` é acionada como fallback.
Se nenhuma stack for reconhecida, `autodetect_languages` retorna a string `"none"` (em caixa baixa).
Contudo, o contrato de retorno de `read_gostver` adota `"None"` (capitalizado) para atributos ausentes (como o perfil `None` e versão `Undeclared`), gerando assimetria e quebra no teste de conformidade `test_read_gostver_missing_file` em CI.

## 2. Requisitos da Correção
1. A função `read_gostver` em `src/audit.rs` deve normalizar qualquer valor de linguagem vazio ou igual a `"none"` (insensível a maiúsculas/minúsculas) para o valor canônico `"None"`.
2. O teste de regressão `test_read_gostver_missing_file` deve passar com 100% de sucesso.

## 3. Árvore de Decisões e Registro Arquitetural (para a Natasha)
1. **Normalização no Retorno de `read_gostver` vs Alteração em `autodetect_languages`:**
   - *Alternativa Considerada:* Alterar o retorno de `autodetect_languages` para retornar `"None"` diretamente.
   - *Motivo do Descarte:* Outras partes do código e comandos CLI podem consumir o identificador de linguagem `"none"` em caixa baixa para mapeamentos internos (ex.: flags CLI ou seletores de regras). A normalização formal deve ocorrer no contrato de tupla de `read_gostver`, convertendo `langs.is_empty() || langs.eq_ignore_ascii_case("none")` para `"None"`.
