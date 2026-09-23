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
    if [ -f "$GIT_ROOT/src/utils/tickets.sh" ]; then
        . "$GIT_ROOT/src/utils/tickets.sh"
    fi
fi

# -----------------------------------------------------------------------------
# Helper: Resolve Gemini API Key (Memory -> ~/.env -> .cccprc)
# -----------------------------------------------------------------------------
_diva_resolve_api_key() {
    # 1. Environment / In-memory variable
    if [ -n "${GEMINI_API_KEY:-}" ]; then
        printf '%s\n' "$GEMINI_API_KEY"
        return 0
    fi

    # 2. Check ~/.env (or local .env)
    for _env_file in "${HOME}/.env" ".env"; do
        if [ -f "$_env_file" ]; then
            if command -v read_file_key >/dev/null 2>&1; then
                if _k=$(read_file_key "$_env_file" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                    printf '%s\n' "$_k"
                    return 0
                fi
            fi
            _env_val=$(sed -n -E 's/^[[:space:]]*(export[[:space:]]+)?GEMINI_API_KEY[[:space:]]*=[[:space:]]*"?([^"#]+)"?.*/\2/p' "$_env_file" 2>/dev/null | sed -e 's/[[:space:]]*$//' -e "s/^'//" -e "s/'$//" | head -n 1 || true)
            if [ -n "$_env_val" ]; then
                printf '%s\n' "$_env_val"
                return 0
            fi
        fi
    done

    # 3. Check local config (.cccprc)
    _local_cfg=""
    if command -v get_local_config_file >/dev/null 2>&1; then
        _local_cfg=$(get_local_config_file 2>/dev/null || true)
    fi
    [ -z "$_local_cfg" ] && [ -f ".cccprc" ] && _local_cfg=".cccprc"

    if [ -n "$_local_cfg" ] && [ -f "$_local_cfg" ]; then
        if command -v read_file_key >/dev/null 2>&1; then
            if _k=$(read_file_key "$_local_cfg" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                printf '%s\n' "$_k"
                return 0
            fi
        fi
    fi

    # 4. Check global config (~/.config/cccp/.cccprc)
    _global_cfg=""
    if command -v get_global_config_file >/dev/null 2>&1; then
        _global_cfg=$(get_global_config_file 2>/dev/null || true)
    fi
    if [ -n "$_global_cfg" ] && [ -f "$_global_cfg" ]; then
        if command -v read_file_key >/dev/null 2>&1; then
            if _k=$(read_file_key "$_global_cfg" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                printf '%s\n' "$_k"
                return 0
            fi
        fi
    fi

    return 1
}

# -----------------------------------------------------------------------------
# Helper: Resolve Forge & Token for Active Repository
# -----------------------------------------------------------------------------
_diva_resolve_forge() {
    if command -v detect_repo_info >/dev/null 2>&1; then
        detect_repo_info
    else
        return 1
    fi
}

_diva_resolve_forge_token() {
    _forge="$1"
    if command -v _catalog_resolve_token >/dev/null 2>&1; then
        _catalog_resolve_token "$_forge" && return 0
    fi
    case "$_forge" in
        codeberg)
            [ -n "${CODEBERG_TOKEN:-}" ] && { printf '%s\n' "$CODEBERG_TOKEN"; return 0; }
            ;;
        tildegit)
            [ -n "${TILDEGIT_TOKEN:-}" ] && { printf '%s\n' "$TILDEGIT_TOKEN"; return 0; }
            ;;
        github)
            [ -n "${GITHUB_TOKEN:-}" ] && { printf '%s\n' "$GITHUB_TOKEN"; return 0; }
            [ -n "${GH_TOKEN:-}" ] && { printf '%s\n' "$GH_TOKEN"; return 0; }
            ;;
    esac
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Escape String for JSON (Pure POSIX)
# -----------------------------------------------------------------------------
_diva_escape_json() {
    _input="$1"
    _out=""
    _first=1
    while IFS= read -r _l || [ -n "$_l" ]; do
        _esc_l=$(printf '%s' "$_l" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' -e 's/\r//g')
        if [ "$_first" -eq 1 ]; then
            _out="$_esc_l"
            _first=0
        else
            _out="${_out}\\n${_esc_l}"
        fi
    done <<EOF
$_input
EOF
    printf '%s' "$_out"
}

# -----------------------------------------------------------------------------
# Helper: Call Google Gemini REST API
# -----------------------------------------------------------------------------
_diva_call_gemini() {
    _api_key="$1"
    _model="$2"
    _user_text="$3"
    _repo_context="${4:-}"

    _esc_text=$(_diva_escape_json "$_user_text")
    _esc_ctx=$(_diva_escape_json "$_repo_context")

    _sys_instructions="You are a senior software engineering triage assistant (O Divã do Gosplan). \
Your task is to analyze user reports, bug descriptions, and feature requests, and decompose them into one or more discrete, atomic issue tickets following Conventional Commits format for titles (e.g. feat(scope): title, fix(scope): title, refactor(scope): title). \
Split multi-topic or unrelated problems into separate tickets. \
Return ONLY a valid JSON object matching this schema: \
{\\\"tickets\\\": [{\\\"title\\\": \\\"Conventional Commit title\\\", \\\"rationale\\\": \\\"Why this is a distinct issue\\\", \\\"labels\\\": [\\\"bug\\\"], \\\"body\\\": \\\"## Description\\\\n\\\\n## Steps to Reproduce\\\\n\\\\n## Expected vs Actual\\\\n\\\"}]} \
Write the ticket body and descriptions in the language used by the user report."

    _prompt="Repository context: ${_esc_ctx}\n\nUser input to triage:\n${_esc_text}"

    _req_body=$(cat <<EOF
{
  "system_instruction": {
    "parts": [{ "text": "${_sys_instructions}" }]
  },
  "contents": [
    {
      "parts": [{ "text": "${_prompt}" }]
    }
  ],
  "generationConfig": {
    "responseMimeType": "application/json",
    "temperature": 0.2
  }
}
EOF
)

    _url="https://generativelanguage.googleapis.com/v1beta/models/${_model}:generateContent?key=${_api_key}"

    _res=$(curl -s -f -X POST \
        -H "Content-Type: application/json" \
        -d "$_req_body" \
        "$_url" 2>/dev/null || true)

    if [ -z "$_res" ]; then
        return 1
    fi

    printf '%s\n' "$_res"
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Parse Gemini Response and Yield Structured Tickets
# -----------------------------------------------------------------------------
_diva_extract_tickets() {
    _raw_gemini_resp="$1"

    # Extract JSON text part
    _json_text=""
    if command -v jq >/dev/null 2>&1; then
        _json_text=$(printf '%s' "$_raw_gemini_resp" | jq -r '.candidates[0].content.parts[0].text // empty' 2>/dev/null || true)
    else
        # Pure POSIX fallback
        _json_text=$(printf '%s\n' "$_raw_gemini_resp" | sed -n '/[[:space:]]*"text":[[:space:]]*"/,$p' | sed -e '1s/^[[:space:]]*"text":[[:space:]]*"//' -e 's/"[[:space:]]*$//' -e 's/\\n/\n/g' -e 's/\\"/"/g' || true)
    fi

    if [ -z "$_json_text" ]; then
        return 1
    fi

    printf '%s\n' "$_json_text"
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Format and Print Column-Wrapped Box Row with Closed Right Border
# Width: 79 columns (│ + 1 space + 75 chars content + 1 space + │)
# -----------------------------------------------------------------------------
_diva_box_field() {
    _b_prefix="$1"
    _b_text="$2"
    _b_cont="              "
    _b_first=1

    _trimmed=$(printf '%s' "$_b_text" | tr -d ' \t\n')
    if [ -z "$_trimmed" ]; then
        _spaces=$(printf "%*s" 61 "")
        printf "│ %s%s │\n" "$_b_prefix" "$_spaces"
        return 0
    fi

    printf '%s\n' "$_b_text" | fold -s -w 61 | while IFS= read -r _b_chunk || [ -n "$_b_chunk" ]; do
        _b_chunk=$(printf '%s' "$_b_chunk" | sed 's/[[:space:]]*$//')
        [ -z "$_b_chunk" ] && continue

        if [ "$_b_first" -eq 1 ]; then
            _pfx="$_b_prefix"
            _b_first=0
        else
            _pfx="$_b_cont"
        fi

        _clen=$(printf '%s' "$_b_chunk" | wc -m | tr -d ' ')
        _pad=$((61 - _clen))
        [ "$_pad" -lt 0 ] && _pad=0
        _spaces=""
        [ "$_pad" -gt 0 ] && _spaces=$(printf "%*s" "$_pad" "")
        printf "│ %s%s%s │\n" "$_pfx" "$_b_chunk" "$_spaces"
    done
}

# -----------------------------------------------------------------------------
# Helper: Format and Print Body Preview Lines with Closed Right Border
# -----------------------------------------------------------------------------
_diva_box_body() {
    _bb_body="$1"
    echo "│ 📝 Prévia do Corpo:                                                         │"
    [ -z "$_bb_body" ] && return 0

    printf '%s\n' "$_bb_body" | while IFS= read -r _bb_line || [ -n "$_bb_line" ]; do
        _bb_trimmed=$(printf '%s' "$_bb_line" | tr -d ' \t\n')
        if [ -z "$_bb_trimmed" ]; then
            _bb_spaces=$(printf "%*s" 75 "")
            printf "│ %s │\n" "$_bb_spaces"
        else
            printf '%s\n' "$_bb_line" | fold -s -w 72 | while IFS= read -r _bb_chunk || [ -n "$_bb_chunk" ]; do
                _bb_chunk=$(printf '%s' "$_bb_chunk" | sed 's/[[:space:]]*$//')
                [ -z "$_bb_chunk" ] && continue
                _clen=$(printf '%s' "$_bb_chunk" | wc -m | tr -d ' ')
                _pad=$((72 - _clen))
                [ "$_pad" -lt 0 ] && _pad=0
                _bb_spaces=""
                [ "$_pad" -gt 0 ] && _bb_spaces=$(printf "%*s" "$_pad" "")
                printf "│    %s%s │\n" "$_bb_chunk" "$_bb_spaces"
            done
        fi
    done
}

# -----------------------------------------------------------------------------
# Helper: Display Soviet Ticket Box & Request Interactive Confirmation
# -----------------------------------------------------------------------------
_diva_present_and_dispatch() {
    _tickets_json="$1"
    _forge="$2"
    _owner="$3"
    _repo="$4"
    _dry_run="${5:-0}"

    _count=0
    if command -v jq >/dev/null 2>&1; then
        _count=$(printf '%s' "$_tickets_json" | jq '.tickets | length' 2>/dev/null || echo 0)
    else
        _count=$(printf '%s' "$_tickets_json" | grep -c '"title":' || echo 0)
    fi

    if [ "$_count" -le 0 ]; then
        echo "Aviso: Nenhum ticket foi identificado pelo modelo."
        return 0
    fi

    echo ""
    echo "★ [ГОСПЛАН] O Divã identificou $_count ticket(s) para abertura ★"
    echo ""

    _created_count=0

    _idx=0
    while [ "$_idx" -lt "$_count" ]; do
        _title=""
        _rationale=""
        _labels=""
        _body=""

        if command -v jq >/dev/null 2>&1; then
            _title=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].title // empty")
            _rationale=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].rationale // empty")
            _labels=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].labels // [] | join(\", \")")
            _body=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].body // empty")
        else
            # Simple POSIX fallback extraction for ticket block
            _title="Ticket $((_idx + 1))"
            _body=""
        fi

        _num=$((_idx + 1))
        echo "┌─────────────────────────────────────────────────────────────────────────────┐"
        _hdr=$(printf "[ДИВАН] Ticket %d/%d" "$_num" "$_count")
        _hlen=$(printf '%s' "$_hdr" | wc -m | tr -d ' ')
        _hpad=$((75 - _hlen))
        [ "$_hpad" -lt 0 ] && _hpad=0
        _hspaces=""
        [ "$_hpad" -gt 0 ] && _hspaces=$(printf "%*s" "$_hpad" "")
        printf "│ %s%s │\n" "$_hdr" "$_hspaces"
        echo "├─────────────────────────────────────────────────────────────────────────────┤"
        if [ -n "$_owner" ] && [ -n "$_repo" ]; then
            _diva_box_field "🎯 Destino  : " "${_owner}/${_repo} (${_forge})"
        fi
        _diva_box_field "📌 Título   : " "$_title"
        if [ -n "$_rationale" ]; then
            _diva_box_field "💡 Racional : " "$_rationale"
        fi
        if [ -n "$_labels" ]; then
            _diva_box_field "🏷️ Labels   : " "$_labels"
        fi
        _diva_box_body "$_body"
        echo "└─────────────────────────────────────────────────────────────────────────────┘"

        if [ "$_dry_run" -eq 1 ]; then
            echo "  [DRY-RUN] Simulação ativa. Ticket não publicado."
            _idx=$((_idx + 1))
            continue
        fi

        # Interactive confirmation
        _choice="y"
        if [ -t 0 ]; then
            if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                printf "  [y] Criar em %s/%s (%s)  [n] Pular  [e] Editar título  [f] Ver corpo completo → " "$_owner" "$_repo" "$_forge"
            else
                printf "  [y] Criar  [n] Pular  [e] Editar título  [f] Ver corpo completo → "
            fi
            read -r _user_choice || _user_choice="n"
            _choice=$(printf '%s' "$_user_choice" | tr '[:upper:]' '[:lower:]')
        fi

        case "$_choice" in
            y|yes|"")
                echo "  -> Publicando ticket na forja ${_forge} (${_owner}/${_repo})..."
                _issue_url=""

                case "$_forge" in
                    codeberg|tildegit)
                        _base_url="https://codeberg.org"
                        [ "$_forge" = "tildegit" ] && _base_url="https://tildegit.org"

                        _token=$(_diva_resolve_forge_token "$_forge" 2>/dev/null || true)
                        if [ -z "$_token" ]; then
                            echo "  [ERRO] Token da forja ${_forge} não encontrado. Não foi possível publicar."
                            _idx=$((_idx + 1))
                            continue
                        fi

                        _esc_t=$(printf '%s' "$_title" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')
                        _esc_b=$(printf '%s' "$_body" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' | tr '\n' ' ')
                        _issue_payload="{\"title\":\"${_esc_t}\",\"body\":\"${_esc_b}\"}"

                        _post_res=$(curl -s -X POST \
                            -H "Authorization: token ${_token}" \
                            -H "Content-Type: application/json" \
                            -H "Accept: application/json" \
                            -d "$_issue_payload" \
                            "${_base_url}/api/v1/repos/${_owner}/${_repo}/issues" 2>/dev/null || true)

                        if command -v jq >/dev/null 2>&1; then
                            _issue_url=$(printf '%s' "$_post_res" | jq -r '.html_url // empty')
                        fi
                        if [ -n "$_issue_url" ]; then
                            echo "  ✅ Criado: ${_issue_url}"
                            _created_count=$((_created_count + 1))
                        else
                            echo "  ✅ Issue enviada para ${_forge}."
                            _created_count=$((_created_count + 1))
                        fi
                        ;;

                    github)
                        if command -v gh >/dev/null 2>&1; then
                            _gh_url=$(gh issue create --repo "${_owner}/${_repo}" --title "$_title" --body "$_body" 2>/dev/null || true)
                            if [ -n "$_gh_url" ]; then
                                echo "  ✅ Criado: ${_gh_url}"
                                _created_count=$((_created_count + 1))
                            fi
                        else
                            _token=$(_diva_resolve_forge_token "github" 2>/dev/null || true)
                            if [ -n "$_token" ]; then
                                _esc_t=$(printf '%s' "$_title" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')
                                _esc_b=$(printf '%s' "$_body" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' | tr '\n' ' ')
                                _post_res=$(curl -s -X POST \
                                    -H "Authorization: token ${_token}" \
                                    -H "Content-Type: application/json" \
                                    -H "Accept: application/vnd.github+json" \
                                    -d "{\"title\":\"${_esc_t}\",\"body\":\"${_esc_b}\"}" \
                                    "https://api.github.com/repos/${_owner}/${_repo}/issues" 2>/dev/null || true)
                                echo "  ✅ Issue enviada para GitHub."
                                _created_count=$((_created_count + 1))
                            else
                                echo "  [ERRO] nem 'gh' nem GITHUB_TOKEN disponíveis para publicação."
                            fi
                        fi
                        ;;
                    *)
                        echo "  [INFO] Publicação automática não suportada para ${_forge}."
                        ;;
                esac
                ;;

            f|full)
                echo ""
                echo "--- CORPO COMPLETO ---"
                printf '%s\n' "$_body"
                echo "-----------------------"
                # Ask again without advancing index
                continue
                ;;

            e|edit)
                printf "  Novo título [%s]: " "$_title"
                read -r _new_title < /dev/tty || _new_title=""
                if [ -n "$_new_title" ]; then
                    _title="$_new_title"
                fi
                continue
                ;;

            n|no|*)
                echo "  ⏭️  Pulado."
                ;;
        esac

        _idx=$((_idx + 1))
    done

    # Re-sync local tickets if tickets were created
    if [ "$_created_count" -gt 0 ]; then
        if command -v cmd_tickets >/dev/null 2>&1; then
            echo ""
            echo "-> Sincronizando .antigravity/tickets.md..."
            cmd_tickets >/dev/null 2>&1 || true
        fi
    fi

    echo ""
    echo "Sessão concluída. $_created_count ticket(s) publicado(s)."
    return 0
}

