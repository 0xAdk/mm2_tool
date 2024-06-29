use clap::Subcommand;

use crate::haxe;
use crate::xxtea;

use std::path::PathBuf;

pub const MM2_SAVE_KEY: &str = "HXl;kjsaf4982097";

#[derive(Subcommand)]
pub enum Cli {
    /// Manege mm2 save games
    Savetool {
        #[command(subcommand)]
        command: Command,
    },
}

#[derive(Subcommand)]
pub enum Command {
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

pub fn run(Cli::Savetool { command }: Cli) {
    match command {
        Command::Save {
            file: _, output: _, ..
        } => {
            todo!("save tool save isn't implemented yet")
        }

        Command::Load { file, output, .. } => {
            let data = std::fs::read(file).unwrap();

            let key = MM2_SAVE_KEY.as_bytes().try_into().unwrap();
            let data =
                xxtea::decrypt_with_padding(data, key).unwrap_or_else(|err| panic!("{err:?}"));
            let data = data
                .into_iter()
                .map(|c| c as char)
                .skip_while(|c| *c != ']')
                .skip(1)
                .collect::<String>();

            let obj = haxe::from_str(&mut data.as_str()).unwrap();
            std::fs::write(output, std::format!("{obj:#?}")).unwrap();
        }
    }
}
