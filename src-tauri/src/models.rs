use kondo::{Config, PlannedOperation, ScanResult, Statistics};
use serde::{Deserialize, Serialize};

// ============================================================================
// Request Models - DTOs for incoming commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryRequest {
    pub path: String,
    pub config_path: Option<String>,
    pub recursive: bool,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub path: String,
    pub config_path: Option<String>,
    pub collision_strategy: CollisionStrategy,
    pub dry_run: bool,
    pub recursive: bool,
    pub backup: bool,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRequest {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveConfigRequest {
    pub config: Config,
    pub path: String,
}

// ============================================================================
// Domain Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollisionStrategy {
    Rename,
    Skip,
    Prompt,
}

impl From<CollisionStrategy> for kondo::CollisionStrategy {
    fn from(strategy: CollisionStrategy) -> Self {
        match strategy {
            CollisionStrategy::Rename => kondo::CollisionStrategy::Rename,
            CollisionStrategy::Skip => kondo::CollisionStrategy::Skip,
            CollisionStrategy::Prompt => kondo::CollisionStrategy::Prompt,
        }
    }
}

// ============================================================================
// Response Models - Re-export from kondo library for consistency
// ============================================================================

pub type ScanResponse = ScanResult;
pub type PreviewResponse = Vec<PlannedOperation>;
pub type ExecuteResponse = Statistics;
pub type ConfigResponse = Config;
