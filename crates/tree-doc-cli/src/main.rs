use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod commands;
mod output;

#[derive(Parser)]
#[command(name = "tree-doc", about = "Tree Document Format validator and viewer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a .tree.json file
    Validate {
        /// Path to the .tree.json file
        file: PathBuf,
    },
    /// View the trunk path of a .tree.json file
    View {
        /// Path to the .tree.json file
        file: PathBuf,
    },
    /// Show summary information about a .tree.json file
    Info {
        /// Path to the .tree.json file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Validate { file } => commands::validate::run(file),
        Commands::View { file } => commands::view::run(file),
        Commands::Info { file } => commands::info::run(file),
    }
}
