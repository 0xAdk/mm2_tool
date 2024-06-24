mod cli;
mod haxe_obj;
mod xxtea;

const MM2_ASSET_KEY: &[u8; 16] = b"aj3fk29dl309f845";

fn main() {
    let cli = <cli::Cli as clap::Parser>::parse();

    match cli.file.extension().unwrap().to_str() {
        Some("dat") => {
            let data = std::fs::read_to_string(cli.file).unwrap();
            let data = &mut data.as_str();

            let obj = haxe_obj::parse(data).unwrap();
            println!("{obj:#?}");
        }

        Some("csv") => {
            let mut data = std::fs::read(cli.file).unwrap();
            {
                let data = match bytemuck::try_cast_slice_mut(&mut data) {
                    Ok(data) => data,
                    Err(err) => {
                        todo!("{}", err)
                    }
                };

                xxtea::decrypt(data, MM2_ASSET_KEY).unwrap();
            }
            let data = std::str::from_utf8(&data).unwrap();
            println!("{}", data);
        }
        _ => panic!("Unsupported file type"),
    }
}
