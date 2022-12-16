mod app;
mod cache;
mod commands;
mod resp;
mod utils;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::new().unwrap();

    app.run().await
}
