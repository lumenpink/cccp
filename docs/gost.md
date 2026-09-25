# Conformidade e Auditoria GOST no CCCP

> **ГОСТ** (*Государственный стандарт* / Norma Técnica Estatal)  
> Diretrizes para padronização técnica, engenharia de software e controle de qualidade.

O CCCP atua como a ferramenta canônica de sincronização, auditoria e migração de conformidade com o padrão GOST para repositórios consumidores.

---

## 1. Arquivo `GOSTVER`

Repositórios compatíveis com o GOST declaram sua versão, perfil e linguagens no arquivo `GOSTVER` localizado na raiz do projeto:

```sh
# GOSTVER — Declaração de Conformidade com o Padrão GOST
GOST_VERSION="1.0.0-draft"
GOST_PROFILE_URL="profiles/lumen.md"
GOST_LANGUAGES="shell,javascript"
```

### Variáveis Declaradas:
- **`GOST_VERSION`**: Versão da norma GOST adotada pelo projeto (SemVer).
- **`GOST_PROFILE_URL`**: Caminho relativo ou URL remota para o perfil de adoção concreta da organização/ecossistema.
- **`GOST_LANGUAGES`**: Lista separada por vírgula de linguagens cujas especificações modulares devem ser mantidas no repositório.

---

## 2. Comandos

### `cccp gost sync`
Sincroniza os documentos da norma para o diretório local `.gost/`:
- Baixa `SPEC.md` da versão declarada para `.gost/GOST.md`.
- Baixa o perfil especificado para `.gost/PROFILE.md`.
- Baixa o catálogo de regras técnicas para `.gost/rules.json`.
- Baixa **exclusivamente** as especificações das linguagens listadas em `GOST_LANGUAGES` para `.gost/languages/<linguagem>.md` (evitando poluição e consumo desnecessário de contexto de IA).

```bash
cccp gost sync
```

### `cccp check` (ou `cccp gost check`)
Audita as regras do repositório contra as normas técnicas do GOST:
- **`GOST-16-GOSTVER`**: Presença e validade do `GOSTVER`.
- **`GOST-01-LICENSE`**: Presença do arquivo `LICENSE` válido e não-vazio.
- **`GOST-02-ANATOMY-ROOT`**: Presença dos arquivos essenciais (`README.md`, `VERSION`, `CHANGELOG.md`, `.gitignore`, `.editorconfig`).
- **`GOST-03-VERSION-SEMVER`**: Formato SemVer 2.0.0 estrito no arquivo `VERSION`.
- **`GOST-04-TICKETS-COMMITS`**: Rastreabilidade de tickets em commits recentes (`(ref #ID)` ou `(closes #ID)`).
- **`GOST-05-BRANCH-PER-TICKET`**: Padrão de branches de desenvolvimento dedicadas (`<type>/<ticket>-<slug>`).
- **`GOST-09-SITE-HEADERS`**: Cabeçalhos HTTP de segurança (`site/_headers`) se vitrine web estiver presente.
- **`GOST-10-CI-SEPARATION`**: Workflows de automação com separação de etapas de teste.
- **`GOST-17-LANGUAGE-SPECS`**: Especificações de linguagem sincronizadas em `.gost/languages/`.

```bash
# Auditoria padrão:
cccp check

# Modo estrito (trata avisos SHOULD como erros bloqueantes):
cccp check --strict

# Auditar outro diretório/repositório:
cccp check -C /caminho/do/projeto
```

### `cccp gost upgrade <versão>`
Executa o fluxo de migração de versão da norma:
1. Localiza e exibe o guia de migração (`upgrades/from-X-to-Y.md`).
2. Atualiza a variável `GOST_VERSION` no arquivo `GOSTVER`.
3. Executa automaticamente o `cccp gost sync` para renovar a especificação e as regras em `.gost/`.

```bash
cccp gost upgrade 1.1.0
```

---

## 3. Disciplina de Terminal e Cores

- **Suporte ANSI Completo**: O relatório de auditoria apresenta cores temáticas Gosplan (`[PASS]` em verde, `[WARN]` em amarelo, `[FAIL]` em vermelho).
- **Suporte a `NO_COLOR`**: Atende estritamente à especificação [no-color.org](https://no-color.org) quando a variável de ambiente `NO_COLOR=1` estiver definida.
- **Detecção de TTY**: Cores são automaticamente suprimidas quando a saída é redirecionada para pipes ou arquivos.
