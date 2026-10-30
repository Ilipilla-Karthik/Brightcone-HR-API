//! Thin wrapper around the migration-free CLI runner.
use backend::cli_start;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli_start::run().await
}
