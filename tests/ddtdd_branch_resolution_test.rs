use cccp::ddtdd::canonical_branch_name;

#[test]
fn test_canonical_branch_name_with_accents_and_kind() {
    let branch = canonical_branch_name(3, "docs: ajustar conteúdo do site");
    assert_eq!(
        branch,
        "docs/3-ajustar-conteudo-do-site",
        "Deve transliterar acentos e extrair o prefixo docs"
    );
}

#[test]
fn test_canonical_branch_name_with_scope_and_accents() {
    let branch = canonical_branch_name(42, "feat(auth): implementar autenticação de sessão");
    assert_eq!(
        branch,
        "feat/42-implementar-autenticacao-de-sessao",
        "Deve remover o escopo do prefixo e transliterar 'autenticação' e 'sessão'"
    );
}

#[test]
fn test_canonical_branch_name_default_kind_fallback() {
    let branch = canonical_branch_name(10, "apenas uma melhoria sem prefixo");
    assert_eq!(
        branch,
        "feat/10-apenas-uma-melhoria-sem-prefixo",
        "Títulos sem tipo convencional devem adotar 'feat' como padrão"
    );
}
