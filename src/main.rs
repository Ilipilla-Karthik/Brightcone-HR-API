use brightcone_hr_api::app::{App, Migrator};
use loco_rs::cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::main::<App, Migrator>().await?;
    Ok(())
}
