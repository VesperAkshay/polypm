// CLI module for command-line interface

pub mod add;
pub mod init;
pub mod install;
pub mod run;
pub mod venv;

use clap::{Parser, Subcommand};
use crate::utils::error::Result;

use self::add::AddCommand;
use self::init::InitCommand;
use self::install::InstallCommand;
use self::run::RunCommand;
use self::venv::{VenvHandler, VenvCommands};

/// Main CLI structure
#[derive(Parser)]
#[command(name = "ppm")]
#[command(about = "A unified package manager for JavaScript and Python projects")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// All available CLI commands
#[derive(Subcommand)]
pub enum Commands {
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

/// CLI command dispatcher
pub struct CliDispatcher;

impl CliDispatcher {
    /// Execute a CLI command
    pub async fn execute(command: Commands) -> Result<()> {
        match command {
            Commands::Init { name, version, javascript, python, force, json } => {
                let cmd = InitCommand {
                    name,
                    version,
                    javascript,
                    python,
                    force,
                    json,
                };
                cmd.run().await
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
                let cmd = InstallCommand {
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
                cmd.run().await
            }
            
            Commands::Add { 
                packages, 
                save_dev, 
                javascript, 
                python, 
                version, 
                json 
            } => {
                let cmd = AddCommand {
                    packages,
                    save_dev,
                    javascript,
                    python,
                    version,
                    json,
                };
                cmd.execute().await
            }
            
            Commands::Run { script, args, list, env, json } => {
                let cmd = RunCommand {
                    script,
                    list,
                    env,
                    json,
                    args,
                };
                cmd.execute().await
            }
            
            Commands::Venv { command } => {
                let handler = VenvHandler { command };
                handler.execute().await
            }
        }
    }
}
