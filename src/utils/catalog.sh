#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -n "$GIT_ROOT" ]; then
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    if [ -f "$GIT_ROOT/src/config/config.sh" ]; then
        . "$GIT_ROOT/src/config/config.sh"
    fi
    if [ -f "$GIT_ROOT/src/utils/config_manager.sh" ]; then
        . "$GIT_ROOT/src/utils/config_manager.sh"
    fi
    if [ -f "$GIT_ROOT/src/utils/repo.sh" ]; then
        . "$GIT_ROOT/src/utils/repo.sh"
    fi
fi

# -----------------------------------------------------------------------------
# Helper: Extract Forge Token from ~/.config/tea/config.yml (Pure POSIX)
# -----------------------------------------------------------------------------
_catalog_get_tea_token() {
    if command -v get_tea_token >/dev/null 2>&1; then
        get_tea_token "$@"
        return $?
    fi
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Resolve Forge API Token dynamically without hardcoding
# -----------------------------------------------------------------------------
_catalog_resolve_token() {
    if command -v resolve_forge_token >/dev/null 2>&1; then
        resolve_forge_token "$@"
        return $?
    fi
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Dispatch HTTP request via sv / svyaz / curl
# -----------------------------------------------------------------------------
_catalog_http() {
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    case "$(type sv 2>/dev/null)" in
        *function*)
            sv "$@"
            return $?
            ;;
    esac
    case "$(type curl 2>/dev/null)" in
        *function*)
            curl "$@"
            return $?
            ;;
    esac

    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
    elif command -v sv >/dev/null 2>&1; then
        sv "$@"
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
    elif command -v curl >/dev/null 2>&1; then
        curl "$@"
    else
        return 127
    fi
}

# -----------------------------------------------------------------------------
# Helper: Parse Remote URL to extract Forge, Slug, and default is_upstream
# -----------------------------------------------------------------------------
_catalog_detect_forge_and_slug() {
    _url="$1"
    if [ -z "$_url" ]; then
        printf 'local unknown false\n'
        return 0
    fi

    if ! _parsed=$(parse_remote_url "$_url" 2>/dev/null); then
        printf 'local unknown false\n'
        return 0
    fi

    _chost=$(printf '%s\n' "$_parsed" | cut -d' ' -f1)
    _cowner=$(printf '%s\n' "$_parsed" | cut -d' ' -f2)
    _crepo=$(printf '%s\n' "$_parsed" | cut -d' ' -f3)
    _cslug="${_cowner}/${_crepo}"
    printf '%s %s false\n' "$_chost" "$_cslug"
}

