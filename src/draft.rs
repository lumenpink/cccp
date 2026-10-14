//! Divan manual session drafting and repository-isolated persistence.
//!
//! Allows users to intentionally save drafts (via Ctrl+R in the editor or CLI)
//! across repositories without interfering with git working trees.

use crate::ai::DecomposedTicket;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DraftStage {
    Prompt,
    TicketsReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DivanDraft {
    pub repo_path: String,
    pub stage: DraftStage,
    pub prompt: String,
    pub tickets: Vec<DecomposedTicket>,
    pub current_index: usize,
    pub updated_at: String,
}

impl DivanDraft {
    /// Save prompt draft manually
    pub fn save_prompt(repo_path: &Path, prompt: &str) -> Result<(), String> {
        let draft = DivanDraft {
            repo_path: repo_path.to_string_lossy().to_string(),
            stage: DraftStage::Prompt,
            prompt: prompt.to_string(),
            tickets: Vec::new(),
            current_index: 0,
            updated_at: current_timestamp(),
        };
        draft.save_to_disk(repo_path)
    }

    /// Save tickets draft manually during review
    pub fn save_tickets(
        repo_path: &Path,
        prompt: &str,
        tickets: &[DecomposedTicket],
        current_index: usize,
    ) -> Result<(), String> {
        let draft = DivanDraft {
            repo_path: repo_path.to_string_lossy().to_string(),
            stage: DraftStage::TicketsReview,
            prompt: prompt.to_string(),
            tickets: tickets.to_vec(),
            current_index,
            updated_at: current_timestamp(),
        };
        draft.save_to_disk(repo_path)
    }

    /// Load existing draft for the target repository
    pub fn load(repo_path: &Path) -> Result<Option<Self>, String> {
        let path = repo_to_draft_path(repo_path);
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read draft file '{}': {}", path.display(), e))?;

        match serde_json::from_str::<DivanDraft>(&content) {
            Ok(draft) => Ok(Some(draft)),
            Err(e) => {
                eprintln!(
                    "Warning: Corrupted draft file '{}' ({}), ignoring.",
                    path.display(),
                    e
                );
                Ok(None)
            }
        }
    }

    /// Clear and remove draft file for the target repository
    pub fn clear(repo_path: &Path) -> Result<(), String> {
        let path = repo_to_draft_path(repo_path);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                format!("Failed to remove draft file '{}': {}", path.display(), e)
            })?;
        }
        Ok(())
    }

    fn save_to_disk(&self, repo_path: &Path) -> Result<(), String> {
        let path = repo_to_draft_path(repo_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "Failed to create draft directory '{}': {}",
                        parent.display(),
                        e
                    )
                })?;
            }
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize draft: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write draft file '{}': {}", path.display(), e))?;

        Ok(())
    }
}

fn current_timestamp() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

fn get_drafts_dir() -> PathBuf {
    std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".cache")
        })
        .join("cccp")
        .join("drafts")
}

fn repo_to_draft_path(repo_path: &Path) -> PathBuf {
    let canon = repo_path
        .canonicalize()
        .unwrap_or_else(|_| repo_path.to_path_buf());
    let s = canon.to_string_lossy();
    let sanitized: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    get_drafts_dir().join(format!("{}.json", sanitized.trim_matches('_')))
}
