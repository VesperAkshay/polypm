// PPM - Polyglot Package Manager
// Main CLI entry point

use clap::Parser;
use std::process;
use ppm::cli::{Cli, CliDispatcher};
use ppm::utils::error::UserError;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let result = CliDispatcher::execute(cli.command).await;
    
    if let Err(err) = result {
        let user_error = UserError::from_ppm_error(&err);
        user_error.print();
        process::exit(user_error.exit_code);
    }
}