# -----------------------------------------------------------------------------
# Helper: Heuristic Metadata Extraction (Description, License, Topics, Website)
# -----------------------------------------------------------------------------
_catalog_extract_description() {
    _dir="$1"
    _desc=""

    # 1. Check existing .git/description (if not default placeholder)
    if [ -f "$_dir/.git/description" ]; then
        _raw=$(head -n 1 "$_dir/.git/description" | tr -d '\r\n')
        case "$_raw" in
            *"Unnamed repository; edit this file 'description'"*|"")
                ;;
            *)
                printf '%s\n' "$_raw"
                return 0
                ;;
        esac
    fi

    # 2. Check Cargo.toml
    if [ -f "$_dir/Cargo.toml" ]; then
        _val=$(grep '^[[:space:]]*description[[:space:]]*=' "$_dir/Cargo.toml" 2>/dev/null | head -n 1 || true)
        if [ -n "$_val" ]; then
            _desc=$(printf '%s' "$_val" | sed -e 's/^[[:space:]]*description[[:space:]]*=[[:space:]]*"//' -e 's/"[[:space:]]*$//')
            if [ -n "$_desc" ]; then
                printf '%s\n' "$_desc"
                return 0
            fi
        fi
    fi

    # 3. Check package.json
    if [ -f "$_dir/package.json" ]; then
        _val=$(grep '^[[:space:]]*"description"[[:space:]]*:' "$_dir/package.json" 2>/dev/null | head -n 1 || true)
        if [ -n "$_val" ]; then
            _desc=$(printf '%s' "$_val" | sed -e 's/^[[:space:]]*"description"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
            if [ -n "$_desc" ]; then
                printf '%s\n' "$_desc"
                return 0
            fi
        fi
    fi

    # 4. Check composer.json
    if [ -f "$_dir/composer.json" ]; then
        _val=$(grep '^[[:space:]]*"description"[[:space:]]*:' "$_dir/composer.json" 2>/dev/null | head -n 1 || true)
        if [ -n "$_val" ]; then
            _desc=$(printf '%s' "$_val" | sed -e 's/^[[:space:]]*"description"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
            if [ -n "$_desc" ]; then
                printf '%s\n' "$_desc"
                return 0
            fi
        fi
    fi

    # 5. Check README.md
    if [ -f "$_dir/README.md" ]; then
        while IFS= read -r _rline || [ -n "$_rline" ]; do
            case "$_rline" in
                [#\<]*|"") continue ;;
                *)
                    _desc=$(printf '%s' "$_rline" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
                    if [ -n "$_desc" ]; then
                        printf '%s\n' "$_desc"
                        return 0
                    fi
                    ;;
            esac
        done < "$_dir/README.md"
    fi

    printf ''
}

_catalog_extract_license() {
    _dir="$1"

    # 1. Cargo.toml
    if [ -f "$_dir/Cargo.toml" ]; then
        _lic=$(grep '^[[:space:]]*license[[:space:]]*=' "$_dir/Cargo.toml" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*license[[:space:]]*=[[:space:]]*"//' -e 's/"[[:space:]]*$//' || true)
        [ -n "$_lic" ] && { printf '%s\n' "$_lic"; return 0; }
    fi

    # 2. package.json
    if [ -f "$_dir/package.json" ]; then
        _lic=$(grep '^[[:space:]]*"license"[[:space:]]*:' "$_dir/package.json" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*"license"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//' || true)
        [ -n "$_lic" ] && { printf '%s\n' "$_lic"; return 0; }
    fi

    # 3. composer.json
    if [ -f "$_dir/composer.json" ]; then
        _lic=$(grep '^[[:space:]]*"license"[[:space:]]*:' "$_dir/composer.json" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*"license"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//' || true)
        [ -n "$_lic" ] && { printf '%s\n' "$_lic"; return 0; }
    fi

    # 4. LICENSE file inspection
    for _lic_file in "$_dir/LICENSE" "$_dir/LICENSE.md" "$_dir/LICENSE.txt" "$_dir/COPYING"; do
        if [ -f "$_lic_file" ]; then
            if grep -qi "Apache License" "$_lic_file"; then
                printf 'Apache-2.0\n'; return 0
            elif grep -qi "MIT License" "$_lic_file" || grep -q "MIT" "$_lic_file"; then
                printf 'MIT\n'; return 0
            elif grep -qi "Affero General Public License" "$_lic_file" || grep -q "AGPL" "$_lic_file"; then
                printf 'AGPL-3.0\n'; return 0
            elif grep -qi "General Public License" "$_lic_file" || grep -q "GPL" "$_lic_file"; then
                printf 'GPL-3.0\n'; return 0
            elif grep -qi "Creative Commons Zero" "$_lic_file" || grep -q "CC0" "$_lic_file"; then
                printf 'CC0-1.0\n'; return 0
            elif grep -qi "Unlicense" "$_lic_file"; then
                printf 'Unlicense\n'; return 0
            elif grep -qi "Creative Commons Attribution-ShareAlike" "$_lic_file"; then
                printf 'CC-BY-SA-4.0\n'; return 0
            fi
        fi
    done

    printf 'MIT\n'
}

_catalog_extract_website() {
    _dir="$1"
    # Check package.json homepage
    if [ -f "$_dir/package.json" ]; then
        _hp=$(grep '^[[:space:]]*"homepage"[[:space:]]*:' "$_dir/package.json" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*"homepage"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//' || true)
        [ -n "$_hp" ] && { printf '%s\n' "$_hp"; return 0; }
    fi
    # Check Cargo.toml homepage
    if [ -f "$_dir/Cargo.toml" ]; then
        _hp=$(grep '^[[:space:]]*homepage[[:space:]]*=' "$_dir/Cargo.toml" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*homepage[[:space:]]*=[[:space:]]*"//' -e 's/"[[:space:]]*$//' || true)
        [ -n "$_hp" ] && { printf '%s\n' "$_hp"; return 0; }
    fi
    printf ''
}

_catalog_extract_topics() {
    _dir="$1"
    # package.json keywords
    if [ -f "$_dir/package.json" ]; then
        _kw=$(grep -A 10 '^[[:space:]]*"keywords"[[:space:]]*:' "$_dir/package.json" 2>/dev/null | grep '"' | grep -v 'keywords' | sed -e 's/^[[:space:]]*"//' -e 's/",*[[:space:]]*$//' | tr '\n' ' ' || true)
        [ -n "$_kw" ] && { printf '%s\n' "$_kw"; return 0; }
    fi
    # Cargo.toml keywords
    if [ -f "$_dir/Cargo.toml" ]; then
        _kw=$(grep '^[[:space:]]*keywords[[:space:]]*=' "$_dir/Cargo.toml" 2>/dev/null | sed -e 's/^[[:space:]]*keywords[[:space:]]*=[[:space:]]*\[//' -e 's/\][[:space:]]*$//' -e 's/"//g' -e 's/,/ /g' || true)
        [ -n "$_kw" ] && { printf '%s\n' "$_kw"; return 0; }
    fi
    printf ''
}

# -----------------------------------------------------------------------------
# Helper: Parse existing catalog.json into memory/lookup
# -----------------------------------------------------------------------------
_catalog_read_existing_json() {
    _json_file="$1"
    [ -f "$_json_file" ] || return 1

    _in_obj=0; _in_topics=0
    _cur_name=""; _cur_forge=""; _cur_slug=""; _cur_desc=""
    _cur_website=""; _cur_upstream="false"; _cur_lic=""; _cur_topics=""

    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in
            *"{"*)
                _in_obj=1; _in_topics=0
                _cur_name=""; _cur_forge=""; _cur_slug=""; _cur_desc=""
                _cur_website=""; _cur_upstream="false"; _cur_lic=""; _cur_topics=""
                ;;
            *"}"*)
                if [ "$_in_obj" -eq 1 ] && [ -n "$_cur_name" ]; then
                    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
                        "$_cur_name" "$_cur_forge" "$_cur_slug" "$_cur_lic" \
                        "$_cur_upstream" "$_cur_website" "$_cur_topics" "$_cur_desc"
                fi
                _in_obj=0; _in_topics=0
                ;;
            *\"name\":*)
                _cur_name=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"name":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"forge\":*)
                _cur_forge=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"forge":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"slug\":*)
                _cur_slug=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"slug":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"description\":*)
                _cur_desc=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"description":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"license\":*)
                _cur_lic=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"license":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"website\":*)
                _cur_website=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"website":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"is_upstream\":*)
                case "$_line" in
                    *true*) _cur_upstream="true" ;;
                    *) _cur_upstream="false" ;;
                esac
                ;;
            *\"topics\":*)
                _in_topics=1
                ;;
            *\]*)
                _in_topics=0
                ;;
            *)
                if [ "$_in_topics" -eq 1 ]; then
                    _t=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                    if [ -n "$_t" ]; then
                        _cur_topics="${_cur_topics}${_cur_topics:+, }$_t"
                    fi
                fi
                ;;
        esac
    done < "$_json_file"
}

