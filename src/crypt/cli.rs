use clap::Subcommand;

use std::path::PathBuf;

const MM2_ASSET_KEY: &str = "aj3fk29dl309f845";

#[derive(Subcommand)]
pub enum Cli {
    /// XXTEA encryption and decryption
    Crypt {
        #[arg(short, long, global = true, default_value = MM2_ASSET_KEY, value_parser = key_parser)]
        key: [u8; 16],

        #[command(subcommand)]
        command: Command,
    },
}

fn key_parser(input: &str) -> Result<[u8; 16], &'static str> {
    input
        .as_bytes()
        .try_into()
        .map_err(|_| "key needs to be 16 characters long")
}

#[derive(Subcommand)]
pub enum Command {
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
