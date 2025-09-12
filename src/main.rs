// PPM - Polyglot Package Manager
// Main CLI entry point

use clap::{Parser, Subcommand};
use std::process;
use ppm::cli::init::InitCommand;
use ppm::cli::install::InstallCommand;
use ppm::cli::add::AddCommand;

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
        /// Project name (default: current directory name)
        #[arg(long)]
        name: Option<String>,
        
        /// Initial version (default: "1.0.0")
        #[arg(long)]
        version: Option<String>,
        
        /// Include JavaScript dependencies section only
        #[arg(long, conflicts_with = "python")]
        javascript: bool,
        
        /// Include Python dependencies section only
        #[arg(long, conflicts_with = "javascript")]
        python: bool,
        
        /// Overwrite existing project.toml
        #[arg(long)]
        force: bool,
        
        /// Output JSON instead of human-readable text
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Init { name, version, javascript, python, force, json } => {
            let init_cmd = InitCommand {
                name,
                version,
                javascript,
                python,
                force,
                json,
            };
            init_cmd.run().await
        }
        Commands::Install { 
            packages, 
            save, 
            save_dev, 
            javascript, 
            python, 
            no_symlinks, 
            offline, 
            frozen, 
            json 
        } => {
            let install_cmd = InstallCommand {
                packages,
                save,
                save_dev,
                javascript,
                python,
                no_symlinks,
                offline,
                frozen,
                json,
            };
            install_cmd.run().await
        }
        Commands::Add { 
            packages, 
            save_dev, 
            javascript, 
            python, 
            version, 
            json 
        } => {
            let add_cmd = AddCommand {
                packages,
                save_dev,
                javascript,
                python,
                version,
                json,
            };
            add_cmd.execute().await
        }
        Commands::Run { .. } => {
            eprintln!("Error: ppm run command not implemented yet");
            process::exit(1);
        }
        Commands::Venv { .. } => {
            eprintln!("Error: ppm venv command not implemented yet");
            process::exit(1);
        }
    };
    
    if let Err(err) = result {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
