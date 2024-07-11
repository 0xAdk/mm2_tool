use clap::{Subcommand, ValueEnum};

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

        #[arg(short, long, value_enum, default_value_t = FileFormat::Auto)]
        format: FileFormat,

        file: PathBuf,
    },
}

#[derive(Debug, Clone, Default, ValueEnum)]
pub enum FileFormat {
    #[default]
    Auto,

    None,
    #[cfg(feature = "export-json")]
    Json,
}
