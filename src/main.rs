use clap::{Parser, Subcommand};
use std::env::var;
use std::path::PathBuf;

mod heyho;
mod preflight;

const MANIFEST_FILE_NAME: &str = "manifest.toml";

#[derive(Parser)]
#[command(
    version,
    about = "ayarla manages your dotfies/settings.

It was implemntend so that the author can tell everyone the following:
\"I have an open source github project written in rust!\"

If you want to try it out have a look at the readme in
https://github.com/kdrblkbs/ayarla for a quickstart guide."
)]
struct Cli {
    /// verbose
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates new settings directory and manifest file
    #[command(arg_required_else_help = true, alias = "yeni")]
    New {
        #[arg(short, long)]
        settings_directory: String,
    },
    /// Bootstraps everything in your manifest within your settings directory
    #[command(arg_required_else_help = true, alias = "lan")]
    Bootstrap {
        #[arg(short, long)]
        settings_directory: String,
    },
    // #[command(arg_required_else_help = true)]
    // Add {
    //     #[arg(short, long)]
    //     todo: String,
    // },
}

fn get_home_from_env() -> Result<PathBuf, anyhow::Error> {
    let home = var("HOME")?;
    Ok(PathBuf::from(home))
}

fn main() -> Result<(), anyhow::Error> {
    let home = get_home_from_env()?;
    let cli = Cli::parse();
    match cli.command {
        Commands::New { settings_directory } => {
            let settings_dir_path = preflight::new_checks(&settings_directory)?;
            heyho::before_we_go(settings_dir_path)?;
        }
        Commands::Bootstrap { settings_directory } => {
            let (settings_dir_path, manifest) =
                preflight::bootstrap_checks(&settings_directory)?;
            heyho::lets_go(home.to_path_buf(), settings_dir_path, manifest)?;
        }
    }
    Ok(())
}
