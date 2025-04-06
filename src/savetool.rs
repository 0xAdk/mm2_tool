use clap::Subcommand;
use serde::{Deserialize, Serialize};
use winnow::{
    combinator::{alt, delimited, repeat},
    token::one_of,
    PResult, Parser,
};

use crate::haxe;
use crate::xxtea;

use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;

pub const MM2_SAVE_KEY: &[u8] = b"HXl;kjsaf4982097";

#[derive(Subcommand)]
pub enum Cli {
    /// Manage mm2 save files
    Savetool {
        #[command(subcommand)]
        command: Command,
    },
}

#[derive(Subcommand)]
pub enum Command {
    Encode {
        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long, value_enum, default_value_t = haxe::cli::FileFormat::Auto)]
        format: haxe::cli::FileFormat,

        file: PathBuf,
    },

    Decode {
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
        Command::Encode {
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

            let data = SaveFile::encode(&save_file);

            let key = MM2_SAVE_KEY.try_into().unwrap();
            let data = xxtea::encrypt_with_padding(data.into_bytes(), key)
                .unwrap_or_else(|err| panic!("{err:?}"));

            std::fs::write(output, data).unwrap();
        }

        Command::Decode {
            file,
            output: output_path,
            format,
        } => {
            let data = std::fs::read(file).unwrap();

            let key = MM2_SAVE_KEY.try_into().unwrap();
            let data =
                xxtea::decrypt_with_padding(data, key).unwrap_or_else(|err| panic!("{err:?}"));

            let save_file = SaveFile::decode(&data);

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
    version: Cow<'a, str>,
    values: Vec<haxe::Value<'a>>,
}

impl<'a> SaveFile<'a> {
    fn encode(save_file: &Self) -> String {
        format!(
            "[{version}]{hxon}",
            version = save_file.version,
            hxon = haxe::to_string(&save_file.values),
        )
    }

    fn decode(data: &'a [u8]) -> Self {
        let input = &mut std::str::from_utf8(data).unwrap();

        Self {
            version: Self::parse_version_tag(input).unwrap().into(),
            values: haxe::from_str(input).unwrap(),
        }
    }

    fn parse_version_tag(input: &mut &str) -> PResult<String> {
        let version = repeat(1.., alt((one_of('0'..='9'), '.')));
        delimited('[', version, ']').parse_next(input)
    }
}
