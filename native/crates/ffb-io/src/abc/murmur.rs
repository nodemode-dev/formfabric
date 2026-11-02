const C1: u64 = 0x87c3_7b91_1142_53d5;
const C2: u64 = 0x4cf5_ad43_2745_937f;

fn fmix(mut k: u64) -> u64 {
    k ^= k >> 33;
    k = k.wrapping_mul(0xff51_afd7_ed55_8ccd);
    k ^= k >> 33;
    k = k.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    k ^= k >> 33;
    k
}

pub fn x64_128_words(data: &[u8], seed: u32) -> (u64, u64) {
    let mut h1 = u64::from(seed);
    let mut h2 = u64::from(seed);
    let (blocks, tail) = data.as_chunks::<16>();
    for block in blocks {
        let mut k1 = u64::from_le_bytes(block[0..8].try_into().unwrap_or([0; 8]));
        let mut k2 = u64::from_le_bytes(block[8..16].try_into().unwrap_or([0; 8]));
        k1 = k1.wrapping_mul(C1).rotate_left(31).wrapping_mul(C2);
        h1 ^= k1;
        h1 = h1.rotate_left(27).wrapping_add(h2).wrapping_mul(5).wrapping_add(0x52dc_e729);
        k2 = k2.wrapping_mul(C2).rotate_left(33).wrapping_mul(C1);
        h2 ^= k2;
        h2 = h2.rotate_left(31).wrapping_add(h1).wrapping_mul(5).wrapping_add(0x3849_5ab5);
    }
    let mut k1 = 0u64;
    let mut k2 = 0u64;
    for (i, b) in tail.iter().enumerate() {
        if i < 8 {
            k1 |= u64::from(*b) << (8 * i);
        } else {
            k2 |= u64::from(*b) << (8 * (i - 8));
        }
    }
    if tail.len() > 8 {
        h2 ^= k2.wrapping_mul(C2).rotate_left(33).wrapping_mul(C1);
    }
    if !tail.is_empty() {
        h1 ^= k1.wrapping_mul(C1).rotate_left(31).wrapping_mul(C2);
    }
    h1 ^= data.len() as u64;
    h2 ^= data.len() as u64;
    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);
    h1 = fmix(h1);
    h2 = fmix(h2);
    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);
    (h1, h2)
}

pub fn key(data: &[u8]) -> [u8; 16] {
    let (h1, h2) = x64_128_words(data, 0);
    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h1.to_le_bytes());
    out[8..].copy_from_slice(&h2.to_le_bytes());
    out
}
