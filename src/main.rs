// PPM - Polyglot Package Manager
// Main CLI entry point (placeholder for TDD)

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "ppm")]
#[command(about = "A unified package manager for JavaScript and Python projects")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new polyglot project
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
        /// Initial version
        #[arg(long, default_value = "1.0.0")]
        version: String,
        /// Include JavaScript dependencies section
        #[arg(long)]
        javascript: bool,
        /// Include Python dependencies section  
        #[arg(long)]
        python: bool,
        /// Include both ecosystems (default)
        #[arg(long)]
        both: bool,
        /// Overwrite existing project.toml
        #[arg(long)]
        force: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Install dependencies
    Install {
        /// Packages to install (if empty, install from project.toml)
        packages: Vec<String>,
        /// Add packages to dependencies (default for new packages)
        #[arg(long)]
        save: bool,
        /// Add packages to dev-dependencies
        #[arg(long)]
        save_dev: bool,
        /// Force JavaScript ecosystem
        #[arg(long)]
        javascript: bool,
        /// Force Python ecosystem
        #[arg(long)]
        python: bool,
        /// Skip symlink creation (install to global store only)
        #[arg(long)]
        no_symlinks: bool,
        /// Use only cached packages (fail if not available)
        #[arg(long)]
        offline: bool,
        /// Use exact versions from lock file (CI mode)
        #[arg(long)]
        frozen: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a new dependency
    Add {
        /// List of packages to add
        packages: Vec<String>,
        /// Add to dev-dependencies instead of dependencies
        #[arg(long)]
        save_dev: bool,
        /// Force JavaScript ecosystem detection
        #[arg(long)]
        javascript: bool,
        /// Force Python ecosystem detection
        #[arg(long)]
        python: bool,
        /// Specific version constraint
        #[arg(long)]
        version: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Run project scripts
    Run {
        /// Script name from project.toml [scripts] section
        script: Option<String>,
        /// Additional arguments to pass to the script
        args: Vec<String>,
        /// Show available scripts instead of running
        #[arg(long)]
        list: bool,
        /// Show environment variables that would be set
        #[arg(long)]
        env: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Manage virtual environments
    Venv {
        /// Venv subcommand
        #[command(subcommand)]
        command: Option<VenvCommands>,
    },
}

#[derive(Subcommand)]
enum VenvCommands {
    /// Create new virtual environment (default)
    Create {
        /// Python version to use
        #[arg(long)]
        python: Option<String>,
        /// Custom path for venv
        #[arg(long)]
        path: Option<String>,
        /// Remove existing venv before creating
        #[arg(long)]
        force: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove existing virtual environment
    Remove {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show virtual environment information
    Info {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Activate venv in current shell (Unix only)
    Shell,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { .. } => {
            // This is intentionally not implemented yet - tests should fail!
            eprintln!("Error: ppm init command not implemented yet");
            process::exit(1);
        }
        Commands::Install { .. } => {
            // This is intentionally not implemented yet - tests should fail!
            eprintln!("Error: ppm install command not implemented yet");
            process::exit(1);
        }
        Commands::Add { .. } => {
            eprintln!("Error: ppm add command not implemented yet");
            process::exit(1);
        }
        Commands::Run { .. } => {
            eprintln!("Error: ppm run command not implemented yet");
            process::exit(1);
        }
        Commands::Venv { .. } => {
            eprintln!("Error: ppm venv command not implemented yet");
            process::exit(1);
        }
    }
}
