# Especificação Normativa: Subcomandos `cccp mutate` e `cccp hygiene` (Fases 7 e 8)

- **Ticket de Referência:** `#73`
- **Norma Base:** GOST v2.0.0 (Seções 7.3 e 8)
- **Perfil:** Kollontai v2.0.0 / Lilya v2.0.0
- **Fase DD-TDD:** Fase 3 — Doc/Spec (Zero código de produção)
- **Status:** Aprovado para Implementação

---

## 1. Escopo e Propósito

Esta especificação define o contrato operacional, resolução de toolchains homologadas por linguagem e execução incremental dos subcomandos:

1. `cccp mutate [--base <b>] [--dry-run]` (Fase 7 — Teste de Mutação no Diff)
2. `cccp hygiene [--fix] [--lang <langs>]` / `cccp fix` (Fase 8 — Quality Gate & Fast Hygiene)

---

## 2. Contrato do Subcomando `cccp mutate`

### 2.1 Assinatura da CLI
```text
cccp mutate [OPTIONS]

OPÇÕES:
  -b, --base <BRANCH>   Branch de referência para cálculo do diff (padrão: "main")
      --dry-run         Exibe o comando a ser executado sem dispará-lo
  -C, --dir <DIR>       Diretório raiz do repositório
```

### 2.2 Resolução de Toolchains de Mutação
O Maestro lê as linguagens declaradas no `GOSTVER` (`GOST_LANGUAGES`) e despacha:

| Linguagem | Executor Canônico Homologado | Comando Executado |
| :--- | :--- | :--- |
| **`rust`** | `cargo-mutants` | `cargo mutants --in-diff` |
| **`php`** | `infection` | `vendor/bin/infection --git-diff-lines` |
| **`javascript`** / **`typescript`** | `stryker` | `npx stryker run --diff` |
| **`go`** | `gremlins` | `gremlins unleash --diff` |
| **`python`** | `mutatest` | `mutatest` |
| **`shell`** / **`r`** | Inspeção Estática | Verificação sintática de invariantes |

---

## 3. Contrato do Subcomando `cccp hygiene` / `cccp fix`

### 3.1 Assinatura da CLI
```text
cccp hygiene [OPTIONS]
cccp fix [OPTIONS]

OPÇÕES:
      --fix             Aplica correções automáticas de formatação e linter
  -l, --lang <LANGS>    Sobrescrita explícita de linguagens (separadas por vírgula)
  -C, --dir <DIR>       Diretório raiz do repositório
```

*Nota:* O comando `cccp fix` é um alias canônico determinístico para `cccp hygiene --fix`.

### 3.2 Matriz de Execução Fast Hygiene (Fase 8)

Quando invocado sem `--fix` (modo verificação):
* **`rust`**:
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`
* **`php`**:
  - `vendor/bin/pint --test`
  - `vendor/bin/phpstan analyse --level=max`
* **`shell`**:
  - `shfmt -d -i 2 -ci -bn`
  - `shellcheck`
* **`javascript` / `typescript`**:
  - `npx @biomejs/biome check`
* **`python`**:
  - `ruff check`
  - `ruff format --check`

Quando invocado com `--fix` (modo correção):
* **`rust`**: `cargo fmt`
* **`php`**: `vendor/bin/pint`
* **`shell`**: `shfmt -w -i 2 -ci -bn`
* **`javascript` / `typescript`**: `npx @biomejs/biome check --write`
* **`python`**: `ruff check --fix && ruff format`

---

## 4. Retorno e Códigos de Saída
* Retorna código de saída `0` (`ExitCode::SUCCESS`) se todas as verificações passarem sem warnings.
* Retorna código `1` (`ExitCode::FAILURE`) imediatamente caso qualquer advertência seja emitida (Princípio *Zero Warnings*).
