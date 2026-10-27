# Active Tickets - cccp
> Synced at: 2026-10-27 17:53:21 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#76] feat(ci): subcomandos de esteira cccp ci publish-artifacts e cccp ci deploy-site
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/76

### Description
Implementar subcomandos de automação executados na esteira hermética (.forgejo/workflows/ci.yml) conforme Seção 10 da GOST 2.0 e Seção 5 de Kollontai 2.0.

Requisitos:
- cccp ci publish-artifacts: compilação e despacho determinístico de binários e manifestos para o Cloudflare R2 / Banca (banca.2lp.in).
- cccp ci deploy-site: empacotamento da vitrine estática e deploy em worktree órfã stateless pages para os espelhos secundários sem inflação de histórico.

---
## [#75] feat(site): gerador deterministico cccp site banner a partir de gost metadata
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/75

### Description
Implementar a geração automatizada do banner minimalista de documentação conforme Regras GOST-20, GOST-26 e Seção 6.2 de Kollontai 2.0.

Requisitos:
- Subcomando cccp site banner para gerar deterministicamente site/assets/og_preview.png (1200x630, Safe Zone 4:3 centralizada) a partir de .gost/metadata.json.
- Renderizar fundo com theme.bg_color, acentos com theme.accent_color, ícone oficial e texto {title} — {tagline} (<= 65 caracteres), sem dependências externas de browser headless.

---
## [#74] feat(governance): subcomandos cccp epic para branches persistentes e cccp inbox para quarentena de prs
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/74

### Description
Implementar a governança de branches persistentes e quarentena de fronteira conforme Seção 15 (Regra GOST-25) e Seção 1.2 de Kollontai 2.0.

