mod commands;

use commands::{CLI, Commands};
use clap::Parser;


#[tokio::main]
async fn main() {
    let cli = CLI::parse();
    match cli.command {
        Some(Commands::login) => {
            commands::login::login().await;
        },
        _ => {}
    }
}
