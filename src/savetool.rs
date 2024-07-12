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

        #[arg(short, long, value_enum, default_value_t = haxe::cli::FileFormat::Auto)]
        format: haxe::cli::FileFormat,

        file: PathBuf,
    },

    Load {
        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long, value_enum, default_value_t = haxe::cli::FileFormat::Auto)]
        format: haxe::cli::FileFormat,

        file: PathBuf,
    },
}

pub fn run(Cli::Savetool { command }: Cli) {
    match command {
        #[cfg_attr(
            not(feature = "export-json"),
            allow(irrefutable_let_patterns, unreachable_code, unused_variables)
        )]
        Command::Save {
            file,
            output,
            format,
        } => {
            let format = haxe::FileFormat::guess(&file, format);
            if let haxe::FileFormat::None = format {
                eprintln!("Error: a format is required when serializing");
                return;
            }

            let data = std::fs::read(file).unwrap();

            let value: Vec<haxe::Value> = match format {
                haxe::FileFormat::None => unreachable!(),

                #[cfg(feature = "export-json")]
                haxe::FileFormat::Json => serde_json::from_slice(&data).unwrap(),
            };

            let data = haxe::to_string(&value);

            // TODO: we should save this tag on load instead of just hardcoding it here
            let tag = "[4.0.104]";
            let data = format!("{tag}{data}").into_bytes();

            let key = MM2_SAVE_KEY.as_bytes().try_into().unwrap();
            let data =
                xxtea::encrypt_with_padding(data, key).unwrap_or_else(|err| panic!("{err:?}"));

            std::fs::write(output, data).unwrap();
        }

        Command::Load {
            file,
            output,
            format,
        } => {
            let data = std::fs::read(file).unwrap();

            let key = MM2_SAVE_KEY.as_bytes().try_into().unwrap();
            let mut data =
                xxtea::decrypt_with_padding(data, key).unwrap_or_else(|err| panic!("{err:?}"));

            let version_tag_end = data.iter().position(|c| *c == b']').unwrap();
            let _ = data.drain(..=version_tag_end);

            let mut data = std::str::from_utf8(&data).unwrap();

            let obj = haxe::from_str(&mut data).unwrap();

            #[cfg_attr(not(feature = "export-json"), allow(unused_variables))]
            let byte_vec_spot: Vec<u8>;
            let string_spot: String;

            let bytes = match haxe::FileFormat::guess(&output, format) {
                haxe::FileFormat::None => {
                    string_spot = format!("{obj:#?}");
                    string_spot.as_bytes()
                }

                #[cfg(feature = "export-json")]
                haxe::FileFormat::Json => {
                    byte_vec_spot = serde_json::to_vec_pretty(&obj).unwrap();
                    byte_vec_spot.as_slice()
                }
            };

            std::fs::write(output, bytes).unwrap();
        }
    }
}
