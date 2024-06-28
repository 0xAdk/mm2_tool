use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

pub const MM2_ASSET_KEY: &str = "aj3fk29dl309f845";
pub const MM2_SAVE_KEY: &str = "HXl;kjsaf4982097";

#[derive(Subcommand)]
pub enum Command {
    /// XXTEA encryption and decryption
    Crypt {
        #[arg(short, long, global = true, default_value = MM2_ASSET_KEY, value_parser = key_parser)]
        key: [u8; 16],

        #[command(subcommand)]
        command: CryptCommand,
    },

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

fn key_parser(input: &str) -> Result<[u8; 16], &'static str> {
    input
        .as_bytes()
        .try_into()
        .map_err(|_| "key needs to be 16 characters long")
}

#[derive(Subcommand)]
pub enum CryptCommand {
    Encrypt {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf,
    },
    Decrypt {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf,
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
