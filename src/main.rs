#![deny(clippy::pedantic)]

mod cli;
mod crypt;
mod haxe_obj;
mod xxtea;

const MM2_ASSET_KEY: &[u8; 16] = b"aj3fk29dl309f845";

fn main() {
    let cli = <cli::Cli as clap::Parser>::parse();

    match cli.command {
        cli::Command::Crypt {
            command, output, ..
        } => match command {
            cli::CryptCommand::Encrypt { file, .. } => {
                let data = std::fs::read(file).unwrap();
                let data = encrypt_mm2_asset(data).unwrap_or_else(|err| panic!("{err:?}"));
                std::fs::write(output, data).unwrap();
            }

            cli::CryptCommand::Decrypt { file, .. } => {
                let data = std::fs::read(file).unwrap();
                let data = decrypt_mm2_asset(data).unwrap_or_else(|err| panic!("{err:?}"));
                std::fs::write(output, data).unwrap();
            }
        },
        cli::Command::Haxe { command, output, .. } => match command {
            cli::HaxeCommand::Serialize { file: _, .. } => {
                todo!("serializing files isn't implemented yet")
            }
            cli::HaxeCommand::Deserialize { file, .. } => {
                let data = std::fs::read_to_string(file).unwrap();
                let data = &mut data.as_str();

                let obj = haxe_obj::parse(data).unwrap();
                std::fs::write(output, std::format!("{obj:?}")).unwrap();
            }
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AssetDecryptError {
    #[error(transparent)]
    FailedCast(#[from] bytemuck::PodCastError),

    #[error(transparent)]
    InvalidUtf8Data(#[from] std::str::Utf8Error),
}

fn decrypt_mm2_asset(mut data: Vec<u8>) -> Result<String, AssetDecryptError> {
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

fn encrypt_mm2_asset(mut data: Vec<u8>) -> Result<Vec<u8>, AssetDecryptError> {
    {
        data.resize(data.len().next_multiple_of(4), 0);
        let data = bytemuck::try_cast_slice_mut(&mut data)?;
        xxtea::encrypt(data, MM2_ASSET_KEY).unwrap();
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xxtea_roundtrip() {
        const KEY: &[u8; 16] = b"aj3fk29dl309f845";
        let data = [0xdead, 0xbeef];

        let mut same_data = data;
        xxtea::encrypt(&mut same_data, KEY).unwrap();
        xxtea::decrypt(&mut same_data, KEY).unwrap();

        assert_eq!(data.len(), same_data.len());
        assert_eq!(data, same_data.as_slice());
    }
}
