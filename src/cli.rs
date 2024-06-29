use clap::{Parser, Subcommand};

use crate::crypt::Cli as CryptCli;
use crate::haxe::Cli as HaxeCli;
use crate::savetool::Cli as SaveToolCli;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(flatten)]
    Crypt(CryptCli),

    #[command(flatten)]
    Haxe(HaxeCli),

    #[command(flatten)]
    Savetool(SaveToolCli),
}
