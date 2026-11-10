//! Integration tests for declarative GOST rules executed via Lunokhod (agnostic engine)

use cccp::audit::{run_gost_audit, CheckStatus};
use cccp::cultural::Theme;
use std::fs;
use tempfile::tempdir;

fn scaffold_minimal_valid_gost2_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    fs::write(p.join("GOSTVER"), "GOST_VERSION=\"2.0.0\"\n").unwrap();
    fs::write(p.join("LICENSE"), "GPL-3.0").unwrap();
    fs::write(p.join("VERSION"), "0.1.0\n").unwrap();
    fs::write(p.join("CHANGELOG.md"), "# Changelog\n").unwrap();
    fs::write(
        p.join("README.md"),
        "# Projeto\n\nDescrição executiva em poucas linhas.\n",
    )
    .unwrap();

    fs::write(p.join(".gitignore"), "target/\n").unwrap();
    fs::write(p.join(".editorconfig"), "root = true\n").unwrap();

    fs::create_dir_all(p.join("site")).unwrap();
    fs::write(p.join("site/index.html"), "<html><body>Docs</body></html>").unwrap();
    fs::write(p.join("site/_headers"), "/*\n  X-Frame-Options: DENY\n").unwrap();

    fs::create_dir_all(p.join(".forgejo/workflows")).unwrap();
    fs::write(p.join(".forgejo/workflows/ci.yml"), "name: CI\n").unwrap();

    let rules = r#"
[rules.GOST-20-SITE-TYPE-AND-SSOT-DOCS]
name = "Topologia de Documentação e README"
severity = "error"
scope = "project"
description = "Verifica se README não excede 50 linhas e se site/index.html existe"
script = '''
local lines = gost.count_lines("README.md")
if lines == 0 and not gost.file_exists("README.md") then
    return false, "README.md ausente"
end
if lines > 50 then
    return false, "README excede 50 linhas (" .. lines .. ")"
end
if not gost.file_exists("site/index.html") then
    return false, "site/index.html ausente"
end
return true
'''

[rules.GOST-24-ZERO-BINARIES-IN-GIT]
name = "Veto a Binários no Repositório"
severity = "error"
scope = "project"
description = "Verifica ausência de pacotes ou binários compilados"
script = '''
local forbidden = {".exe", ".so", ".dll", ".dylib", ".a", ".tar.gz", ".tgz", ".zip", ".whl", ".deb", ".rpm", ".apk", ".appimage"}
local files = gost.list_files(".")
for _, file in ipairs(files) do
    local lower = string.lower(file)
    for _, ext in ipairs(forbidden) do
        if string.sub(lower, -string.len(ext)) == ext then
            return false, "Binário compilado detectado: " .. file
        end
    end
end
return true
'''

[rules.GOST-26-METADATA-MANIFEST-SSOT]
name = "Manifesto de Metadados e Anti-Jargão"
severity = "error"
scope = "project"
description = "Valida metadados e veta jargões corporativos"
script = '''
if not gost.file_exists(".gost/metadata.json") then
    return true
end
local content = gost.read_file(".gost/metadata.json")
if not content then return false, "Falha ao ler metadata.json" end
local lower = string.lower(content)
local jargons = {"soberano", "soberana", "hexagonal", "ssot", "rust puro", "em rust", "em php", "wasm"}
for _, j in ipairs(jargons) do
    if string.find(lower, j, 1, true) then
        return false, "Jargão técnico proibido detectado: " .. j
    end
end
local json = gost.parse_json(content)
if not json then
    return false, "JSON malformado em metadata.json"
end
local required = {"slug", "title", "site_type", "tagline", "summary", "abstract", "tags", "icon", "theme"}
for _, req in ipairs(required) do
    if json[req] == nil then
        return false, "Campo obrigatório ausente: " .. req
    end
end
return true
'''

[rules.GOST-27-DDD-HEXAGONAL-PURITY]
name = "Pureza Arquitetural do Domínio"
severity = "error"
scope = "project"
description = "Veta I/O impuro na camada de domínio"
script = '''
if not gost.is_dir("src/domain") then
    return true
end
local files = gost.list_files("src/domain")
local forbidden = {"std::fs", "std::net", "std::process", "tokio::fs", "tokio::net", "reqwest", "rusqlite", "sqlx", "clap"}
for _, file in ipairs(files) do
    if string.sub(file, -3) == ".rs" then
        local code = gost.read_file(file) or ""
        for _, pat in ipairs(forbidden) do
            if string.find(code, pat, 1, true) then
                return false, "I/O proibido no domínio: " .. pat
            end
        end
    end
end
return true
'''
"#;

    fs::write(p.join("rules.toml"), rules).unwrap();
    dir
}

