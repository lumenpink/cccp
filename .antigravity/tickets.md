# Active Tickets - cccp
> Synced at: 2026-09-25 08:43:01 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#46] Continuamos com o problema de confundir o nome do software com o nome da forge
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/46

### Description
 lumen@baquara:~/src/gost$ cccp diva -r navalha
Error: O nome de repositório "navalha" é ambíguo. Encontrados múltiplos correspondentes:
  - lumenpink/navalha (codeberg)
  - lumenpink/navalha (forgejo)
Por favor, informe o nome completo no formato "proprietario/repositorio".


creio que precisamos deixar muito claro no software essa questão. Talvez, internamente, criar um prefixo pra quando for o software? 
Tipo: forgeapp_fojgejo, forgeapp_github, forgeapp_gitlab e forgeapp_sourcehut?

Assim a gente mapeia rápido e fácil onde tá a treta. Que tal?

---
## [#45] feat(auth): suporte a chave de API via env/flag/auto e autenticação explícita na interface web
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/45

### Description
## Contexto & Problema
Atualmente, o token de sessão é injetado diretamente no HTML gerado pelo cccp web, fazendo com que qualquer navegador abra já logado sem nunca exigir a chave de API. Além disso, o usuário não consegue definir sua própria chave via ambiente ou flag.

## Requisitos & Solução
- Permitir configuração da chave de API através de:
  1. Variável de ambiente (CCCP_TOKEN ou CCCP_API_KEY).
  2. Flag de linha de comando (--token, -t).
  3. Geração automática criptográfica aleatória quando não fornecida.
- A interface web não deve abrir logada de forma implícita se o token não for fornecido na URL ou sessão.
- Apresentar tela ou diálogo de autenticação na UI para inserção da chave de API caso não esteja autenticado.

---
## [#44] refactor(web): padronizar flag --open (opt-in) ao invés de --no-open em comandos com interface web
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/44

### Description
## Contexto & Objetivos
Atualmente comandos com servidor web (como 'cccp web' e 'cccp diva --web') abrem o navegador automaticamente por padrão, disponibilizando a flag '--no-open' para evitar a abertura.

## Mudanças Solicitadas
- Inverter o comportamento padrão: por padrão, não abrir o navegador automaticamente.
- Substituir a flag '--no-open' por '--open' / '-o' (opt-in explícito).
- Alinhar com convenções de ferramentas de terminal e documentação.

---
## [#43] feat(cockpit): adicionar seletor de repositório ativo
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/43

### Description
## Descrição
Adicionar uma caixa de seleção (dropdown/select) na interface do cockpit para permitir que o usuário defina qual repositório está sendo trabalhado no momento.

## Passos para Reproduzir / Cenário de Uso
1. Acessar a interface do cockpit.
2. Localizar a barra de ferramentas/painel do cockpit.
3. Selecionar um repositório na caixa de seleção para alterar o contexto ativo.

## Esperado vs Atual
- **Esperado:** O cockpit exibe um componente de seleção de repositório que atualiza o contexto da aplicação para o repositório selecionado.
- **Atual:** Não existe um seletor de repositório no cockpit para gerenciar o repositório ativo.

---
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