Requisitos:
- cccp epic start|finish <slug>: ciclo de vida e checkout de branches persistentes de integração epic/<slug>.
- cccp inbox list|review|accept: quarentena de PRs externos em refs/heads/inbox/* isolados contra push acidental nos espelhos secundários (pushurl = no_push).

---
## [#73] feat(ddtdd): subcomandos cccp mutate para fase 7 e cccp hygiene para fase 8
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/73

### Description
Implementar comandos de validação ativa local no fluxo DD-TDD conforme Seções 7.3 e 8 da norma GOST 2.0.

Requisitos:
- cccp mutate: invoca o executor de teste de mutação da stack declarada em GOSTVER (cargo-mutants --in-diff em Rust, infection --git-diff-lines em PHP, etc.) restrito ao diff da branch local (git diff <base>..HEAD).
- cccp hygiene (ou cccp fix): executa linter com -D warnings e formatador determinístico das linguagens declaradas no GOSTVER de forma rápida e incremental.

---
## [#72] feat(ddtdd): subcomandos cccp start, cccp commit com travas de fase e cccp close com laudo
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/72

### Description
Implementar os subcomandos de ciclo de desenvolvimento DD-TDD do Maestro cccp conforme as Seções 5 e 6 da norma GOST 2.0 e Seção 1.2 do perfil Kollontai 2.0.

Requisitos:
- cccp start <id> [--base <b>]: checkout automático para branch <tipo>/<id>-<slug> lendo tipo e slug a partir do ticket #<id>.
- cccp commit "<resumo>": dedução determinística do <id> e da fase (ph3..ph8) pelo histórico da branch de trabalho. Aplicação das travas de fase (bloqueia código em ph3, exige teste vermelho em ph4, exige teste verde em ph5, impõe ordem test -> refactor em ph7) e formata <tipo>: <resumo> (ref #<id> - ph<N>).
- cccp close "<resumo>": exige corpo estruturado com Laudo Textual de Resolução ([Resumo]:, [Solução]:, [Validação]: >= 80 chars), valida sincronização de CHANGELOG.md e site/ e grava commit final (closes #<id> - ph9).
- cccp status: inspeciona a branch ativa e exibe ticket atual, fase alcançada e próximo passo obrigatório.

---
## [#70] feat(check): expandir auditoria para conformidade gost 2.0 e validacao de fases dd-tdd
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/70

### Description
Aprofundar o comando de verificação da GOST (cccp check --strict).

Requisitos:
- Validar conformidade integral com a norma GOST 2.0.0 e perfis especializados (Kollontai 2.0 e Lilya 2.0).
- Inspecionar o cumprimento determinístico das 9 fases do ciclo DD-TDD no branch atual e no histórico de commits.

---
## [#69] feat(catalog): incluir url canonica, host do servidor e apelidos customizaveis no catalogo
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/69

### Description
Expandir a modelagem de dados do Gosplan State Catalog.

Requisitos:
- Armazenar endereço canônico do repositório e host da forja.
- Implementar atribuição automática de apelido baseada no domínio (ex.: primeiro nome do domínio, ou segundo se for git.*).
- Permitir definir apelidos arbitrários fixados no arquivo de configuração do CCCP.

---
## [#68] feat(completion): enriquecer autocompletion dinamico e instalador automatico no bashrc
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/68

### Description
Aperfeiçoar o suporte a autocompletion no terminal.

Requisitos:
- Estender as regras de completude para incluir dinamicamente os slugs de repositórios registrados no catálogo.
- Adicionar helper de instalação (ex.: cccp completion install --bash) para injetar a configuração no .bashrc e na sessão interativa atual.

---
## [#67] feat(cli): subcomando cccp prompt para integracao de status do repositorio no shell
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/67

### Description
Fornecer um integrador rápido em Rust para o prompt do shell (Bash/Zsh).

Requisitos:
- Subcomando cccp prompt de latência ultrabaixa para consumo em PROMPT_COMMAND / PS1.
- Retornar string formatada contendo: estado da árvore (clean/dirty), branch atual, contagem de branches locais e PRs, tickets locais em aberto e status de CI cacheado.

---
## [#66] fix(repo): isolar repositorios remotos e upstream contra mutacoes locais de catalogo
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/66

### Description
Repositórios marcados no catálogo Gosplan com status remote ou com upstreams remotos não devem sofrer intervenções acidentais.

Requisitos:
- Comandos em lote (assemble, sync, hook) devem ignorar automaticamente repositórios que não possuam workspace local de trabalho.
- Bloquear qualquer tentativa de criação ou modificação de arquivos em repositórios categorizados como remote.

---
## [#65] feat(repo): assemble seletivo por repo com trava de working tree limpa e automacao gost
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/65

### Description
Refinar o comando de assemble de especificações da GOST.

Requisitos:
- Adicionar flag para execução em repositório único: cccp repo assemble --repo <slug>.
- Travar a execução se a working tree do repositório estiver dirty.
- Implementar automação do ciclo estrito da GOST durante o assemble (abertura de ticket temporário/interno -> commit com ID -> push) para que a sincronização da GOST não deixe múltiplos repositórios com estado pendente no catálogo.

---
## [#64] feat(git): travas para comandos git manuais e checagem de paridade com origin antes de push
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/64

### Description
Proteger o fluxo diário de desenvolvimento contra execuções diretas do git que contornem o Maestro CCCP.

Requisitos:
- Implementar travas de proteção (via hooks gerenciados pelo cccp hook ou wrappers) para impedir git commit, git push, etc., forçando o uso dos comandos CCCP.
- No comando cccp push, incluir checagem automática de paridade com o tracking branch (origin), prevenindo rejeições remotas e eliminando a necessidade de git pull manual antes do envio.

---
## [#63] fix(cli): permitir argumento posicional ou flag de repo no comando divan
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/63

### Description
O subcomando divan não aceita o parâmetro direto de repositório após a instrução (ex.: cccp divan cccp ou cccp divan user/repo), nem através de opção/flag explícita.

Requisitos:
- Ajustar o parser CLI (clap) para aceitar tanto <repo> posicional opcional quanto --repo <repo>.
- Repassar o contexto do repositório selecionado para o fluxo interativo do Divan.

---
## [#57] refactor(update): descontinuar auto-update interno em favor da jornaleira
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/57

### Description
Remover rotina interna de verificacao periodica de auto-update (check_auto_update) no CCCP, transferindo a gestao de ciclo de vida e atualizacoes dos binarios para o gestor central Jornaleira.

---
