use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub struct OneTimeToken(String);

impl OneTimeToken {
    pub fn value(&self) -> &String {
        &self.0
    }

    pub fn generate() -> OneTimeToken {
        let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyz012345".chars().collect();

        let mut rng = ChaCha20Rng::from_entropy();
        let mut random_bytes = [0u8; 15];
        rng.fill_bytes(&mut random_bytes);

        let mut token = String::with_capacity(24);
        let mut bit_buffer: u32 = 0;
        let mut bit_count = 0;

        for byte in random_bytes.iter() {
            bit_buffer |= (*byte as u32) << bit_count;
            bit_count += 8;

            while bit_count >= 5 {
                let index = (bit_buffer & 0x1F) as usize;
                token.push(charset[index]);
                bit_buffer >>= 5;
                bit_count -= 5;
            }
        }

        if bit_count > 0 {
            token.push(charset[(bit_buffer & 0x1F) as usize]);
        }

        OneTimeToken(token)
    }
}