#[test]
fn test_audit_gost20_readme_line_ceiling() {
    let repo = scaffold_minimal_valid_gost2_repo();
    let theme = Theme::with_color(false);

    // 1. README <= 50 linhas -> PASS
    let report = run_gost_audit(repo.path(), true, &theme);
    let check = report
        .checks
        .iter()
        .find(|c| c.code == "GOST-20-SITE-TYPE-AND-SSOT-DOCS");
    assert!(check.is_some(), "GOST-20 check must exist in report");
    assert_eq!(check.unwrap().status, CheckStatus::Pass);

    // 2. README > 50 linhas -> FAIL
    let long_readme: String = (0..55).map(|i| format!("Linha {i}\n")).collect();
    fs::write(repo.path().join("README.md"), long_readme).unwrap();

    let report_fail = run_gost_audit(repo.path(), true, &theme);
    let check_fail = report_fail
        .checks
        .iter()
        .find(|c| c.code == "GOST-20-SITE-TYPE-AND-SSOT-DOCS")
        .unwrap();
    assert_eq!(check_fail.status, CheckStatus::Fail);

    // 3. Mutation test: Exatamente 50 linhas -> PASS
    let exact_50: String = (0..50).map(|i| format!("Linha {i}\n")).collect();
    fs::write(repo.path().join("README.md"), exact_50).unwrap();
    let report_50 = run_gost_audit(repo.path(), true, &theme);
    let check_50 = report_50
        .checks
        .iter()
        .find(|c| c.code == "GOST-20-SITE-TYPE-AND-SSOT-DOCS")
        .unwrap();
    assert_eq!(check_50.status, CheckStatus::Pass);

    // 4. Mutation test: Exatamente 51 linhas -> FAIL
    let exact_51: String = (0..51).map(|i| format!("Linha {i}\n")).collect();
    fs::write(repo.path().join("README.md"), exact_51).unwrap();
    let report_51 = run_gost_audit(repo.path(), true, &theme);
    let check_51 = report_51
        .checks
        .iter()
        .find(|c| c.code == "GOST-20-SITE-TYPE-AND-SSOT-DOCS")
        .unwrap();
    assert_eq!(check_51.status, CheckStatus::Fail);
}

#[test]
fn test_audit_gost24_zero_binaries() {
    let repo = scaffold_minimal_valid_gost2_repo();
    let theme = Theme::with_color(false);

    // 1. Sem binários -> PASS
    let report = run_gost_audit(repo.path(), true, &theme);
    let check = report
        .checks
        .iter()
        .find(|c| c.code == "GOST-24-ZERO-BINARIES-IN-GIT");
    assert!(check.is_some(), "GOST-24 check must exist in report");
    assert_eq!(check.unwrap().status, CheckStatus::Pass);

    // 2. Com arquivo binário proibido (.so) -> FAIL
    fs::write(repo.path().join("libevil.so"), b"binary").unwrap();
    let report_fail = run_gost_audit(repo.path(), true, &theme);
    let check_fail = report_fail
        .checks
        .iter()
        .find(|c| c.code == "GOST-24-ZERO-BINARIES-IN-GIT")
        .unwrap();
    assert_eq!(check_fail.status, CheckStatus::Fail);

    // 3. Mutation test: extensão maiúscula (.EXE) e em subpasta aninhada -> FAIL
    fs::remove_file(repo.path().join("libevil.so")).unwrap();
    fs::create_dir_all(repo.path().join("nested/deep/bin")).unwrap();
    fs::write(repo.path().join("nested/deep/bin/tool.EXE"), b"win32").unwrap();
    let report_upper = run_gost_audit(repo.path(), true, &theme);
    let check_upper = report_upper
        .checks
        .iter()
        .find(|c| c.code == "GOST-24-ZERO-BINARIES-IN-GIT")
        .unwrap();
    assert_eq!(check_upper.status, CheckStatus::Fail);
}

