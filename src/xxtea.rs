use std::num::Wrapping;

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

const TEA_DELTA: Wrapping<u32> = Wrapping(0x9e3779b9);

// From the improved version of the reference code in https://w.wiki/AU4y
pub fn crypt(mode: TeaMode, data: &mut [u32], key: &[u8; 16]) -> Result<(), ()> {
    if data.len() < 2 {
        return Err(());
    }

    let v = bytemuck::cast_slice_mut::<_, Wrapping<_>>(data);
    let key = bytemuck::cast_ref::<_, [Wrapping<u32>; 4]>(key);

    let mx = move |y, z, sum, p, e: Wrapping<u32>| {
        ((z >> 5 ^ y << 2) + (y >> 3 ^ z << 4)) ^ ((sum ^ y) + (key[(p & 3) ^ e.0 as usize] ^ z))
    };

    let rounds = 6 + 52 / v.len();
    match mode {
        TeaMode::Encrypt => {
            let mut sum = Wrapping(0);
            let mut z = v[v.len() - 1];
            for _ in 0..rounds {
                let e = (sum >> 2) & Wrapping(3);

                sum += TEA_DELTA;
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
            let mut sum = Wrapping(rounds as u32) * TEA_DELTA;
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
