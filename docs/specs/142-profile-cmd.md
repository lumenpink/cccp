# Especificação Técnica: Subcomando `cccp profile`

**Ticket Associado:** #142  
**Branch:** `feat/142-profile-set`  
**Norma:** GOST 1.4.0 (Perfil Lumen / Herança Kollontai)

## 1. Objetivo
Implementar o subcomando `cccp profile` no CLI `cccp` para inspecionar, listar e atualizar o perfil canônico GOST ativo no repositório atual, gerenciando o arquivo `GOSTVER` e sincronizando os metadados locais.

## 2. Interface de Linha de Comando (CLI)

```bash
cccp profile [COMMAND]

Commands:
  get          Exibe a URL e o alias do perfil GOST configurado no repositório atual
  list         Lista os perfis GOST conhecidos e oficiais do ecossistema
  set <NAME>   Atualiza o perfil ativo no repositório atual (suporta aliases e URLs diretas)
```

### 2.1 Aliases Canônicos Suportados:
- `lumen` (ou `default`): `https://gostlumen.2lp.in/profile.md`
- `kollontai`: `https://kollontai.2lp.in/profile.md`

Qualquer URL iniciando com `http://` ou `https://` pode ser fornecida diretamente.

## 3. Comportamento do `cccp profile set`:
1. Localiza a raiz do repositório (diretório contendo `.git` ou `GOSTVER`).
2. Se `GOSTVER` existir:
   - Substitui ou adiciona a linha `GOST_PROFILE_URL="<url>"`.
3. Se `GOSTVER` não existir:
   - Cria o arquivo com versão padrão `GOST_VERSION="1.4.0"` e o perfil configurado.
4. Exibe mensagem com formatação e cores do tema Gosplan / CCCP.
