# Active Tickets - cccp
> Synced at: 2026-09-24 23:14:07 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#42] feat(check): implementar comando padrão cccp check conforme a gost spec
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/42

### Description
## Descrição

Implementar o comando padrão `cccp check` no projeto `cccp` para garantir conformidade com a especificação Gost (gost spec).

## Requisitos / Passos para Implementação

1. Mapear as regras e requisitos definidos na Gost spec para a etapa de verificação (`check`).
2. Criar a estrutura do subcomando `cccp check` com as opções e flags padrão necessárias.
3. Adicionar testes unitários/de integração para validar o comportamento do comando.

## Esperado vs Atual

- **Esperado:** O comando `cccp check` executa as validações padrão alinhadas à Gost spec.
- **Atual:** O comando não possui o comportamento padrão implementado de acordo com a especificação.

---
## [#41] acesso web
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/41

### Description
Implementar acesso web
Para os comandos que fazem sentido, claro
Tipo diva

---
## [#40] refactor(paths): adequar armazenamento de arquivos e binários à especificação XDG
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/40

### Description
## Descrição

Atualmente, o `cccp` armazena tanto seus arquivos de configuração quanto seus binários diretamente no diretório `~/.cccp`. Isso não segue a especificação XDG Base Directory.

É necessário reavaliar e refatorar os locais de armazenamento para utilizar os caminhos padrão do sistema:
- **Configurações:** `$XDG_CONFIG_HOME/cccp` (fallback: `~/.config/cccp`)
- **Binários/Dados:** `$XDG_DATA_HOME/cccp` ou `~/.local/bin` / `~/.local/share/cccp`

## Passos para Reproduzir

1. Executar o `cccp` em um ambiente sem pré-configuração.
2. Verificar a criação do diretório `~/.cccp` no diretório home do usuário contendo configurações e binários mesclados.

## Comportamento Esperado vs Atual

- **Esperado:** Respeitar as variáveis de ambiente `$XDG_CONFIG_HOME` e `$XDG_DATA_HOME`, salvando configurações em `~/.config/cccp` e binários/executáveis em locais apropriados (ex: `~/.local/bin` ou `~/.local/share/cccp`).
- **Atual:** Todos os artefatos (configuração e binários) são salvos de forma centralizada em `~/.cccp`.

---
## [#39] feat(serve): implementar comando unificado cccp serve (estático, dev e demo)
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/39

### Description
## Descrição

Consolidar a multiplicidade de scripts heterogêneos de desenvolvimento e preview (`serve.sh`, `dev.sh`, `demo.sh`, `start.sh`) espalhados pelos repositórios em um comando unificado de primeiro nível no CCCP (`cccp serve`), eliminando código duplicado e padronizando a experiência do desenvolvedor sob a estética Gosplan.

## Funcionalidades & Modos

### 1. Modo Estático Padrão (`cccp serve [diretório] [opções]`)
- **Alvo:** Detecta automaticamente `./site/`, `./dist/`, `./public/` ou raiz `./`.
- **Servidor HTTP:** Prioriza o camarada **`mayak`** (`mk`); fallback automático para `python3 -m http.server` e `busybox httpd`.
- **Rede Padrão:** Escuta em `0.0.0.0` (acessível via LAN / Tailscale por padrão).
- **Auto-porta:** Porta inicial `8080` com incremento automático se a porta estiver ocupada (`lsof` / `ss`).
- **Flags:**
  - `--localhost`, `-l`: Restringe o bind para `127.0.0.1`.
  - `--host <IP>`: IP customizado.
  - `-p, --port <N>`: Porta inicial personalizada.
  - `-o, --open`: Abre o navegador padrão via `xdg-open` / `open`.

### 2. Modo Desenvolvimento (`cccp serve --dev`)
- **Projetos Dinâmicos:**
  - Se for PHP (`index.php`, `fuxico.php`, etc.) -> executa `php -S "$HOST:$PORT" <entrypoint>`.
  - Se for Node (`package.json`) -> executa script dev/serve do projeto via `npx` / `npm`.
- **Projetos Compilados / Estáticos com Build:**
  - Dispara watcher em segundo plano com debounce (`inotifywait` com fallback para polling mtime).
  - Monitora `src/`, `packages/`, `crates/`, `site/` (ignorando `.git/`, `target/`, `node_modules/`, `cache/`).
  - Auto-detecção de builder (`combine.sh`, `compile.php`, `cargo build`, `make`).
  - Delega para `scripts/dev.sh` ou `.cccp/dev.sh` se o repositório definir rotina própria.

### 3. Modo Demonstração (`cccp serve --demo`)
- Executa rotina de setup/seed/fixtures do projeto (`scripts/demo.sh` ou `.cccp/demo.sh`).
- Faz polling de healthcheck até o serviço responder.
- Exibe banner temático da *Exposição das Conquistas da Economia Nacional (VDNKh)* com links e rotas ativas.
- Abre o navegador automaticamente.

