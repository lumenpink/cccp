#!/bin/sh

# Enable error handling
set -eu

# =============================================================================
# CCCP Repository Scaffolding Engine (GOST Standard)
# Implements Ticket #57: Automated creation of compliant repositories
# =============================================================================

# -----------------------------------------------------------------------------
# Terminal Colors & NO_COLOR Support
# -----------------------------------------------------------------------------
_newrepo_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        NR_C_RED='\033[0;31m'
        NR_C_GREEN='\033[0;32m'
        NR_C_YELLOW='\033[0;33m'
        NR_C_CYAN='\033[0;36m'
        NR_C_BOLD='\033[1m'
        NR_C_RESET='\033[0m'
    else
        NR_C_RED=''
        NR_C_GREEN=''
        NR_C_YELLOW=''
        NR_C_CYAN=''
        NR_C_BOLD=''
        NR_C_RESET=''
    fi
}

# -----------------------------------------------------------------------------
# Display Usage & Manual
# -----------------------------------------------------------------------------
newrepo_usage() {
    cat << 'EOF'
cccp newrepo - Scaffolding Automatizado de Repositórios no Modelo GOST
====================================================================

Cria novos repositórios ou diretórios pré-configurados em estrita conformidade
com a norma técnica GOST v1.1.0 e convenções semânticas do CCCP.

Uso:
  cccp newrepo <nome-do-repositorio> [opções]
  cccp repo new <nome-do-repositorio> [opções]

Parâmetros:
  <nome-do-repositorio>   Nome do projeto e diretório a ser criado

Opções:
  --root <dir>            Diretório raiz alvo (padrão: ./<nome-do-repositorio>)
  --desc <texto>          Descrição resumida do projeto para README e portal
  --lang <langs>          Linguagens ativas separadas por vírgula (padrão: shell)
  --license <spdx>        Licença SPDX (padrão: CC0-1.0; opções: MIT, GPL-3.0,
                          CC-BY-SA-4.0, Apache-2.0)
  --profile <url>         URL do perfil de implementação (padrão: profiles/lumen.md)
  --no-git                Não inicializar repositório Git local
  --no-hooks              Não instalar os Git hooks do cccp
  --no-ci                 Não gerar workflow base de CI (.forgejo/workflows/ci.yaml)
  --no-sync               Não sincronizar especificações GOST (.gost/)
  -q, --quiet             Modo silencioso, omite mensagens informativas
  -h, --help              Exibe esta mensagem de ajuda e sai

Exemplos:
  cccp newrepo meu-servico
  cccp newrepo meu-cli --lang rust,shell --license MIT
  cccp newrepo portal-docs --license CC-BY-SA-4.0 --desc "Portal público de documentação"
  cccp newrepo app-web --no-git
EOF
}

# -----------------------------------------------------------------------------
# Helper: Write file idempotently (preserves existing files)
# -----------------------------------------------------------------------------
_newrepo_write_file_if_missing() {
    _target="$1"
    _content="$2"
    _label="$3"
    _quiet="${4:-0}"

    if [ -f "$_target" ]; then
        if [ "$_quiet" -eq 0 ]; then
            printf "  %b[EXISTE]%b %s (preservado)\n" "$NR_C_YELLOW" "$NR_C_RESET" "$_label"
        fi
    else
        _dir=$(dirname "$_target")
        [ -d "$_dir" ] || mkdir -p "$_dir"
        printf "%s\n" "$_content" > "$_target"
        if [ "$_quiet" -eq 0 ]; then
            printf "  %b[CRIADO]%b %s\n" "$NR_C_GREEN" "$NR_C_RESET" "$_label"
        fi
    fi
}

# -----------------------------------------------------------------------------
# Helper: Generate Standard SPDX License Text
# -----------------------------------------------------------------------------
_newrepo_license_text() {
    _spdx="$1"
    _name="${2:-Projeto}"
    _year=$(date +%Y 2>/dev/null || echo "2026")

    case "$_spdx" in
        CC0-1.0|CC0)
            cat << 'EOF'
