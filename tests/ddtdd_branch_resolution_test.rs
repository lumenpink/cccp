use cccp::ddtdd::canonical_branch_name;

#[test]
fn test_canonical_branch_name_with_accents_and_kind() {
    let branch = canonical_branch_name(3, "docs: ajustar conteúdo do site");
    assert_eq!(
        branch, "docs/3-ajustar-conteudo-do-site",
        "Deve transliterar acentos e extrair o prefixo docs"
    );
}

#[test]
fn test_canonical_branch_name_with_scope_and_accents() {
    let branch = canonical_branch_name(42, "feat(auth): implementar autenticação de sessão");
    assert_eq!(
        branch, "feat/42-implementar-autenticacao-de-sessao",
        "Deve remover o escopo do prefixo e transliterar 'autenticação' e 'sessão'"
    );
}

#[test]
fn test_canonical_branch_name_default_kind_fallback() {
    let branch = canonical_branch_name(10, "apenas uma melhoria sem prefixo");
    assert_eq!(
        branch, "feat/10-apenas-uma-melhoria-sem-prefixo",
        "Títulos sem tipo convencional devem adotar 'feat' como padrão"
    );
}

#[test]
fn test_canonical_branch_name_complex_accents_and_symbols() {
    let branch = canonical_branch_name(99, "fix(i18n): correção de ações & atenção!");
    assert_eq!(
        branch, "fix/99-correcao-de-acoes-atencao",
        "Deve transliterar cedilhas e acentos múltiplos sem quebrar traços"
    );
}

#[test]
fn test_parse_single_issue_json() {
    let json_val = serde_json::json!({
        "number": 105,
        "title": "feat: nova funcionalidade",
        "state": "open",
        "user": { "login": "lumen" },
        "labels": [{ "name": "enhancement" }],
        "body": "Descrição de teste",
        "html_url": "https://gambi.2lp.in/lumen/cccp/issues/105",
        "created_at": "2026-10-30T12:00:00Z"
    });

    let issue = cccp::forges::parse_single_issue_json(&json_val)
        .expect("Deve realizar o parse de issue com sucesso");
    assert_eq!(issue.number, 105);
    assert_eq!(issue.title, "feat: nova funcionalidade");
    assert_eq!(issue.author, "lumen");
    assert_eq!(issue.labels, vec!["enhancement"]);
}

#[test]
fn test_canonical_branch_name_mutation_all_vowels_and_empty() {
    let branch = canonical_branch_name(77, "refactor:  ÁÉÍÓÚ âêô ãõ ç   ");
    assert_eq!(
        branch, "refactor/77-aeiou-aeo-ao-c",
        "Deve transliterar todas as vogais acentuadas maiúsculas e minúsculas"
    );

    let empty_branch = canonical_branch_name(88, "feat: !!! ???");
    assert_eq!(
        empty_branch, "feat/88",
        "Quando o slug resultante for vazio, deve formatar apenas como kind/id"
    );
}
