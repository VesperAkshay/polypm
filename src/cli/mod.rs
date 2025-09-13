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
#[command(long_about = r#"PPM (Polyglot Package Manager) enables seamless dependency management 
across JavaScript and Python ecosystems in a single project.

Features:
  • Unified project.toml configuration
  • Automatic virtual environment management
  • Cross-platform symlink support
  • Script execution with proper environment setup
  • Dependency locking and reproducible builds

Examples:
  ppm init --name my-project    Initialize a new polyglot project
  ppm add express requests      Add packages from different ecosystems
  ppm install                   Install all dependencies
  ppm run build                 Execute project scripts
  ppm venv create              Create Python virtual environment

For detailed documentation, visit: https://github.com/VesperAkshay/polypm"#)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// All available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new polyglot project
    #[command(long_about = r#"Initialize a new polyglot project with unified configuration.

Creates a project.toml file with sensible defaults for JavaScript and Python 
dependency management. You can customize the project name, version, and 
supported ecosystems.

Examples:
  ppm init                              Create project with auto-detected name
  ppm init --name my-app --version 0.1.0  Custom name and version  
  ppm init --javascript                 JavaScript-only project
  ppm init --python                     Python-only project
  ppm init --force                      Overwrite existing project.toml"#)]
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
    
    /// Install dependencies from project.toml or add new packages
    #[command(long_about = r#"Install project dependencies from project.toml or add new packages.

When run without arguments, installs all dependencies listed in project.toml:
  • JavaScript packages to node_modules/ with symlink optimization
  • Python packages to virtual environment (.venv/)
  • Creates lock file for reproducible builds

When packages are specified, adds them to project.toml and installs.

Examples:
  ppm install                           Install all dependencies from project.toml
  ppm install --dev                     Include dev dependencies
  ppm install --python                  Python packages only
  ppm install express@4.18.0           Add and install specific package
  ppm install --offline                Use cached packages only"#)]
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
    
    /// Add new dependencies to the project
    #[command(long_about = r#"Add new dependencies to project.toml and install them.

Automatically detects the package ecosystem (JavaScript vs Python) and adds 
the dependency to the appropriate section. You can override detection with 
--javascript or --python flags.

The package is added to project.toml and immediately installed. Use --save-dev 
to add to development dependencies instead.

Examples:
  ppm add express                       Auto-detect ecosystem and add
  ppm add express@^4.18.0              Add with specific version constraint
  ppm add @types/node --javascript     Force JavaScript ecosystem  
  ppm add pytest black --python --save-dev  Python dev dependencies
  ppm add package --version "~1.0.0"   Add with version flag"#)]
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
    
    /// Execute project scripts with proper environment setup
    #[command(long_about = r#"Execute scripts defined in project.toml [scripts] section.

Scripts run with the proper environment setup for both JavaScript and Python:
  • NODE_PATH set to node_modules directory
  • Python virtual environment activated
  • PPM environment variables available

Use --list to see all available scripts, or --env to inspect the environment 
that would be set for a specific script.

Examples:
  ppm run build                         Execute the 'build' script
  ppm run test -- --verbose            Pass arguments to the script
  ppm run --list                       Show all available scripts
  ppm run start --env                  Show environment for 'start' script"#)]
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
    
    /// Manage Python virtual environments
    #[command(long_about = r#"Create and manage Python virtual environments for the project.

PPM automatically creates and manages Python virtual environments to isolate 
Python dependencies. The virtual environment is created in .venv/ by default
and activated automatically when running Python scripts.

Subcommands:
  create    Create a new virtual environment (default action)
  remove    Remove the existing virtual environment  
  info      Show information about the current virtual environment
  shell     Print activation command for current shell (Unix only)

Examples:
  ppm venv create                       Create virtual environment with defaults
  ppm venv create --python python3.11  Use specific Python version
  ppm venv info                         Show current virtual environment details
  ppm venv remove                       Remove virtual environment"#)]
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