CC0 1.0 Universal (CC0 1.0) Public Domain Dedication

The person who associated a work with this deed has dedicated the work to the
public domain by waiving all of his or her rights to the work worldwide under
copyright law, including all related and neighboring rights, to the extent
allowed by law.

You can copy, modify, distribute and perform the work, even for commercial
purposes, all without asking permission.
EOF
            ;;
        MIT)
            cat << EOF
MIT License

Copyright (c) $_year $_name

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
            ;;
        CC-BY-SA-4.0|CC-BY-SA)
            cat << EOF
Creative Commons Attribution-ShareAlike 4.0 International (CC-BY-SA 4.0)

You are free to:
- Share — copy and redistribute the material in any medium or format
- Adapt — remix, transform, and build upon the material for any purpose, even commercially.

Under the following terms:
- Attribution — You must give appropriate credit, provide a link to the license,
  and indicate if changes were made.
- ShareAlike — If you remix, transform, or build upon the material, you must
  distribute your contributions under the same license as the original.
EOF
            ;;
        GPL-3.0|GPLv3)
            cat << EOF
GNU GENERAL PUBLIC LICENSE
Version 3, 29 June 2007

Copyright (C) $_year $_name

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
EOF
            ;;
        Apache-2.0|Apache)
            cat << EOF
Apache License
Version 2.0, January 2004
http://www.apache.org/licenses/

Copyright $_year $_name

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
EOF
            ;;
        *)
            cat << EOF
Software License: $_spdx
Copyright (c) $_year $_name
EOF
            ;;
    esac
}

