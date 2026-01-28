use loco_rs::cli;
use migration::Migrator;
use rustok_server::app::App;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    Ok(cli::main::<App, Migrator>().await?)
}