# -----------------------------------------------------------------------------
# Subcommand: sync-local
# Scans workspace, updates .git/description, creates catalog.json & README.md
# -----------------------------------------------------------------------------
_catalog_sync_local() {
    _src_dir="$1"
    if [ ! -d "$_src_dir" ]; then
        echo "Error: Directory '$_src_dir' does not exist." >&2
        return 1
    fi

    echo "-> [ГОСПЛАН] Sincronizando .git/description em cada repositório de $_src_dir..."
    _cat_json="${_src_dir}/catalog.json"
    _cat_readme="${_src_dir}/README.md"
    _existing_records=""
    if [ -f "$_cat_json" ]; then
        _existing_records=$(_catalog_read_existing_json "$_cat_json" 2>/dev/null || true)
    fi

    _tmp_json="${_cat_json}.tmp.$$"
    _tmp_md="${_cat_readme}.tmp.$$"

    printf '[\n' > "$_tmp_json"
    printf '# Catálogo de Projetos (`%s`)\n\n' "$_src_dir" > "$_tmp_md"
    printf 'Índice estruturado de todos os repositórios locais e suas publicações remotas.\n\n' >> "$_tmp_md"
    printf '| Projeto | Forge | Licença | Descrição | Tags |\n' >> "$_tmp_md"
    printf '| :--- | :---: | :---: | :--- | :--- |\n' >> "$_tmp_md"

    _first_repo=1
    for _repo_entry in "$_src_dir"/*; do
        [ -d "$_repo_entry" ] || continue
        [ -d "$_repo_entry/.git" ] || [ -f "$_repo_entry/.git" ] || continue

        _rname=$(basename "$_repo_entry")
        _rforge="unknown"
        _rslug="$_rname"
        _rupstream="false"
        _rdesc=""
        _rlicense=""
        _rwebsite=""
        _rtopics=""

        # Check existing metadata from previous catalog.json
        if [ -n "$_existing_records" ]; then
            _rec=$(printf '%s\n' "$_existing_records" | grep "^${_rname}[[:space:]]" | head -n 1 || true)
            if [ -n "$_rec" ]; then
                _rforge=$(printf '%s' "$_rec" | cut -f2)
                _rslug=$(printf '%s' "$_rec" | cut -f3)
                _rlicense=$(printf '%s' "$_rec" | cut -f4)
                _rupstream=$(printf '%s' "$_rec" | cut -f5)
                _rwebsite=$(printf '%s' "$_rec" | cut -f6)
                _rtopics=$(printf '%s' "$_rec" | cut -f7)
                _rdesc=$(printf '%s' "$_rec" | cut -f8-)
            fi
        fi

        # Detect forge and slug from git remote if unconfigured or generic
        _origin_url=$(git -C "$_repo_entry" config remote.origin.url 2>/dev/null || true)
        if [ -n "$_origin_url" ]; then
            _detected=$(_catalog_detect_forge_and_slug "$_origin_url")
            _det_forge=$(printf '%s' "$_detected" | cut -d' ' -f1)
            _det_slug=$(printf '%s' "$_detected" | cut -d' ' -f2)
            if [ "$_rforge" = "unknown" ] || [ -z "$_rforge" ]; then
                _rforge="$_det_forge"
            fi
            if [ "$_rslug" = "$_rname" ] || [ -z "$_rslug" ]; then
                _rslug="$_det_slug"
            fi
        fi

        # Extract description from git/files if empty
        if [ -z "$_rdesc" ]; then
            _rdesc=$(_catalog_extract_description "$_repo_entry")
        fi
        [ -z "$_rdesc" ] && _rdesc="Repositório Git ${_rname}"

        # Write description to .git/description if writable
        if [ -d "$_repo_entry/.git" ]; then
            printf '%s\n' "$_rdesc" > "$_repo_entry/.git/description" 2>/dev/null || true
            echo "  [OK] ${_rname}/.git/description"
        else
            echo "  [OK] ${_rname} (atualizado)"
        fi

        # Extract license if empty
        if [ -z "$_rlicense" ]; then
            _rlicense=$(_catalog_extract_license "$_repo_entry")
        fi

        # Extract website if empty
        if [ -z "$_rwebsite" ]; then
            _rwebsite=$(_catalog_extract_website "$_repo_entry")
        fi

        # Extract topics if empty
        if [ -z "$_rtopics" ]; then
            _rtopics=$(_catalog_extract_topics "$_repo_entry")
        fi

        # Format JSON topics array
        _topics_json=""
        if [ -n "$_rtopics" ]; then
            _tfirst=1
            # Replace comma separators with spaces
            _clean_topics=$(printf '%s' "$_rtopics" | tr ',' ' ')
            for _top in $_clean_topics; do
                [ -n "$_top" ] || continue
                [ "$_tfirst" -eq 1 ] && _tfirst=0 || _topics_json="${_topics_json}, "
                _topics_json="${_topics_json}\"${_top}\""
            done
        fi

        # Escape quotes in description for JSON
        _desc_json=$(printf '%s' "$_rdesc" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')

        # Append to catalog.json
        if [ "$_first_repo" -eq 1 ]; then
            _first_repo=0
        else
            printf ',\n' >> "$_tmp_json"
        fi

        cat >> "$_tmp_json" <<EOF
  {
    "name": "${_rname}",
    "forge": "${_rforge}",
    "slug": "${_rslug}",
    "description": "${_desc_json}",
    "topics": [${_topics_json}],
    "license": "${_rlicense}",
    "website": "${_rwebsite}",
    "is_upstream": ${_rupstream}
  }
EOF

        # Append to README.md
        _forge_icon="$_rforge"
        case "$_rforge" in
            *github.com*|github)   _forge_icon="🐙 GitHub" ;;
            *codeberg.org*|codeberg) _forge_icon="🏔️ Codeberg" ;;
            *tildegit.org*|tildegit) _forge_icon="🟣 TildeGit" ;;
            *gitlab.com*|*framagit.org*|gitlab)   _forge_icon="🦊 GitLab" ;;
            *sr.ht*|sourcehut) _forge_icon="🛖 SourceHut" ;;
        esac

        _bt=$(printf '\140')
        _tags_md=""
        _tag_count=0
        for _top in $(printf '%s' "$_rtopics" | tr ',' ' '); do
            [ -n "$_top" ] || continue
            _tags_md="${_tags_md} ${_bt}${_top}${_bt}"
            _tag_count=$((_tag_count + 1))
            [ "$_tag_count" -ge 4 ] && break
        done

        printf '| **[%s](./%s)** | %s | %s%s%s | %s |%s |\n' \
            "$_rname" "$_rname" "$_forge_icon" "$_bt" "$_rlicense" "$_bt" "$_rdesc" "$_tags_md" >> "$_tmp_md"
    done

    printf '\n]\n' >> "$_tmp_json"

    mv "$_tmp_json" "$_cat_json"
    mv "$_tmp_md" "$_cat_readme"

    echo "  [OK] Gravado $_cat_json"
    echo "  [OK] Gravado $_cat_readme"
    return 0
}

# -----------------------------------------------------------------------------
# Subcommand: sync-remote
# Sincroniza metadados com as forjas remotas (Codeberg, TildeGit, GitHub)
# -----------------------------------------------------------------------------
_catalog_sync_remote() {
    _src_dir="$1"
    _cat_json="${_src_dir}/catalog.json"
    if [ ! -f "$_cat_json" ]; then
        echo "Error: Catálogo '$_cat_json' não encontrado. Execute 'cccp catalog sync-local' primeiro." >&2
        return 1
    fi

    echo ""
    echo "-> [ГОСПЛАН] Sincronizando metadados com os servidores remotos (Codeberg, TildeGit, GitHub)..."

    _records=$(_catalog_read_existing_json "$_cat_json" 2>/dev/null || true)
    if [ -z "$_records" ]; then
        echo "Aviso: Nenhum repositório encontrado em $_cat_json."
        return 0
    fi

    echo "$_records" | while IFS= read -r _entry || [ -n "$_entry" ]; do
        [ -n "$_entry" ] || continue
        _name=$(printf '%s' "$_entry" | cut -f1)
        _forge=$(printf '%s' "$_entry" | cut -f2)
        _slug=$(printf '%s' "$_entry" | cut -f3)
        _license=$(printf '%s' "$_entry" | cut -f4)
        _is_upstream=$(printf '%s' "$_entry" | cut -f5)
        _website=$(printf '%s' "$_entry" | cut -f6)
        _topics=$(printf '%s' "$_entry" | cut -f7)
        _desc=$(printf '%s' "$_entry" | cut -f8-)

        if [ "$_is_upstream" = "true" ]; then
            echo "  [INFO] ${_name} (${_forge}): Repositório upstream de terceiros (${_slug}). Pulando metadados remotos."
            continue
        fi

        _forgeapp=""
        if command -v detect_forgeapp >/dev/null 2>&1; then
            _forgeapp=$(detect_forgeapp "$_forge" 2>/dev/null || true)
        fi

        case "$_forgeapp" in
            forgeapp_forgejo)
                _domain="$_forge"
                case "$_domain" in
                    codeberg) _domain="codeberg.org" ;;
                    tildegit) _domain="tildegit.org" ;;
                    forgejo) _domain="${FORGEJO_HOST:-codeberg.org}" ;;
                esac
                _base_url="https://${_domain}"

                _token=$(_catalog_resolve_token "$_domain" 2>/dev/null || true)
                [ -z "$_token" ] && _token=$(_catalog_resolve_token "$_forge" 2>/dev/null || true)
                if [ -z "$_token" ]; then
                    echo "  [AVISO] ${_name} (${_forge}): Token de autenticação não configurado para ${_forge}. Pulando."
                    continue
                fi

                # 1. Update repo description & website
                _esc_desc=$(escape_json_string "$_desc")
                _esc_web=$(escape_json_string "$_website")
                _payload="{\"description\":\"${_esc_desc}\""
                if [ -n "$_esc_web" ]; then
                    _payload="${_payload},\"website\":\"${_esc_web}\""
                fi
                _payload="${_payload}}"

                _res=$(_catalog_http -s -o /dev/null -w "%{http_code}" -X PATCH \
                    -H "Authorization: token ${_token}" \
                    -H "Content-Type: application/json" \
                    -H "Accept: application/json" \
                    -H "User-Agent: CCCP-Catalog/1.0" \
                    -d "$_payload" \
                    "${_base_url}/api/v1/repos/${_slug}" 2>/dev/null || echo "000")

                # 2. Update topics
                _topics_arr=""
                _tfirst=1
                for _t in $(printf '%s' "$_topics" | tr ',' ' '); do
                    [ -n "$_t" ] || continue
                    [ "$_tfirst" -eq 1 ] && _tfirst=0 || _topics_arr="${_topics_arr},"
                    _topics_arr="${_topics_arr}\"${_t}\""
                done
                _top_payload="{\"topics\":[${_topics_arr}]}"

                _res_top=$(_catalog_http -s -o /dev/null -w "%{http_code}" -X PUT \
                    -H "Authorization: token ${_token}" \
                    -H "Content-Type: application/json" \
                    -H "Accept: application/json" \
                    -H "User-Agent: CCCP-Catalog/1.0" \
                    -d "$_top_payload" \
                    "${_base_url}/api/v1/repos/${_slug}/topics" 2>/dev/null || echo "000")

                case "$_res" in
                    200|201|204)
                        echo "  [OK] ${_forge}: ${_slug} (descrição e tópicos sincronizados)"
                        ;;
                    *)
                        echo "  [ERRO] ${_forge} ${_slug}: HTTP ${_res}"
                        ;;
                esac
                ;;

            forgeapp_github|github)
                if command -v gh >/dev/null 2>&1; then
                    _gh_args="repo edit ${_slug} --description \"${_desc}\""
                    if [ -n "$_website" ]; then
                        _gh_args="${_gh_args} --homepage \"${_website}\""
                    fi
                    for _t in $(printf '%s' "$_topics" | tr ',' ' '); do
                        [ -n "$_t" ] || continue
                        _gh_args="${_gh_args} --add-topic \"${_t}\""
                    done

                    if eval "gh ${_gh_args} >/dev/null 2>&1"; then
                        echo "  [OK] GitHub: ${_slug} (descrição e tópicos via gh)"
                    else
                        echo "  [ERRO] GitHub: ${_slug} (falha ao executar gh repo edit)"
                    fi
                else
                    _token=$(_catalog_resolve_token "github.com" 2>/dev/null || true)
                    [ -z "$_token" ] && _token=$(_catalog_resolve_token "$_forge" 2>/dev/null || true)
                    if [ -z "$_token" ]; then
                        echo "  [AVISO] ${_name} (github): nem 'gh' nem GITHUB_TOKEN configurados. Pulando."
                        continue
                    fi

                    _esc_desc=$(escape_json_string "$_desc")
                    _esc_web=$(escape_json_string "$_website")
                    _payload="{\"description\":\"${_esc_desc}\""
                    [ -n "$_esc_web" ] && _payload="${_payload},\"homepage\":\"${_esc_web}\""
                    _payload="${_payload}}"

                    _res=$(_catalog_http -s -o /dev/null -w "%{http_code}" -X PATCH \
                        -H "Authorization: token ${_token}" \
                        -H "Accept: application/vnd.github+json" \
                        -H "User-Agent: CCCP-Catalog/1.0" \
                        -d "$_payload" \
                        "https://api.github.com/repos/${_slug}" 2>/dev/null || echo "000")

                    case "$_res" in
                        200|201|204)
                            echo "  [OK] GitHub: ${_slug} (descrição atualizada via REST)"
                            ;;
                        *)
                            echo "  [ERRO] GitHub ${_slug}: HTTP ${_res}"
                            ;;
                    esac
                fi
                ;;

            *)
                echo "  [INFO] ${_name} (${_forge}): sincronização remota não suportada para ${_forge}."
                ;;
        esac
    done
    return 0
}

# -----------------------------------------------------------------------------
# Subcommand: list
# Lists cataloged repositories from catalog.json or scans dynamically
# -----------------------------------------------------------------------------
_catalog_list() {
    _src_dir="$1"
    _as_json="${2:-0}"
    _cat_json="${_src_dir}/catalog.json"

    if [ "$_as_json" -eq 1 ]; then
        if [ -f "$_cat_json" ]; then
            cat "$_cat_json"
            return 0
        fi
    fi

    if [ -f "$_cat_json" ]; then
        _records=$(_catalog_read_existing_json "$_cat_json" 2>/dev/null || true)
        if [ -z "$_records" ]; then
            echo "Catálogo vazio em $_cat_json"
            return 0
        fi

        printf '%-20s %-10s %-30s %-12s %s\n' "PROJETO" "FORGE" "SLUG" "LICENÇA" "DESCRIÇÃO"
        printf '%-20s %-10s %-30s %-12s %s\n' "-------" "-----" "----" "-------" "---------"
        echo "$_records" | while IFS= read -r _line || [ -n "$_line" ]; do
            [ -n "$_line" ] || continue
            _p_name=$(printf '%s' "$_line" | cut -f1)
            _p_forge=$(printf '%s' "$_line" | cut -f2)
            _p_slug=$(printf '%s' "$_line" | cut -f3)
            _p_lic=$(printf '%s' "$_line" | cut -f4)
            _p_desc=$(printf '%s' "$_line" | cut -f8-)
            printf '%-20s %-10s %-30s %-12s %s\n' "$_p_name" "$_p_forge" "$_p_slug" "$_p_lic" "$_p_desc"
        done
        return 0
    fi

    # Fallback: scan on the fly
    echo "Nenhum catalog.json encontrado. Varrendo $_src_dir..."
    _catalog_sync_local "$_src_dir"
}

# -----------------------------------------------------------------------------
# Helper: Resolve Catalog Target Directory based on Strict Precedence:
# 1. Command-line flag (--dir / -d)
# 2. Environment variable (CATALOG_DIR / CCCP_CATALOG_DIR)
# 3. Global cccp configuration (catalog_dir in config file)
# 4. Fallback value: ~/src
# -----------------------------------------------------------------------------
_catalog_resolve_target_dir() {
    _cli_dir="${1:-}"
    _resolved=""

    # 1. Command-line flag
    if [ -n "$_cli_dir" ]; then
        _resolved="$_cli_dir"
    # 2. Environment variable
    elif [ -n "${CATALOG_DIR:-}" ]; then
        _resolved="$CATALOG_DIR"
    elif [ -n "${CCCP_CATALOG_DIR:-}" ]; then
        _resolved="$CCCP_CATALOG_DIR"
    # 3. Global CCCP configuration
    else
        _cfg_dir=""
        _local_cfg=""
        if command -v get_local_config_file >/dev/null 2>&1; then
            _local_cfg=$(get_local_config_file 2>/dev/null || true)
        fi
        if [ -n "$_local_cfg" ] && [ -f "$_local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
            _cfg_dir=$(read_file_key "$_local_cfg" "catalog_dir" 2>/dev/null || true)
        fi
        if [ -z "$_cfg_dir" ]; then
            _global_cfg=""
            if command -v get_global_config_file >/dev/null 2>&1; then
                _global_cfg=$(get_global_config_file 2>/dev/null || true)
            else
                _global_cfg="${XDG_CONFIG_HOME:-${HOME}/.config}/cccp/config"
            fi
            if [ -n "$_global_cfg" ] && [ -f "$_global_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
                _cfg_dir=$(read_file_key "$_global_cfg" "catalog_dir" 2>/dev/null || true)
            fi
        fi

        if [ -n "$_cfg_dir" ]; then
            _resolved="$_cfg_dir"
        else
            # 4. Fallback: ~/src
            _resolved="~/src"
        fi
    fi

    # Expand leading tilde (~) if present
    case "$_resolved" in
        "~") _resolved="$HOME" ;;
        "~/"*) _resolved="${HOME}/${_resolved#\~/}" ;;
    esac

    printf '%s\n' "$_resolved"
}

# -----------------------------------------------------------------------------
# Main Subcommand Dispatcher: cmd_catalog
# -----------------------------------------------------------------------------
cmd_catalog() {
    _action="list"
    _cli_dir=""
    _as_json=0

    while [ $# -gt 0 ]; do
        case "$1" in
            list)
                _action="list"
                shift
                ;;
            local|sync-local)
                _action="sync-local"
                shift
                ;;
            remote|sync-remote)
                _action="sync-remote"
                shift
                ;;
            all)
                _action="all"
                shift
                ;;
            --json)
                _as_json=1
                shift
                ;;
            --dir|-d)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing directory argument for $1" >&2; return 1; }
                _cli_dir="$1"
                shift
                ;;
            -h|--help|help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "catalog"
                else
                    echo "Usage: cccp catalog [list|sync-local|sync-remote|all] [--dir <path>] [--json]"
                fi
                return 0
                ;;
            *)
                echo "Error: Unknown catalog subcommand or option '$1'" >&2
                return 1
                ;;
        esac
    done

    _target_dir=$(_catalog_resolve_target_dir "$_cli_dir")

    case "$_action" in
        list)
            _catalog_list "$_target_dir" "$_as_json"
            ;;
        sync-local)
            _catalog_sync_local "$_target_dir"
            ;;
        sync-remote)
            _catalog_sync_remote "$_target_dir"
            ;;
        all)
            _catalog_sync_local "$_target_dir"
            _catalog_sync_remote "$_target_dir"
            ;;
    esac
}
