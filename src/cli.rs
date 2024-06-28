use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::crypt::Cli as CryptCli;
use crate::haxe::Cli as HaxeCli;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

pub const MM2_SAVE_KEY: &str = "HXl;kjsaf4982097";

#[derive(Subcommand)]
pub enum Command {
    #[command(flatten)]
    Crypt(CryptCli),

    #[command(flatten)]
    Haxe(HaxeCli),

    /// Manege mm2 save games
    Savetool {
        #[command(subcommand)]
        command: SaveToolCommand,
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
