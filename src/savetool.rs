use clap::Subcommand;
use serde::Deserialize;
use serde::Serialize;

use crate::haxe;
use crate::xxtea;

use std::io::Write;
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
            let format = haxe::FileFormat::guess(format, &file);
            if let haxe::FileFormat::Debug = format {
                eprintln!("Error: a format is required when serializing");
                return;
            }

            let data = std::fs::read(file).unwrap();

            let save_file: SaveFile = match format {
                haxe::FileFormat::Debug => unreachable!(),

                #[cfg(feature = "export-json")]
                haxe::FileFormat::Json => serde_json::from_slice(&data).unwrap(),
            };

            let data = SaveFile::save(&save_file);

            let key = MM2_SAVE_KEY.as_bytes().try_into().unwrap();
            let data = xxtea::encrypt_with_padding(data.into_bytes(), key)
                .unwrap_or_else(|err| panic!("{err:?}"));

            std::fs::write(output, data).unwrap();
        }

        Command::Load {
            file,
            output: output_path,
            format,
        } => {
            let data = std::fs::read(file).unwrap();

            let key = MM2_SAVE_KEY.as_bytes().try_into().unwrap();
            let mut data =
                xxtea::decrypt_with_padding(data, key).unwrap_or_else(|err| panic!("{err:?}"));

            let save_file = SaveFile::load(&mut data);

            let mut output = std::fs::File::create(&output_path).unwrap();
            match haxe::FileFormat::guess(format, &output_path) {
                haxe::FileFormat::Debug => {
                    write!(output, "{save_file:#?}").unwrap();
                }

                #[cfg(feature = "export-json")]
                haxe::FileFormat::Json => {
                    output
                        .write_all(&serde_json::to_vec_pretty(&save_file).unwrap())
                        .unwrap();
                }
            };
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveFile<'a> {
    version: std::borrow::Cow<'a, str>,
    values: Vec<haxe::Value<'a>>,
}

impl<'a> SaveFile<'a> {
    fn save(save_file: &Self) -> String {
        format!(
            "[{version}]{hxon}",
            version = save_file.version,
            hxon = haxe::to_string(&save_file.values),
        )
    }

    fn load(data: &'a mut Vec<u8>) -> Self {
        let version_tag_end = data.iter().position(|c| *c == b']').unwrap();
        let _ = data.drain(..=version_tag_end);

        Self {
            version: "4.0.105".into(),
            values: haxe::from_str(std::str::from_utf8(data).unwrap()).unwrap(),
        }
    }
}
