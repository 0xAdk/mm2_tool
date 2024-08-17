pub mod cli;
mod de;
mod ser;
mod value;

#[cfg(test)]
mod tests;

pub use de::from_str;
pub use ser::to_string;
pub use value::Value;

pub use cli::{Cli, Command};
pub fn run(Cli::Haxe { command }: Cli) {
    match command {
        #[cfg_attr(
            not(feature = "export-json"),
            allow(irrefutable_let_patterns, unreachable_code, unused_variables)
        )]
        Command::Serialize {
            file,
            output,
            format,
        } => {
            let format = FileFormat::guess(&file, format);
            if let FileFormat::Debug = format {
                eprintln!("Error: a format is required when serializing");
                return;
            }

            let data = std::fs::read(file).unwrap();

            let value: Vec<Value> = match format {
                FileFormat::Debug => unreachable!(),

                #[cfg(feature = "export-json")]
                FileFormat::Json => serde_json::from_slice(&data).unwrap(),
            };

            std::fs::write(output, to_string(&value)).unwrap();
        }

        Command::Deserialize {
            file,
            output,
            format,
        } => {
            let data = std::fs::read_to_string(file).unwrap();
            let obj = from_str(&data).unwrap();

            #[cfg_attr(not(feature = "export-json"), allow(unused_variables))]
            let byte_vec_spot: Vec<u8>;
            let string_spot: String;

            let bytes = match FileFormat::guess(&output, format) {
                FileFormat::Debug => {
                    string_spot = format!("{obj:#?}");
                    string_spot.as_bytes()
                }

                #[cfg(feature = "export-json")]
                FileFormat::Json => {
                    byte_vec_spot = serde_json::to_vec_pretty(&obj).unwrap();
                    byte_vec_spot.as_slice()
                }
            };

            std::fs::write(output, bytes).unwrap();
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileFormat {
    Debug,
    #[cfg(feature = "export-json")]
    Json,
}

impl FileFormat {
    pub fn guess(output: &std::path::Path, format: cli::FileFormat) -> FileFormat {
        use cli::FileFormat::Auto;

        let extension = output.extension().and_then(|e| e.to_str());
        match (extension, format) {
            #[cfg(feature = "export-json")]
            (_, cli::FileFormat::Json) | (Some("json"), Auto) => FileFormat::Json,

            (_, cli::FileFormat::Debug | Auto) => FileFormat::Debug,
        }
    }
}
