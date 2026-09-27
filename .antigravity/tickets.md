# Active Tickets - cccp
> Synced at: 2026-09-27 02:37:27 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#57] feat(cli): implementar comando newrepo para scaffolding de repositórios no modelo gost
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/57

### Description
## Description

Implementar um novo comando `newrepo` que permita a criação automatizada de repositórios completos já estruturados no modelo `gost` com todos os requisitos iniciais pré-configurados.

## Steps to Reproduce / Propósitos de Uso

1. Executar o comando da CLI (ex: `cccp newrepo <nome-do-repositorio>`)
2. Verificar a estrutura de arquivos e configurações geradas.

## Expected vs Actual

- **Esperado**: O comando cria uma nova pasta/repositório com a estrutura base do modelo `gost`, scripts e arquivos de configuração iniciais.
- **Atual**: Não existe um comando dedicado para automatizar o scaffolding de repositórios.

---
## [#56] chore(gemini): atualizar integração para utilizar a versão mais recente do modelo
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/56

### Description
## Description

Atualizar as configurações e chamadas do cliente Gemini para utilizar a versão padrão mais recente do modelo (ex: `gemini-1.5-flash` ou superior), que possui melhor desempenho e menor latência.

## Steps to Reproduce / Scenarios

1. Analisar a versão atual do modelo Gemini configurada no código.
2. Atualizar para o endpoint/modelo mais recente recomendado pela Google AI SDK.

## Expected vs Actual

- **Expected:** Utilizar o modelo mais recente e otimizado.
- **Actual:** O sistema pode estar apontando para modelos mais antigos ou mais pesados.

---
## [#55] feat(triage): adicionar opção para postagem direta sem intermédio do Gemini
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/55

### Description
## Description

Implementar um modo/flag (ex: `--direct` ou `--no-llm`) no `diva` para postar os dados informados diretamente no repositório, ignorando a etapa de triagem e estruturação via Gemini.

## Steps to Reproduce / Scenarios

1. Executar o comando de postagem com a flag de envio direto.
2. Verificar se o ticket é criado com o texto bruto sem realizar chamadas à API da Gemini.

## Expected vs Actual

- **Expected:** Possibilidade de ignorar a etapa de IA e postar diretamente.
- **Actual:** Atualmente, todos os fluxos dependem do processamento do Gemini.

---
## [#54] feat(cli): aceitar repositório como parâmetro posicional único no diva
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/54

### Description
## Description

Ajustar o parsing de argumentos da CLI do `diva` para que, caso apenas um parâmetro seja informado na linha de comando, ele seja automaticamente atribuído como o repositório alvo.

## Steps to Reproduce / Scenarios

1. Executar `diva <repositorio>`
2. O comando deve inferir que `<repositorio>` é o alvo da operação sem exigir flags explícitas (ex: `-r` ou `--repo`).

## Expected vs Actual

- **Expected:** O parâmetro único é interpretado como o repositório.
- **Actual:** O CLI exige flags adicionais ou falha por falta de parâmetros nomeados.

---
## [#53] chore(legal)!: retificar licenciamento desde o commit inicial e reescrever histórico
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/53

### Description
## Contexto
Para garantir pureza jurídica e coerência desde a gênese do projeto, os termos de licenciamento corretos devem ser consolidados retroativamente a partir do primeiro commit (root commit), eliminando declarações legadas ou conflitantes ao longo do histórico Git.

## Tarefas
- [ ] Substituir/ajustar o arquivo `LICENSE` no commit inicial e em seus descendentes.
- [ ] Reescrever a árvore histórica do repositório mantendo metadados de autoria e timestamps originais.
- [ ] Recriar ou re-apontar tags anotadas existentes para os novos commits correspondentes.
- [ ] Executar push sincronizado na forja com `--force-with-lease`.

## ⚠️ Atenção
Esta operação é um Breaking Change (`chore(legal)!`) no histórico Git do repositório, alterando os hashes SHA de todos os commits subsequentes.

---
## [#52] chore(legal): adequar matriz de licenciamento modular ao padrão GOST 1.1.0
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/52

