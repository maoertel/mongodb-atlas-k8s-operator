use std::sync::Arc;

use clap::Parser;

/// MongoDB Atlas Kubernetes Operator
#[derive(Parser)]
pub struct Cli {
    /// OAuth access token for Atlas API authentication
    #[clap(long, env = "ATLAS_ACCESS_TOKEN")]
    pub access_token: Arc<str>,

    /// Path to configuration file
    #[clap(long, short, env = "CONFIG_PATH")]
    pub config_path: String,

    /// Namespaces to watch (can be specified multiple times)
    #[clap(long, short)]
    pub namespaces: Option<Vec<String>>,
}