# -----------------------------------------------------------------------------
# Main Subcommand Dispatcher: cmd_diva
# -----------------------------------------------------------------------------
cmd_diva() {
    _model="${GEMINI_MODEL:-gemini-3.6-flash}"
    _dry_run=0
    _user_text=""
    _custom_repo=""

    while [ $# -gt 0 ]; do
        case "$1" in
            config)
                shift
                if [ $# -gt 0 ]; then
                    _key="$1"
                    _local_cfg=$(get_local_config_file 2>/dev/null || true)
                    [ -z "$_local_cfg" ] && _local_cfg="${HOME}/.cccprc"
                    write_file_key "$_local_cfg" "gemini_api_key" "$_key"
                    echo "Chave gemini_api_key salva com sucesso em $_local_cfg"
                    return 0
                else
                    echo "Uso: cccp diva config <GEMINI_API_KEY>"
                    return 1
                fi
                ;;
            --dry-run)
                _dry_run=1
                shift
                ;;
            --model|-m)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for $1" >&2; return 1; }
                _model="$1"
                shift
                ;;
            --repo|-r)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for $1" >&2; return 1; }
                _custom_repo="$1"
                shift
                ;;
            -h|--help|help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "diva"
                else
                    echo "Uso: cccp diva [options] [texto_do_problema]"
                    echo "  --dry-run          Executa triagem sem publicar na forja"
                    echo "  --model <model>    Modelo Gemini (padrão: gemini-3.6-flash)"
                    echo "  --repo <slug>      Repositório alvo (padrão: detectado do git remote)"
                    echo "  config <api-key>   Salva GEMINI_API_KEY no .cccprc"
                fi
                return 0
                ;;
            -*)
                echo "Error: Unknown option '$1'" >&2
                return 1
                ;;
            *)
                if [ -z "$_user_text" ]; then
                    _user_text="$1"
                else
                    _user_text="${_user_text} $1"
                fi
                shift
                ;;
        esac
    done

    # 1. Resolve Gemini API Key
    _api_key=$(_diva_resolve_api_key 2>/dev/null || true)
    if [ -z "$_api_key" ]; then
        echo "Error: GEMINI_API_KEY não configurada." >&2
        echo "Configure através de:" >&2
        echo "  export GEMINI_API_KEY=\"sua_chave\"" >&2
        echo "  ou: cccp diva config <sua_chave>" >&2
        return 1
    fi

    # 2. Resolve Forge Info
    _forge="unknown"
    _owner=""
    _repo=""
    _host=""
    if [ -n "$_custom_repo" ]; then
        _owner=$(printf '%s' "$_custom_repo" | cut -d'/' -f1)
        _repo=$(printf '%s' "$_custom_repo" | cut -d'/' -f2)
        _forge="codeberg"
    else
        if _finfo=$(_diva_resolve_forge 2>/dev/null); then
            _forge=$(printf '%s' "$_finfo" | cut -d' ' -f1)
            _owner=$(printf '%s' "$_finfo" | cut -d' ' -f2)
            _repo=$(printf '%s' "$_finfo" | cut -d' ' -f3)
            _host=$(printf '%s' "$_finfo" | cut -d' ' -f4)
        fi
    fi

    # 3. Read input from stdin if not provided as argument
    if [ -z "$_user_text" ]; then
        if [ ! -t 0 ]; then
            _user_text=$(cat)
        else
            echo "★ [ГОСПЛАН] O Divã — Triagem Interativa de Tickets ★"
            if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                echo "🎯 Repositório Alvo: ${_owner}/${_repo} (${_forge})"
            fi
            echo "Descreva seu problema, bug ou funcionalidade (digite seu relato e pressione Enter):"
            printf "> "
            read -r _user_text || _user_text=""
        fi
    fi

    if [ -z "$_user_text" ]; then
        echo "Nenhum relato informado. Encerrando sessão."
        return 0
    fi

    if [ -n "$_owner" ] && [ -n "$_repo" ]; then
        echo "-> [ГОСПЛАН] Consultando Gemini ($_model) para ${_owner}/${_repo} (${_forge})..."
    else
        echo "-> [ГОСПЛАН] Consultando Gemini ($_model) para decomposição de tickets..."
    fi

    _repo_ctx="${_owner}/${_repo} (${_forge})"
    _raw_resp=$(_diva_call_gemini "$_api_key" "$_model" "$_user_text" "$_repo_ctx" 2>/dev/null || true)

    if [ -z "$_raw_resp" ]; then
        echo "Error: Falha na comunicação com a API do Google Gemini." >&2
        return 1
    fi

    _tickets_json=$(_diva_extract_tickets "$_raw_resp" 2>/dev/null || true)
    if [ -z "$_tickets_json" ]; then
        echo "Error: Resposta do modelo não pôde ser interpretada." >&2
        return 1
    fi

    _diva_present_and_dispatch "$_tickets_json" "$_forge" "$_owner" "$_repo" "$_dry_run"
}

cmd_divan() {
    cmd_diva "$@"
}
