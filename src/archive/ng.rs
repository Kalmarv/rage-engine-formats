//! The archive ciphers: NG (GTA V's own 17-round table cipher) and AES-256 ECB. Both decrypt whole 16-byte blocks
//! in place and leave a trailing partial block as it is.
use aes::{
    Aes256,
    cipher::{Array, BlockCipherDecrypt, KeyInit},
};

use crate::archive::keys::{Keys, NG_ROUNDS as ROUNDS, NgKey, NgRound};
use crate::{Error, Result};

/// For each of a round's four output words, the four input bytes it mixes.
type Picks = [[usize; 4]; 4];

/// Rounds 1, 2 and 17: each output word mixes four consecutive input bytes.
const CONSECUTIVE: Picks = [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]];

/// Rounds 3 to 16: the input bytes are taken diagonally (a ShiftRows-like spread).
const DIAGONAL: Picks = [[0, 7, 10, 13], [1, 4, 11, 14], [2, 5, 8, 15], [3, 6, 9, 12]];

/// One round: word `w` of the output is the round key's word `w` xor the table entries of its four input bytes (the
/// table of a byte is the one for its position in the block).
fn round(b: &[u8; 16], k: &[u32; 4], t: &NgRound, picks: &Picks) -> [u8; 16] {
    let mut out = [0u8; 16];
    for ((word, pick), &key) in out.as_chunks_mut::<4>().0.iter_mut().zip(picks).zip(k) {
        let x = pick.iter().fold(key, |x, &p| {
            // `p` is a position in the block (below 16) and a byte indexes the table's 256 entries
            let entry = t
                .get(p)
                .zip(b.get(p))
                .and_then(|(table, &byte)| table.get(usize::from(byte)))
                .copied()
                .unwrap_or_default();
            x ^ entry
        });
        *word = x.to_le_bytes();
    }
    out
}

/// One block through the 17 rounds; `tables` holds [`ROUNDS`] rounds' tables (checked by the caller).
fn ng_block(block: &mut [u8; 16], key: &NgKey, tables: &[NgRound]) {
    let mut b = *block;
    for (r, (k, t)) in key.as_chunks::<4>().0.iter().zip(tables).enumerate() {
        let picks = if r < 2 || r == ROUNDS - 1 {
            &CONSECUTIVE
        } else {
            &DIAGONAL
        };
        b = round(&b, k, t, picks);
    }
    *block = b;
}

/// NG-decrypt `data` in place with the key chosen by `name` and `length` (an archive's file name and size, or an
/// entry's name and its size as the archive records it).
pub fn ng_decrypt(keys: &Keys, name: &str, length: u32, data: &mut [u8]) -> Result<()> {
    let key = keys.ng_key(name, length)?;
    if keys.ng_tables.len() != ROUNDS {
        return Err(Error::Keys(format!(
            "an NG archive needs the tables of {ROUNDS} rounds, the key material has {}",
            keys.ng_tables.len()
        )));
    }
    for block in data.as_chunks_mut::<16>().0 {
        ng_block(block, key, &keys.ng_tables);
    }
    Ok(())
}

/// AES-256-ECB-decrypt `data` of the archive at `archive` in place (archives flagged AES: two of Enhanced's, both
/// cutscene archives nested in `x64c.rpf`).
pub fn aes_decrypt(keys: &Keys, archive: &str, data: &mut [u8]) -> Result<()> {
    let key = keys
        .aes_key
        .ok_or_else(|| Error::Keys(format!("{archive}: an AES archive needs the AES key")))?;
    let cipher = Aes256::new(&Array::from(key));
    for block in data.as_chunks_mut::<16>().0 {
        let mut b = Array::from(*block);
        cipher.decrypt_block(&mut b);
        block.copy_from_slice(&b);
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    reason = "a reference written the plain way, over fixtures"
)]
mod tests {
    use super::*;

    /// The cipher written out round by round with plain indexing, as the format is described.
    fn reference(block: [u8; 16], key: &NgKey, tables: &[NgRound]) -> [u8; 16] {
        let mut b = block;
        for r in 0..17 {
            let k = &key[4 * r..4 * r + 4];
            let mut out = [0u8; 16];
            for w in 0..4 {
                let pick: [usize; 4] = if r < 2 || r == 16 {
                    [4 * w, 4 * w + 1, 4 * w + 2, 4 * w + 3]
                } else {
                    [[0, 7, 10, 13], [1, 4, 11, 14], [2, 5, 8, 15], [3, 6, 9, 12]][w]
                };
                let x = tables[r][pick[0]][usize::from(b[pick[0]])]
                    ^ tables[r][pick[1]][usize::from(b[pick[1]])]
                    ^ tables[r][pick[2]][usize::from(b[pick[2]])]
                    ^ tables[r][pick[3]][usize::from(b[pick[3]])]
                    ^ k[w];
                out[4 * w..4 * w + 4].copy_from_slice(&x.to_le_bytes());
            }
            b = out;
        }
        b
    }

    /// A deterministic word stream (xorshift) for tables and keys no real archive uses.
    fn words(seed: u32) -> impl Iterator<Item = u32> {
        let mut x = seed;
        std::iter::repeat_with(move || {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x
        })
    }

    #[test]
    fn the_rounds_match_the_plain_description() {
        let mut w = words(0x1234_5678);
        let key: NgKey = std::array::from_fn(|_| w.next().unwrap());
        let tables: Vec<NgRound> = (0..ROUNDS)
            .map(|_| std::array::from_fn(|_| std::array::from_fn(|_| w.next().unwrap())))
            .collect();
        for seed in 1..64u32 {
            let block: [u8; 16] =
                std::array::from_fn(|i| (seed.wrapping_mul(0x9E37_79B1) >> (i % 24)) as u8 ^ i as u8);
            let mut got = block;
            ng_block(&mut got, &key, &tables);
            assert_eq!(got, reference(block, &key, &tables), "block {seed}");
        }
    }
}
