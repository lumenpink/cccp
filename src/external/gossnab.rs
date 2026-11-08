use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppUpdateResponse {
    pub app: String,
    pub channel: String,
    pub latest_version: String,
    pub download_url: String,
    pub sha256: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QueueItem {
    pub job_id: String,
    pub app: String,
    pub forge: String,
    pub status: String,
    pub job: String,
    pub commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QueueResponse {
    pub active_jobs_count: usize,
    pub queue: Vec<QueueItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CatalogApp {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub latest_version: String,
    pub channel: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CatalogResponse {
    pub apps: Vec<CatalogApp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StagingArtifact {
    pub id: String,
    pub app: String,
    pub version: String,
    pub channel: String,
    pub commit_sha: String,
    pub file_name: String,
    pub sha256: String,
    pub staging_download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeploymentEvent {
    pub artifact_id: String,
    pub app: String,
    pub version: String,
    pub channel: String,
    pub status: String,
    pub staging_purged: bool,
    pub nodes_updated: Vec<String>,
}
