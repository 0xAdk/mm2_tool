use crate::xxtea;

mod cli;

pub use cli::{Cli, Command};
pub fn run(Cli::Crypt { key, command }: Cli) {
    match command {
        Command::Encrypt { file, output, .. } => {
            let data = std::fs::read(file).unwrap();
            let data =
                xxtea::encrypt_with_padding(data, &key).unwrap_or_else(|err| panic!("{err:?}"));
            std::fs::write(output, data).unwrap();
        }

        Command::Decrypt { file, output, .. } => {
            let data = std::fs::read(file).unwrap();
            let data =
                xxtea::decrypt_with_padding(data, &key).unwrap_or_else(|err| panic!("{err:?}"));
            std::fs::write(output, data).unwrap();
        }
    }
}