#[test]
fn test_audit_gost26_metadata_and_anti_jargon() {
    let repo = scaffold_minimal_valid_gost2_repo();
    let theme = Theme::with_color(false);

    fs::create_dir_all(repo.path().join(".gost")).unwrap();

    // 1. Manifesto com jargão proibido -> FAIL
    let bad_metadata = r##"{
      "slug": "meu-app",
      "title": "Meu App",
      "site_type": "doc",
      "tagline": "App em rust puro com arquitetura hexagonal soberana",
      "summary": "Um resumo com jargão proibido.",
      "abstract": "Texto de abstract.",
      "tags": ["app", "cli", "util"],
      "icon": "icon",
      "theme": { "bg_color": "#000", "accent_color": "#fff" }
    }"##;
    fs::write(repo.path().join(".gost/metadata.json"), bad_metadata).unwrap();

    let report_fail = run_gost_audit(repo.path(), true, &theme);
    let check_fail = report_fail
        .checks
        .iter()
        .find(|c| c.code == "GOST-26-METADATA-MANIFEST-SSOT");
    assert!(check_fail.is_some(), "GOST-26 check must exist");
    assert_eq!(check_fail.unwrap().status, CheckStatus::Fail);

    // 2. Manifesto limpo -> PASS
    let good_metadata = r##"{
      "slug": "meu-app",
      "title": "Meu App",
      "site_type": "doc",
      "tagline": "Gerenciador eficiente de notas e tarefas",
      "summary": "Um resumo conciso e objetivo sem jargão.",
      "abstract": "Texto de apresentação executiva e clara.",
      "tags": ["app", "cli", "util"],
      "icon": "icon",
      "theme": { "bg_color": "#000", "accent_color": "#fff" }
    }"##;
    fs::write(repo.path().join(".gost/metadata.json"), good_metadata).unwrap();

    let report_ok = run_gost_audit(repo.path(), true, &theme);
    let check_ok = report_ok
        .checks
        .iter()
        .find(|c| c.code == "GOST-26-METADATA-MANIFEST-SSOT")
        .unwrap();
    assert_eq!(check_ok.status, CheckStatus::Pass);

    // 3. Mutation test: Campo obrigatório faltando (ex: "theme") -> FAIL
    let missing_field_metadata = r##"{
      "slug": "meu-app",
      "title": "Meu App",
      "site_type": "doc",
      "tagline": "Gerenciador eficiente de notas e tarefas",
      "summary": "Um resumo conciso e objetivo sem jargão.",
      "abstract": "Texto de apresentação executiva e clara.",
      "tags": ["app", "cli", "util"],
      "icon": "icon"
    }"##;
    fs::write(
        repo.path().join(".gost/metadata.json"),
        missing_field_metadata,
    )
    .unwrap();
    let report_missing = run_gost_audit(repo.path(), true, &theme);
    let check_missing = report_missing
        .checks
        .iter()
        .find(|c| c.code == "GOST-26-METADATA-MANIFEST-SSOT")
        .unwrap();
    assert_eq!(check_missing.status, CheckStatus::Fail);

    // 4. Mutation test: JSON sintaticamente malformado -> FAIL
    fs::write(
        repo.path().join(".gost/metadata.json"),
        "{ invalid json ,,, }",
    )
    .unwrap();
    let report_syntax = run_gost_audit(repo.path(), true, &theme);
    let check_syntax = report_syntax
        .checks
        .iter()
        .find(|c| c.code == "GOST-26-METADATA-MANIFEST-SSOT")
        .unwrap();
    assert_eq!(check_syntax.status, CheckStatus::Fail);
}

#[test]
fn test_audit_gost27_domain_purity() {
    let repo = scaffold_minimal_valid_gost2_repo();
    let theme = Theme::with_color(false);

    fs::create_dir_all(repo.path().join("src/domain")).unwrap();

    // 1. Arquivo de domínio com I/O proibido (std::fs) -> FAIL
    fs::write(
        repo.path().join("src/domain/entity.rs"),
        "use std::fs::File;\npub struct Entity;\n",
    )
    .unwrap();

    let report_fail = run_gost_audit(repo.path(), true, &theme);
    let check_fail = report_fail
        .checks
        .iter()
        .find(|c| c.code == "GOST-27-DDD-HEXAGONAL-PURITY");
    assert!(check_fail.is_some(), "GOST-27 check must exist");
    assert_eq!(check_fail.unwrap().status, CheckStatus::Fail);

    // 2. Arquivo de domínio puro -> PASS
    fs::write(
        repo.path().join("src/domain/entity.rs"),
        "pub struct Entity { pub id: u64 }\n",
    )
    .unwrap();

    let report_ok = run_gost_audit(repo.path(), true, &theme);
    let check_ok = report_ok
        .checks
        .iter()
        .find(|c| c.code == "GOST-27-DDD-HEXAGONAL-PURITY")
        .unwrap();
    assert_eq!(check_ok.status, CheckStatus::Pass);
}
