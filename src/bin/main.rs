use loco_rs::cli;
use brightcone_hr_portal::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App>().await
}