### Description
## Contexto
Conforme definido na norma técnica GOST v1.1.0 (`specs/legal/licenses.md`), cada repositório deve adotar a matriz de licenciamento estritamente adequada à sua natureza/perfil (princípio anti-boilerplate), com a licença padrão na raiz (`LICENSE`) e arquivos satélites canônicos quando pertinentes (`LICENSE-MEDIA`, `LICENSE-LOGO`, `LICENSE-APACHE`).

## Tarefas
- [ ] Identificar a tipologia do repositório (Aplicação, Comrade/Microapp, Biblioteca/Protocolo ou Especificação).
- [ ] Adequar o arquivo raiz `LICENSE` aos termos legais correspondentes.
- [ ] Incluir satélites pertinentes (`LICENSE-MEDIA`, `LICENSE-LOGO`, `LICENSE-APACHE`) se houver mídias ou marcas a proteger.
- [ ] Mapear explicitamente a matriz de licenciamento no `README.md`.

## Critérios de Aceite
- Licença raiz condizente com a política GOST 1.1.0.
- Ausência de arquivos satélites vazios ou boilerplate genérico irrelevante.
- Seção de Licença no `README.md` discriminando os arquivos e termos aplicáveis.

---
## [#50] fix(diva): usar comrade kv (kvant) como editor de texto padrão
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/50

### Description
## Description

Atualmente, o Divã não está configurado para utilizar o Comrade KV (Kvant) como seu micro editor de texto padrão.

## Steps to Reproduce

1. Iniciar uma ação de edição de texto no Divã com as configurações padrão.
2. Verificar qual editor de texto é chamado pela aplicação.

## Expected vs Actual

- **Esperado:** O Comrade KV (Kvant) deve ser utilizado como o editor de texto padrão do Divã.
- **Atual:** O Divã utiliza outro editor de texto ou não define o Comrade KV por padrão.

---
## [#49] refactor(cli): desduplicar aliases no micro help e unificar comando de tickets
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/49

### Description
## Description

Ao executar `cccp` sem parâmetros, a listagem simplificada de comandos (micro help) mostra aliases como `ticket|tickets`. Além disso, o help geral menciona `sync-tickets`, gerando dúvida sobre a existência de 3 comandos para a mesma finalidade.

O micro help deve ser limpo para exibir unicamente o comando principal/canônico de cada funcionalidade, reservando a listagem de aliases apenas para a documentação detalhada de ajuda.

## Steps to Reproduce

1. Executar `cccp` sem passar nenhum argumento.
2. Observar a listagem rápida mostrando `ticket|tickets`.
3. Executar `cccp --help` e observar a presença do alias `sync-tickets`.

## Expected vs Actual

- **Esperado:** O micro help deve exibir apenas um nome de comando canônico por funcionalidade (ex: `tickets`).
- **Atual:** O micro help lista variações com pipe (`ticket|tickets`) e causa redundância com outros aliases (`sync-tickets`).

---
## [#48] fix(apps): corrigir identificação de forge e metadados no app tickets e demais subapps
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/48

### Description
## Description

O aplicativo `tickets` está se identificando em suas mensagens/metadados como um conector específico para as APIs do GitHub ou Codeberg. No entanto, o comportamento correto definido é o acesso genérico aos 4 tipos de forgeapps suportados em qualquer forge.

É necessário verificar a implementação e documentação interna do app `tickets` e auditar todos os outros subaplicativos para garantir que nenhum esteja com identificação de backend/forge incorreta.

## Steps to Reproduce

1. Executar o app `tickets` ou inspecionar suas descrições/logs de ajuda.
2. Verificar menções explícitas e limitantes a GitHub/Codeberg em vez da abstração de forgeapps.

## Expected vs Actual

- **Esperado:** Os apps devem se identificar de forma agnóstica de forge, indicando o suporte aos tipos de forgeapps configurados.
- **Atual:** O app `tickets` se identifica incorretamente como acessando diretamente a API do GitHub/Codeberg.

---
