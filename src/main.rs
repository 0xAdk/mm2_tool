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

        Some("csv" | "txt") => match decrypt_mm2_asset(cli.file) {
            Ok(s) => println!("{s}"),
            Err(err) => println!("{err:?}"),
        },
        _ => panic!("Unsupported file type"),
    }
}

#[derive(thiserror::Error, Debug)]
enum AssetDecryptError {
    #[error(transparent)]
    FailedCast(#[from] bytemuck::PodCastError),

    #[error(transparent)]
    InvalidUtf8Data(#[from] std::str::Utf8Error),
}

fn decrypt_mm2_asset(file: std::path::PathBuf) -> Result<String, AssetDecryptError> {
    let mut data = std::fs::read(file).unwrap();
    {
        let data = bytemuck::try_cast_slice_mut(&mut data)?;
        xxtea::decrypt(data, MM2_ASSET_KEY).unwrap();
    }

    // pop at most 4 nul padding bytes from the end
    for _ in 0..4 {
        match data.last() {
            Some(&0) => {
                data.pop();
            }
            _ => break,
        }
    }

    Ok(std::str::from_utf8(&data)?.to_string())
}
