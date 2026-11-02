const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

pub(crate) fn decode(text: &str) -> Result<Vec<u8>, String> {
    let text = text.rsplit_once(',').filter(|(head, _)| head.starts_with("data:")).map_or(text, |(_, tail)| tail);
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut acc = 0u32;
    let mut bits = 0u32;
    let mut padding = 0;
    for c in text.bytes().filter(|c| !c.is_ascii_whitespace()) {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => {
                padding += 1;
                continue;
            }
            _ => return Err("the thumbnail is not base64".into()),
        };
        if padding > 0 {
            return Err("the thumbnail is not base64".into());
        }
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Ok(out)
}

pub(crate) const MAX_THUMBNAIL: usize = 1 << 20;
const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

pub(crate) fn png_from_base64(text: &str) -> Result<Vec<u8>, String> {
    if text.len() > MAX_THUMBNAIL * 2 {
        return Err(format!("the thumbnail is larger than {} KiB", MAX_THUMBNAIL / 1024));
    }
    let bytes = decode(text)?;
    if bytes.len() > MAX_THUMBNAIL {
        return Err(format!("the thumbnail is larger than {} KiB", MAX_THUMBNAIL / 1024));
    }
    if !bytes.starts_with(&PNG) {
        return Err("the thumbnail is not a PNG image".into());
    }
    Ok(bytes)
}
