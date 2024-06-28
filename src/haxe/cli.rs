use clap::Subcommand;

use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Cli {
    /// Haxe serialization and deserialization
    Haxe {
        #[command(subcommand)]
        command: Command,
    },
}

#[derive(Subcommand)]
pub enum Command {
    Serialize {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf,
    },

    Deserialize {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf,
    },
}