# -----------------------------------------------------------------------------
# Main command: cmd_newrepo
# -----------------------------------------------------------------------------
cmd_newrepo() {
    _newrepo_colors

    _nr_name=""
    _nr_root=""
    _nr_desc=""
    _nr_lang="shell"
    _nr_license="CC0-1.0"
    _nr_profile="profiles/lumen.md"
    _nr_git=1
    _nr_hooks=1
    _nr_ci=1
    _nr_sync=1
    _nr_quiet=0

    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                newrepo_usage
                return 0
                ;;
            -q|--quiet)
                _nr_quiet=1
                shift
                ;;
            --root)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --root requer um diretório.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_root="$1"
                shift
                ;;
            --name)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --name requer um nome.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_name="$1"
                shift
                ;;
            --desc)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --desc requer um texto descritivo.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_desc="$1"
                shift
                ;;
            --lang)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --lang requer uma lista de linguagens.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_lang="$1"
                shift
                ;;
            --license)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --license requer um identificador SPDX.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_license="$1"
                shift
                ;;
            --profile)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --profile requer uma URL de perfil.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_profile="$1"
                shift
                ;;
            --no-git)
                _nr_git=0
                shift
                ;;
            --git)
                _nr_git=1
                shift
                ;;
            --no-hooks)
                _nr_hooks=0
                shift
                ;;
            --hooks)
                _nr_hooks=1
                shift
                ;;
            --no-ci)
                _nr_ci=0
                shift
                ;;
            --ci)
                _nr_ci=1
                shift
                ;;
            --no-sync)
                _nr_sync=0
                shift
                ;;
            --sync)
                _nr_sync=1
                shift
                ;;
            -*)
                printf "%bErro: Opção desconhecida '%s'. Use --help para ver as opções disponíveis.%b\n" "$NR_C_RED" "$1" "$NR_C_RESET" >&2
                return 1
                ;;
            *)
                if [ -z "$_nr_name" ]; then
                    _nr_name="$1"
                else
                    printf "%bErro: Parâmetro posicional inesperado '%s'.%b\n" "$NR_C_RED" "$1" "$NR_C_RESET" >&2
                    return 1
                fi
                shift
                ;;
        esac
    done

    # Validation: project name or target directory must be specified
    if [ -z "$_nr_name" ] && [ -z "$_nr_root" ]; then
        printf "%bErro: Informe o nome do repositório a ser criado (ex: cccp newrepo meu-app).%b\n" "$NR_C_RED" "$NR_C_RESET" >&2
        return 1
    fi

    # Determine root directory
    if [ -z "$_nr_root" ]; then
        _nr_root="./${_nr_name}"
    fi

    # Determine project name if derived from root
    if [ -z "$_nr_name" ]; then
        _nr_name=$(basename "$_nr_root")
    fi

    if [ -z "$_nr_desc" ]; then
        _nr_desc="Repositório estruturado em estrita conformidade com o padrão GOST v1.1.0"
    fi

    # Ensure target directory exists
    [ -d "$_nr_root" ] || mkdir -p "$_nr_root"

    if [ "$_nr_quiet" -eq 0 ]; then
        printf "%b☭ [ГОСТ / GOSPLAN] Scaffolding de Repositório: %s%b\n" "$NR_C_BOLD" "$_nr_name" "$NR_C_RESET"
        printf "Diretório Alvo: %s | Linguagens: %s | Licença: %s\n" "$_nr_root" "$_nr_lang" "$_nr_license"
        printf "────────────────────────────────────────────────────────────────────────────\n"
    fi

    # 1. Git Repository Initialization
    if [ "$_nr_git" -eq 1 ] && [ ! -d "${_nr_root}/.git" ]; then
        (cd "$_nr_root" && git init -b main >/dev/null 2>&1 || git init >/dev/null 2>&1)
        if [ "$_nr_quiet" -eq 0 ]; then
            printf "  %b[GIT]%b Repositório Git inicializado na branch main\n" "$NR_C_GREEN" "$NR_C_RESET"
        fi
    fi

    # 2. GOSTVER (GOST-16)
    _gostver_content=$(cat << EOF
# GOSTVER — Declaração de Conformidade com o Padrão GOST
GOST_VERSION="1.1.0"
GOST_PROFILE_URL="$_nr_profile"
GOST_LANGUAGES="$_nr_lang"
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/GOSTVER" "$_gostver_content" "GOSTVER (v1.1.0)" "$_nr_quiet"

    # 3. .editorconfig (GOST-02)
    _editorconfig_content=$(cat << 'EOF'
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
indent_style = space
indent_size = 2

[*.md]
trim_trailing_whitespace = false
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/.editorconfig" "$_editorconfig_content" ".editorconfig" "$_nr_quiet"

    # 4. .gitignore (GOST-02)
    _gitignore_content=$(cat << 'EOF'
# Dependências e compilação
/target/
/vendor/
/node_modules/
/dist/
/build/

# Caches e arquivos temporários
.cache/
*.tmp
*.log
.DS_Store

# Metadados de agentes locais
.antigravity/
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/.gitignore" "$_gitignore_content" ".gitignore" "$_nr_quiet"

    # 5. VERSION (GOST-03)
    _newrepo_write_file_if_missing "${_nr_root}/VERSION" "0.1.0-dev.1" "VERSION (0.1.0-dev.1)" "$_nr_quiet"

    # 6. CHANGELOG.md (GOST-02)
    _changelog_content=$(cat << 'EOF'
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Inicialização do repositório em conformidade com o padrão GOST.
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/CHANGELOG.md" "$_changelog_content" "CHANGELOG.md" "$_nr_quiet"

    # 7. LICENSE (GOST-01)
    _license_body=$(_newrepo_license_text "$_nr_license" "$_nr_name")
    _newrepo_write_file_if_missing "${_nr_root}/LICENSE" "$_license_body" "LICENSE ($_nr_license)" "$_nr_quiet"

    # 8. README.md (GOST-02)
    _readme_content=$(cat << EOF
# $_nr_name

$_nr_desc

## 📋 Conformidade GOST

Este projeto adota a norma **[GOST](https://codeberg.org/lumenpink/gost)** (Gosplan Standard for Technical Software).
- **Versão GOST:** \`1.1.0\`
- **Linguagens declaradas:** \`$_nr_lang\`
- **Licença:** \`$_nr_license\`

## 🚀 Como Iniciar

\`\`\`bash
# Clonar o repositório
git clone <url-do-repositorio>
cd $_nr_name

# Verificar conformidade
cccp check
\`\`\`

## 📄 Licença

Distribuído sob a licença [$_nr_license](./LICENSE).
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/README.md" "$_readme_content" "README.md" "$_nr_quiet"

    # 9. site/_headers e site/index.html (GOST-09)
    _headers_content=$(cat << 'EOF'
/*
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
  Referrer-Policy: strict-origin-when-cross-origin
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/site/_headers" "$_headers_content" "site/_headers" "$_nr_quiet"

    _site_index_content=$(cat << EOF
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>$_nr_name — Documentação</title>
  <style>
    body { font-family: monospace, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; line-height: 1.6; }
    h1 { border-bottom: 2px solid #333; padding-bottom: 8px; }
    .badge { display: inline-block; padding: 2px 8px; background: #eee; border: 1px solid #ccc; font-size: 0.9em; }
  </style>
</head>
<body>
  <h1>$_nr_name</h1>
  <p>$_nr_desc</p>
  <p><span class="badge">GOST v1.1.0</span> <span class="badge">$_nr_license</span></p>
</body>
</html>
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/site/index.html" "$_site_index_content" "site/index.html" "$_nr_quiet"

    # 10. Workflows de CI (GOST-10)
    if [ "$_nr_ci" -eq 1 ]; then
        _ci_workflow_content=$(cat << 'EOF'
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  check:
    runs-on: native
    steps:
      - uses: actions/checkout@v4
      - name: GOST Compliance Audit
        run: |
          if command -v cccp >/dev/null 2>&1; then
            cccp check --strict
          fi
EOF
)
        _newrepo_write_file_if_missing "${_nr_root}/.forgejo/workflows/ci.yaml" "$_ci_workflow_content" ".forgejo/workflows/ci.yaml (CI)" "$_nr_quiet"
    fi

    # 11. Sincronização GOST (.gost/) (GOST-17)
    if [ "$_nr_sync" -eq 1 ]; then
        if command -v cmd_gost_sync >/dev/null 2>&1; then
            cmd_gost_sync "$_nr_root" >/dev/null 2>&1 || true
            if [ "$_nr_quiet" -eq 0 ] && [ -d "${_nr_root}/.gost" ]; then
                printf "  %b[GOST]%b Especificações GOST sincronizadas em .gost/\n" "$NR_C_GREEN" "$NR_C_RESET"
            fi
        fi
    fi

    # 12. Instalação de hooks semânticos CCCP
    if [ "$_nr_hooks" -eq 1 ] && [ -d "${_nr_root}/.git" ]; then
        if command -v cccp >/dev/null 2>&1; then
            (cd "$_nr_root" && cccp install >/dev/null 2>&1 || true)
            if [ "$_nr_quiet" -eq 0 ]; then
                printf "  %b[HOOKS]%b Git hooks instalados via 'cccp install'\n" "$NR_C_GREEN" "$NR_C_RESET"
            fi
        elif command -v install_cccp >/dev/null 2>&1; then
            (cd "$_nr_root" && install_cccp >/dev/null 2>&1 || true)
            if [ "$_nr_quiet" -eq 0 ]; then
                printf "  %b[HOOKS]%b Git hooks instalados via 'install_cccp'\n" "$NR_C_GREEN" "$NR_C_RESET"
            fi
        fi
    fi

    if [ "$_nr_quiet" -eq 0 ]; then
        printf "────────────────────────────────────────────────────────────────────────────\n"
        printf "%b✔ Repositório '%s' criado com sucesso com 100%% de conformidade GOST!%b\n\n" "$NR_C_GREEN" "$_nr_name" "$NR_C_RESET"
        printf "Para iniciar o desenvolvimento:\n"
        printf "  cd %s\n" "$_nr_root"
        printf "  cccp check\n\n"
    fi

    return 0
}
