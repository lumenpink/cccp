//! Divan session checkpointing and crash recovery.
//!
//! Provides automatic, repository-indexed session persistence to preserve user prompts
//! and decomposed tickets across timeouts, network failures, and abrupt terminations.

use crate::ai::DecomposedTicket;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckpointStage {
    PromptEntered,
    TicketsPendingReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DivanCheckpoint {
    pub repo_path: String,
    pub stage: CheckpointStage,
    pub prompt: String,
    pub tickets: Vec<DecomposedTicket>,
    pub current_index: usize,
    pub updated_at: String,
}

impl DivanCheckpoint {
    /// Save prompt checkpoint before invoking AI model
    pub fn save_prompt(repo_path: &Path, prompt: &str) -> Result<(), String> {
        let cp = DivanCheckpoint {
            repo_path: repo_path.to_string_lossy().to_string(),
            stage: CheckpointStage::PromptEntered,
            prompt: prompt.to_string(),
            tickets: Vec::new(),
            current_index: 0,
            updated_at: current_timestamp(),
        };
        cp.save_to_disk(repo_path)
    }

    /// Save tickets checkpoint after AI model returns decomposed tickets
    pub fn save_tickets(
        repo_path: &Path,
        prompt: &str,
        tickets: &[DecomposedTicket],
        current_index: usize,
    ) -> Result<(), String> {
        let cp = DivanCheckpoint {
            repo_path: repo_path.to_string_lossy().to_string(),
            stage: CheckpointStage::TicketsPendingReview,
            prompt: prompt.to_string(),
            tickets: tickets.to_vec(),
            current_index,
            updated_at: current_timestamp(),
        };
        cp.save_to_disk(repo_path)
    }

    /// Load existing checkpoint for the target repository
    pub fn load(repo_path: &Path) -> Result<Option<Self>, String> {
        let path = repo_to_checkpoint_path(repo_path);
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read checkpoint file '{}': {}", path.display(), e))?;

        match serde_json::from_str::<DivanCheckpoint>(&content) {
            Ok(cp) => Ok(Some(cp)),
            Err(e) => {
                // If the checkpoint is corrupted, warn and return None
                eprintln!(
                    "Warning: Corrupted checkpoint file '{}' ({}), ignoring.",
                    path.display(),
                    e
                );
                Ok(None)
            }
        }
    }

    /// Clear and remove checkpoint file for the target repository
    pub fn clear(repo_path: &Path) -> Result<(), String> {
        let path = repo_to_checkpoint_path(repo_path);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                format!(
                    "Failed to remove checkpoint file '{}': {}",
                    path.display(),
                    e
                )
            })?;
        }
        Ok(())
    }

    fn save_to_disk(&self, repo_path: &Path) -> Result<(), String> {
        let path = repo_to_checkpoint_path(repo_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "Failed to create checkpoint directory '{}': {}",
                        parent.display(),
                        e
                    )
                })?;
            }
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize checkpoint: {}", e))?;

        fs::write(&path, json).map_err(|e| {
            format!(
                "Failed to write checkpoint file '{}': {}",
                path.display(),
                e
            )
        })?;

        Ok(())
    }
}

fn current_timestamp() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

fn get_checkpoint_dir() -> PathBuf {
    std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".cache")
        })
        .join("cccp")
        .join("checkpoints")
}

fn repo_to_checkpoint_path(repo_path: &Path) -> PathBuf {
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
    get_checkpoint_dir().join(format!("{}.json", sanitized.trim_matches('_')))
}
