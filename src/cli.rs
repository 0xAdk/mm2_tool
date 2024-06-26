use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// XXTEA encryption and decryption
    Crypt {
        #[command(subcommand)]
        command: CryptCommand,
    },

    /// Haxe serialization and deserialization
    Haxe {
        #[command(subcommand)]
        command: HaxeCommand,
    },
}

#[derive(Subcommand)]
pub enum CryptCommand {
    Encrypt {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf
    },
    Decrypt {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf
    },
}

#[derive(Subcommand)]
pub enum HaxeCommand {
    Serialize {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf
    },
    Deserialize {
        #[arg(short, long)]
        output: PathBuf,

        file: PathBuf
    },
}
