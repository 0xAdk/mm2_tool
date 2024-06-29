#![deny(clippy::pedantic)]

mod cli;
mod xxtea;

mod crypt;
mod haxe;
mod savetool;

fn main() {
    let cli = <cli::Cli as clap::Parser>::parse();

    match cli.command {
        cli::Command::Crypt(cli) => crypt::run(cli),
        cli::Command::Haxe(cli) => haxe::run(cli),
        cli::Command::Savetool(cli) => savetool::run(cli),
    }
}