## Diretrizes Técnicas
- 100% POSIX sh, zero-AWK no core do comando.
- Encerramento limpo via traps POSIX (`SIGINT`, `SIGTERM`), finalizando quaisquer processos filhos em background.
- Banner Gosplan temático com URLs (Loopback e LAN/Tailscale).

---
## [#38] feat(repo): implementar visão geral e gestão unificada de múltiplos repositórios
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/38

### Description
## Description

Adicionar funcionalidade de gestão agregada de repositórios, permitindo visualizar em um único painel ou comando o status de múltiplos projetos.

### Funcionalidades necessárias:
- Listar repositórios que possuem alterações não salvas/commitadas (`dirty`).
- Listar repositórios com commits pendentes de envio (`unpushed`).
- Listar repositórios que possuem tickets/issues em aberto no Forgejo.

## Comportamento Esperado vs Atual

- **Esperado:** Ter uma visão consolidada do status de múltiplos repositórios simultaneamente.
- **Atual:** A verificação de status precisa ser feita individualmente em cada repositório.

---
## [#37] Implement pre-push hook with act: local CI gate, live spinner and structured logs
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/37

### Description
## Problem
Pushes can reach the remote forge with failing tests. Running `composer test` or `cargo test` locally is insufficient — local environments diverge from CI (runtime versions, env vars, system deps), producing the classic 'passes locally, fails on CI' problem.

## Expected Behavior
The `pre-push` hook installed by `cccp install` should:

### Core behavior
1. Detect if `act` (Docker-based Forgejo/GitHub Actions runner) is available
2. Run `act -j test --no-pull` against the repo's `.forgejo/workflows/ci.yml` (using cached images)
3. Stream full output to a structured log file: `.git/act-logs/<YYYY-MM-DD>_<HH-MM>_<branch>_<short-sha>.log`
4. Show live feedback in the terminal (spinner, current step name, elapsed time)
5. On success: print summary and allow the push to proceed
6. On failure: print condensed error summary with log path, abort the push

### UX (terminal output during gate)
```
⠸ [pre-push] Running CI gate locally...
  Job: test › Lint & Static Analysis     [0:12]
  Log: .git/act-logs/2026-09-24_09-41_feat-42_a4b3c2d.log
```

On pass:
```
✅ CI gate passed (1m 34s) — pushing to origin/main
```

On fail:
```
❌ CI gate failed (0m 47s) — push aborted
   → Step failed: cargo test
   → Log: .git/act-logs/2026-09-24_09-41_feat-42_a4b3c2d.log
   → Bypass (conscious): git push --no-verify
```

### Command: cccp act log
Shortcut to open/tail the most recent act log file.

### Opt-out
`git push --no-verify` is the documented conscious bypass.

## Acceptance Criteria
- [ ] pre-push hook runs `act -j test` before allowing push
- [ ] act stdout/stderr fully captured in `.git/act-logs/`
- [ ] Log filename includes timestamp, branch name and short sha
- [ ] Live spinner showing current step and elapsed time
- [ ] Clear pass/fail summary with log path
- [ ] `cccp act log` opens/tails the latest log
- [ ] Graceful degradation if `act` or Docker is not running (warn, don't block)
- [ ] Documented in `cccp help`

---
## [#36] Add 'cccp ticket' command as a first-class alias for issue creation via diva
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/36

### Description
## Problem

Currently opening a ticket requires using external tools (`tea`, `gh`, Codeberg web UI) even though `cccp diva` already handles AI-powered issue generation internally. This is inconsistent with the GOST standard that mandates every commit have a ticket — the friction of switching tools discourages the practice.

## Expected Behavior

A first-class `cccp ticket` command (or `cccp issue`) that:
- Wraps `cccp diva` flow for the current repository
- Opens an issue interactively via Gemini AI (natural language description → structured ticket)
- Optionally accepts a short description as argument: `cccp ticket "broken login redirect"`
- Confirms before publishing (existing diva UX)
- Returns the issue number so it can immediately be used in branch naming (`feat/<id>-slug`)
- Works with the forge configured in `.cccprc` (Codeberg/Forgejo as primary)

## Acceptance Criteria
- [ ] `cccp ticket` with no args opens interactive diva chat
- [ ] `cccp ticket "<description>"` pre-populates diva with the description
- [ ] After creation, prints the issue number and suggested branch name
- [ ] Documented in `cccp help`

---
## [#35] fix(cli): validar existência local do repositório informado via flag -r
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/35

### Description
## Description

Se um repositório desconhecido/inexistente for passado através do parâmetro `-r`, a CLI aceita o valor sem realizar a verificação de existência local.

## Steps to Reproduce

1. Executar o comando informando um repositório inexistente: `-r repo_inexistente`.
2. Observar que o argumento é aceito sem erro de validação.

## Expected vs Actual

**Expected:** O sistema deve aceitar apenas repositórios que existam localmente, retornando erro caso contrário.
**Actual:** O repositório desconhecido é aceito silenciosamente.

---
