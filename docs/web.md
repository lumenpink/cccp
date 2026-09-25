# CCCP Web Interface & Mayak Command Bridge

> Conventional Commits Compliance Program — Gosplan Web Cockpit & Bridge Integration

## 1. Visão Geral

O comando **`cccp web`** (e o atalho **`cccp diva --web`**) inicializa uma interface web interativa, responsiva e esteticamente alinhada às diretrizes do Gosplan para operações assistidas do CCCP, com foco primordial na **Triagem Interativa de Tickets (O Divã)** e visualização de status do projeto.

Em vez de exigir daemons persistentes pesados ou runtimes adicionais (Node.js, Python, PHP), a arquitetura utiliza o utilitário nativo soviético **Mayak (`mk`)** atuando como **Command Bridge** HTTP local.

---

## 2. Arquitetura do Command Bridge

```
┌─────────────────────────────────┐
│     Navegador Web (Cliente)     │
│   (HTML5 / CSS / Vanilla JS)    │
└────────────────┬────────────────┘
                 │ HTTP (Fetch API)
                 │ POST /api/exec
                 │ Header: Authorization: Bearer <token>
                 ▼
┌─────────────────────────────────┐
│      Comrade Mayak (`mk`)       │
│  - Micro HTTP Server em Rust    │
│  - Servidor estático SPA        │
│  - Validador de Token           │
│  - Sandbox travado em `cccp`    │
└────────────────┬────────────────┘
                 │ IPC (stdin / stdout / exit_code)
                 ▼
┌─────────────────────────────────┐
│         CCCP CLI Engine         │
│  `cccp diva ...`                │
│  `cccp status`                  │
│  `cccp tickets`                 │
└─────────────────────────────────┘
```

### 2.1 Princípios de Segurança & Sandbox
1. **Binário Travado:** O Mayak é iniciado com a flag `--bridge <caminho_cccp>`. O servidor web rejeita qualquer tentativa de executar binários arbitrários do sistema operacional;
2. **Token de Sessão:** Cada inicialização de `cccp web` gera um token criptográfico único (`CCCP_WEB_TOKEN`). Requisições a `/api/exec` sem o cabeçalho `Authorization: Bearer <token>` ou `X-Mayak-Token` são rejeitadas com `401 Unauthorized`;
3. **Escuta Local:** Por padrão, a interface vincula-se ao loopback (`127.0.0.1`), permitindo vinculação a `0.0.0.0` apenas sob instrução explícita (`--host 0.0.0.0`);
4. **Zero Dependência Externa no Cliente:** A interface web consiste em SPA estático autocontido, sem dependência de CDNs externas, bibliotecas npm ou tracking.

---

## 3. Comandos & CLI

### 3.1 Inicialização Direta
```bash
# Inicia a interface web (padrão 8080) sem abrir o navegador
cccp web

# Inicia e abre o navegador automaticamente (opt-in)
cccp web --open

# Inicia com token fixo via flag
cccp web --token meu-segredo

# Inicia com token fixo via variável de ambiente
CCCP_TOKEN="meu-segredo" cccp web

# Inicia em porta específica direto na aba do Divã
cccp diva --web --open
# ou
cccp ticket --web --open
```

### 3.2 Opções do `cccp web`
- `-p, --port <N>`: Porta inicial para escuta (padrão: 8080).
- `--host <IP>`: Endereço de bind (padrão: 127.0.0.1).
- `-o, --open`: Abre automaticamente o navegador padrão (opt-in explícito).
- `-t, --token <KEY>`: Define a chave de autenticação da API (também via `$CCCP_TOKEN` ou `$CCCP_API_KEY`).
- `--no-open`: Não abre o navegador (comportamento padrão mantido para retrocompatibilidade).
- `-q, --quiet`: Modo silencioso (suprime logs no terminal).

---

## 4. Funcionalidades da Interface

### 4.1 Seletor de Repositório Ativo
- Dropdown integrado no cabeçalho do Cockpit para selecionar e alternar o repositório em contexto de trabalho;
- Descoberta automática através do catálogo de repositórios locais (`catalog.json`);
- Atualização em tempo real das abas de Status, Tickets e Divã para o repositório selecionado.

### 4.2 O Divã (Diva Triage Cockpit)
- Campo de entrada livre para relatos de problemas, transcrições de reuniões ou ideias;
- Análise semântica assistida via IA (Gemini);
- Exibição de cards individuais com tipos convencionais (`feat`, `fix`, `refactor`, `docs`, etc.);
- Edição em tempo real de título, escopo e corpo;
- Despacho em lote para o Forge configurado (Codeberg, GitHub, GitLab, Forgejo);
- Geração imediata do comando git para criação de branch GOST:
  `git checkout -b <tipo>/<id>-<slug>`.

### 4.3 Inspeção & Status do Repositório
- Visualização do estado da árvore de trabalho (limpa / modificada);
- Branch atual, upstream sincronizado e status dos hooks do CCCP;
- Versão ativa do CCCP e ferramentas do ecossistema.

### 4.4 Gestão de Tickets
- Leitura dos tickets sincronizados locais (`.antigravity/tickets.md`);
- Filtros por estado e forge de origem.
