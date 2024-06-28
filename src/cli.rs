use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::crypt;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

pub const MM2_SAVE_KEY: &str = "HXl;kjsaf4982097";

#[derive(Subcommand)]
pub enum Command {
    #[command(flatten)]
    Crypt(crypt::Cli),

    /// Haxe serialization and deserialization
    Haxe {
        #[command(subcommand)]
        command: HaxeCommand,
    },

    /// Manege mm2 save games
    Savetool {
        #[command(subcommand)]
        command: SaveToolCommand,
    },
}

#[derive(Subcommand)]
pub enum HaxeCommand {
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

#[derive(Subcommand)]
pub enum SaveToolCommand {
    Save {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf,
    },
    Load {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf,
    },
}
