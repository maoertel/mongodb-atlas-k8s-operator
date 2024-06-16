use clap::Parser;

/// Application to scale mongodb atlas clusters
#[derive(Parser)]
pub struct Cli {
    /// The public key of your Atlas API key.
    #[clap(long, requires = "private_key", env = "ATLAS_PUBLIC_KEY")]
    pub(crate) public_key: String,

    /// The private key of your Atlas API key.
    #[clap(long, requires = "public_key", env = "ATLAS_PRIVATE_KEY")]
    pub(crate) private_key: String,
}
