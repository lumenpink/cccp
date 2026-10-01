use cccp::agents::{
    cmd_agent_check, cmd_agent_update, ensure_agent_gitignored, extract_template_vars,
    resolve_agent_var, unset_agent_var, write_agent_var,
};
use cccp::cultural::Theme;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_extract_template_vars() {
    let text = r#"
# Heading
Hello {{NAME}}, welcome to ${PROJECT_DIR}!
Another {{NAME}} mention.
${EMPTY_VAR} and {{ANOTHER_VAR}}
Plain text without vars.
{{invalid-char}} ${{another_one}}
"#;
    let vars = extract_template_vars(text);
    // invalid-char has a hyphen, so bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_' won't match '-'
    assert_eq!(
        vars,
        vec![
            "ANOTHER_VAR",
            "EMPTY_VAR",
            "NAME",
            "PROJECT_DIR",
            "another_one"
        ]
    );
}

#[test]
fn test_write_and_unset_agent_var() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Write custom variable
    let (target_k, display_k) = write_agent_var("api_key", "secret123", root).unwrap();
    assert_eq!(target_k, "agent_var_api_key");
    assert_eq!(display_k, "api_key");

    let cfg = fs::read_to_string(root.join(".cccprc")).unwrap();
    assert!(cfg.contains("agent_var_api_key = secret123"));

    // 2. Write special key 'url'
    let (target_k2, display_k2) =
        write_agent_var("url", "https://example.com/AGENTS.md", root).unwrap();
    assert_eq!(target_k2, "agent_md_url");
    assert_eq!(display_k2, "agent_md_url");

    let cfg = fs::read_to_string(root.join(".cccprc")).unwrap();
    assert!(cfg.contains("agent_md_url = https://example.com/AGENTS.md"));

    // 3. Write special key 'file'
    let (target_k3, display_k3) = write_agent_var("file", "custom/AGENTS.md", root).unwrap();
    assert_eq!(target_k3, "agent_md_file");
    assert_eq!(display_k3, "agent_md_file");

    // 4. Overwrite custom variable
    write_agent_var("api_key", "secret456", root).unwrap();
    let cfg = fs::read_to_string(root.join(".cccprc")).unwrap();
    assert!(cfg.contains("agent_var_api_key = secret456"));
    assert!(!cfg.contains("secret123"));

    // 5. Unset custom variable
    let unset_k = unset_agent_var("api_key", root).unwrap();
    assert_eq!(unset_k, "api_key");
    let cfg = fs::read_to_string(root.join(".cccprc")).unwrap();
    assert!(!cfg.contains("agent_var_api_key"));

    // 6. Unset url
    unset_agent_var("agent_md_url", root).unwrap();
    let cfg = fs::read_to_string(root.join(".cccprc")).unwrap();
    assert!(!cfg.contains("agent_md_url"));
}

#[test]
fn test_resolve_agent_var_precedence() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. None initially
    assert_eq!(resolve_agent_var("my_test_var", Some(root)), None);

    // 2. Local .cccprc
    write_agent_var("my_test_var", "local_val", root).unwrap();
    assert_eq!(
        resolve_agent_var("my_test_var", Some(root)),
        Some("local_val".to_string())
    );
    // Check normalized resolution
    assert_eq!(
        resolve_agent_var("MY_TEST_VAR", Some(root)),
        Some("local_val".to_string())
    );
    assert_eq!(
        resolve_agent_var("agent_my_test_var", Some(root)),
        Some("local_val".to_string())
    );

    // 3. Environment overrides local .cccprc
    std::env::set_var("AGENT_VAR_MY_TEST_VAR", "env_val");
    assert_eq!(
        resolve_agent_var("my_test_var", Some(root)),
        Some("env_val".to_string())
    );

    std::env::remove_var("AGENT_VAR_MY_TEST_VAR");
}

#[test]
fn test_ensure_agent_gitignored() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let gitignore = root.join(".gitignore");

    // File outside .antigravity does not modify .gitignore
    ensure_agent_gitignored(root, &root.join("docs/AGENTS.md"));
    assert!(!gitignore.exists());

    // File inside .antigravity creates .gitignore
    let target = root.join(".antigravity/AGENTS.md");
    ensure_agent_gitignored(root, &target);
    assert!(gitignore.is_file());
    let content = fs::read_to_string(&gitignore).unwrap();
    assert_eq!(content.trim(), ".antigravity/");

    // Calling again does not duplicate
    ensure_agent_gitignored(root, &target);
    let content2 = fs::read_to_string(&gitignore).unwrap();
    assert_eq!(
        content2
            .lines()
            .filter(|l| l.trim() == ".antigravity/")
            .count(),
        1
    );
}

#[test]
fn test_cmd_agent_check_and_update() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let root = dir.path();

    let template_file = root.join("my_template.md");
    fs::write(
        &template_file,
        "Author: {{AUTHOR}}\nProject: ${PROJECT}\nExtra: {{OPTIONAL}}\n",
    )
    .unwrap();

    let dest_file = root.join(".antigravity/AGENTS.md");
    let theme = Theme::init(false);

    // Check when variables missing -> returns false
    let check_res = cmd_agent_check(
        Some(template_file.to_str().unwrap()),
        Some(dest_file.to_str().unwrap()),
        Some(root),
        &theme,
    )
    .unwrap();
    assert!(!check_res);

    // Set some variables
    write_agent_var("author", "Vladimir", root).unwrap();
    write_agent_var("project", "CCCP", root).unwrap();

    // Still missing OPTIONAL
    assert!(!cmd_agent_check(
        Some(template_file.to_str().unwrap()),
        Some(dest_file.to_str().unwrap()),
        Some(root),
        &theme,
    )
    .unwrap());

    // Run update with partial variables
    cmd_agent_update(
        Some(template_file.to_str().unwrap()),
        Some(dest_file.to_str().unwrap()),
        Some(root),
        &theme,
    )
    .unwrap();

    assert!(dest_file.is_file());
    let content = fs::read_to_string(&dest_file).unwrap();
    assert!(content.contains("Author: Vladimir"));
    assert!(content.contains("Project: CCCP"));
    assert!(content.contains("Extra: {{OPTIONAL}}")); // preserved placeholder

    // Gitignore updated
    let gitignore = root.join(".gitignore");
    assert!(gitignore.is_file());
    assert!(fs::read_to_string(gitignore)
        .unwrap()
        .contains(".antigravity/"));

    // Provide missing variable and re-check
    write_agent_var("optional", "None", root).unwrap();
    let check_full = cmd_agent_check(
        Some(template_file.to_str().unwrap()),
        Some(dest_file.to_str().unwrap()),
        Some(root),
        &theme,
    )
    .unwrap();
    assert!(check_full);

    // Re-update
    cmd_agent_update(
        Some(template_file.to_str().unwrap()),
        Some(dest_file.to_str().unwrap()),
        Some(root),
        &theme,
    )
    .unwrap();

    let content_final = fs::read_to_string(&dest_file).unwrap();
    assert!(content_final.contains("Extra: None"));
}
