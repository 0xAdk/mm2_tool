use clap::{Parser, Subcommand};

use crate::crypt;
use crate::haxe;
use crate::savetool;

#[derive(Parser)]
#[command(
    infer_subcommands = true,
    // so help and haxe subcommands don't conflict as `h`
    disable_help_subcommand = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(flatten)]
    Crypt(crypt::Cli),

    #[command(flatten)]
    Haxe(haxe::Cli),

    #[command(flatten)]
    Savetool(savetool::Cli),
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Command::Crypt(cli) => crypt::run(cli),
        Command::Haxe(cli) => haxe::run(cli),
        Command::Savetool(cli) => savetool::run(cli),
    }
}
