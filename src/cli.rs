use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

const MM2_ASSET_KEY: &str = "aj3fk29dl309f845";

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
