use cccp::cultural::Theme;
use cccp::hook_runner::{
    commit_msg_rejected_guidance, pre_commit_blocked_message, pre_push_blocked_message,
};

#[test]
fn test_pre_commit_blocked_message_instructs_commands_and_warns_bypass() {
    let theme = Theme::with_color(false);
    let msg = pre_commit_blocked_message(&theme);

    // Deve indicar comandos canônicos copiáveis
    assert!(
        msg.contains("cccp commit"),
        "pre-commit deve instruir o comando cccp commit"
    );
    assert!(
        msg.contains("cccp close"),
        "pre-commit deve instruir o comando cccp close"
    );

    // Deve alertar expressamente contra bypass manual e flags indevidas
    assert!(
        msg.contains("Bypass") || msg.contains("burlar") || msg.contains("contorno"),
        "pre-commit deve alertar contra tentativas de contorno/bypass"
    );
    assert!(
        msg.contains("CCCP_MAESTRO_ACTIVE"),
        "pre-commit deve mencionar expressamente que o uso manual de CCCP_MAESTRO_ACTIVE é proibido"
    );
}

#[test]
fn test_pre_push_blocked_message_instructs_push_and_warns_bypass() {
    let theme = Theme::with_color(false);
    let msg = pre_push_blocked_message(&theme);

    // Deve indicar comando canônico
    assert!(
        msg.contains("cccp push"),
        "pre-push deve instruir o comando cccp push"
    );

    // Deve alertar contra bypass manual
    assert!(
        msg.contains("Bypass") || msg.contains("burlar") || msg.contains("contorno"),
        "pre-push deve alertar contra tentativas de contorno/bypass"
    );
    assert!(
        msg.contains("CCCP_MAESTRO_ACTIVE"),
        "pre-push deve mencionar expressamente a proibição de bypass de variáveis de ambiente"
    );
}

#[test]
fn test_commit_msg_rejected_guidance_instructs_cccp_and_warns_no_verify() {
    let theme = Theme::with_color(false);
    let msg = commit_msg_rejected_guidance(&theme);

    // Deve instruir o uso do cccp commit
    assert!(
        msg.contains("cccp commit"),
        "commit-msg deve instruir o uso de cccp commit"
    );

    // Deve alertar contra --no-verify
    assert!(
        msg.contains("--no-verify"),
        "commit-msg deve alertar contra o uso de --no-verify"
    );
}

#[test]
fn test_mutation_colored_themes_preserve_actionable_commands() {
    // Mesmo com ANSI ligado, os comandos brutos devem ser legíveis e pesquisáveis
    let theme_colored = Theme::with_color(true);
    let msg_pre_commit = pre_commit_blocked_message(&theme_colored);
    let msg_pre_push = pre_push_blocked_message(&theme_colored);
    let msg_commit_msg = commit_msg_rejected_guidance(&theme_colored);

    assert!(msg_pre_commit.contains("cccp commit"));
    assert!(msg_pre_commit.contains("cccp close"));
    assert!(msg_pre_push.contains("cccp push"));
    assert!(msg_commit_msg.contains("cccp commit"));
}

#[test]
fn test_mutation_rejects_missing_anti_bypass_safeguards() {
    let theme = Theme::with_color(false);
    let commit_msg = pre_commit_blocked_message(&theme);
    let push_msg = pre_push_blocked_message(&theme);

    // Garantir que proibições específicas constem em ambos
    assert!(
        commit_msg.contains("CCCP_MAESTRO_ACTIVE"),
        "pre-commit deve citar CCCP_MAESTRO_ACTIVE"
    );
    assert!(
        commit_msg.contains("CCCP_BYPASS_LOCKS"),
        "pre-commit deve citar CCCP_BYPASS_LOCKS"
    );
    assert!(
        push_msg.contains("CCCP_MAESTRO_ACTIVE"),
        "pre-push deve citar CCCP_MAESTRO_ACTIVE"
    );
}
