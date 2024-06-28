#![deny(clippy::pedantic)]

mod cli;
mod crypt;
mod haxe;
mod xxtea;

fn main() {
    let cli = <cli::Cli as clap::Parser>::parse();

    match cli.command {
        cli::Command::Crypt(cli) => crypt::run(cli),

        cli::Command::Haxe { command, .. } => match command {
            cli::HaxeCommand::Serialize {
                file: _, output: _, ..
            } => {
                todo!("serializing files isn't implemented yet")
            }

            cli::HaxeCommand::Deserialize { file, output, .. } => {
                let data = std::fs::read_to_string(file).unwrap();
                let data = &mut data.as_str();

                let obj = haxe::from_str(data).unwrap();
                std::fs::write(output, std::format!("{obj:#?}")).unwrap();
            }
        },

        cli::Command::Savetool { command, .. } => match command {
            cli::SaveToolCommand::Save {
                file: _, output: _, ..
            } => {
                todo!("save tool save isn't implemented yet")
            }

            cli::SaveToolCommand::Load { file, output, .. } => {
                let data = std::fs::read(file).unwrap();

                let key = cli::MM2_SAVE_KEY.as_bytes().try_into().unwrap();
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
        },
    }
}
