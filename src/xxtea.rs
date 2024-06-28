use std::num::Wrapping;

#[derive(Debug, Clone, Copy)]
pub enum TeaMode {
    Encrypt,
    Decrypt,
}

pub fn decrypt(data: &mut [u32], key: &[u8; 16]) -> Result<(), ()> {
    crypt(TeaMode::Decrypt, data, key)
}

pub fn encrypt(data: &mut [u32], key: &[u8; 16]) -> Result<(), ()> {
    crypt(TeaMode::Encrypt, data, key)
}

const TEA_DELTA: Wrapping<u32> = Wrapping(0x9e_37_79_b9);

// From the improved version of the reference code in https://w.wiki/AU4y
#[allow(clippy::many_single_char_names)]
pub fn crypt(mode: TeaMode, data: &mut [u32], key: &[u8; 16]) -> Result<(), ()> {
    if data.len() < 2 {
        return Err(());
    }

    let v: &mut [Wrapping<u32>] = bytemuck::cast_slice_mut(data);
    let key: &[Wrapping<u32>; 4] = bytemuck::cast_ref(key);

    let mx = move |y, z, sum, p, e: Wrapping<u32>| {
        ((z >> 5 ^ y << 2) + (y >> 3 ^ z << 4)) ^ ((sum ^ y) + (key[(p & 3) ^ e.0 as usize] ^ z))
    };

    let rounds = 6 + 52 / v.len();
    match mode {
        TeaMode::Encrypt => {
            let mut sum = Wrapping(0);
            let mut z = v[v.len() - 1];
            for _ in 0..rounds {
                sum += TEA_DELTA;
                let e = (sum >> 2) & Wrapping(3);
                let mut p = 0;
                while p < v.len() - 1 {
                    let y = v[p + 1];
                    v[p] += mx(y, z, sum, p, e);
                    z = v[p];
                    p += 1;
                }
                let y = v[0];
                v[v.len() - 1] += mx(y, z, sum, p, e);
                z = v[v.len() - 1];
            }
        }

        TeaMode::Decrypt => {
            let mut sum = Wrapping(u32::try_from(rounds).unwrap()) * TEA_DELTA;
            let mut y = v[0];
            for _ in 0..rounds {
                let e = (sum >> 2) & Wrapping(3);
                let mut p = v.len() - 1;
                while p > 0 {
                    let z = v[p - 1];
                    v[p] -= mx(y, z, sum, p, e);
                    y = v[p];
                    p -= 1;
                }
                let z = v[v.len() - 1];
                v[0] -= mx(y, z, sum, p, e);
                y = v[0];
                sum -= TEA_DELTA;
            }
        }
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum CryptPadError {
    #[error(transparent)]
    FailedCast(#[from] bytemuck::PodCastError),

    #[error(transparent)]
    InvalidUtf8Data(#[from] std::str::Utf8Error),
}

pub fn decrypt_with_padding(mut data: Vec<u8>, key: &[u8; 16]) -> Result<Vec<u8>, CryptPadError> {
    {
        let data = bytemuck::try_cast_slice_mut(&mut data)?;
        decrypt(data, key).unwrap();
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

    Ok(data)
}

pub fn encrypt_with_padding(mut data: Vec<u8>, key: &[u8; 16]) -> Result<Vec<u8>, CryptPadError> {
    {
        data.resize(data.len().next_multiple_of(4), 0);
        let data = bytemuck::try_cast_slice_mut(&mut data)?;
        encrypt(data, key).unwrap();
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        const KEY: &[u8; 16] = b"aj3fk29dl309f845";
        let data = [0xdead, 0xbeef];

        let mut same_data = data;
        encrypt(&mut same_data, KEY).unwrap();
        decrypt(&mut same_data, KEY).unwrap();

        assert_eq!(data.len(), same_data.len());
        assert_eq!(data, same_data.as_slice());
    }
}
