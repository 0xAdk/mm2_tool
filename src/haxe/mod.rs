mod cli;
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

        Command::Deserialize { file, output, .. } => {
            let data = std::fs::read_to_string(file).unwrap();
            let data = &mut data.as_str();

            let obj = from_str(data).unwrap();
            std::fs::write(output, std::format!("{obj:#?}")).unwrap();
        }
    }
}
