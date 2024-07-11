pub mod cli;
mod de;

pub use de::parse as from_str;

pub use cli::{Cli, Command};
pub fn run(Cli::Haxe { command }: Cli) {
    match command {
        Command::Serialize {
            file: _, output: _, ..
        } => {
            todo!("serializing files isn't implemented yet")
        }

        Command::Deserialize {
            file,
            output,
            format,
        } => {
            let data = std::fs::read_to_string(file).unwrap();
            let mut data = data.as_str();

            let obj = from_str(&mut data).unwrap();


            let string_spot: String;
            #[allow(unused_variables)]
            let byte_vec_spot: Vec<u8>;

            let bytes = match FileFormat::guess(&output, format) {
                FileFormat::None => {
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
    None,
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

            (_, cli::FileFormat::None | Auto) => FileFormat::None,
        }
    }
}
