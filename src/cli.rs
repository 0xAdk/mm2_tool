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
        // TODO: make this optional + global
        // #[arg(short, long, global = true)]
        #[arg(short, long)]
        output: PathBuf,

        #[command(subcommand)]
        command: CryptCommand,
    },

    /// Haxe serialization and deserialization
    Haxe {
        // TODO: make this optional + global
        // #[arg(short, long, global = true)]
        #[arg(short, long)]
        output: PathBuf,

        #[command(subcommand)]
        command: HaxeCommand,
    },
}

#[derive(Subcommand)]
pub enum CryptCommand {
    Encrypt { file: PathBuf },
    Decrypt { file: PathBuf },
}

#[derive(Subcommand)]
pub enum HaxeCommand {
    Serialize { file: PathBuf },
    Deserialize { file: PathBuf },
}
