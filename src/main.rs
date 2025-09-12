// PPM - Polyglot Package Manager
// Main CLI entry point

use clap::Parser;
use std::process;
use ppm::cli::{Cli, CliDispatcher};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let result = CliDispatcher::execute(cli.command).await;
    
    if let Err(err) = result {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
