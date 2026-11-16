use serde_json::{json, Value};

use crate::{args, Command, CommandError, Group, Session, EXTERNAL};

pub(crate) const MAX_BYTES: usize = 512 * 1024 * 1024;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub(crate) fn decode_base64(text: &str, limit: usize) -> Result<Vec<u8>, CommandError> {
    let body = text.trim().split_once(',').filter(|(head, _)| head.starts_with("data:")).map_or(text.trim(), |(_, rest)| rest);
    let mut out: Vec<u8> = Vec::with_capacity(body.len() / 4 * 3);
    let (mut acc, mut bits, mut pad) = (0u32, 0u32, 0usize);
    for ch in body.bytes() {
        let v = match ch {
            b'A'..=b'Z' => ch - b'A',
            b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => {
                pad += 1;
                continue;
            }
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return Err(bad("data is not base64")),
        };
        if pad > 0 {
            return Err(bad("data is not base64: characters after padding"));
        }
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
            if out.len() > limit {
                return Err(bad(format!("data is larger than {} MB", limit / (1024 * 1024))));
            }
        }
    }
    if pad > 2 || (bits == 6) || (pad > 0 && (body.bytes().filter(|c| !c.is_ascii_whitespace()).count() % 4 != 0)) {
        return Err(bad("data is not base64: bad length"));
    }
    Ok(out)
}

fn magic_ok(kind: &str, bytes: &[u8]) -> bool {
    match kind {
        "png" => bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]),
        "gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "webm" => bytes.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]),
        "mp4" => bytes.len() >= 8 && &bytes[4..8] == b"ftyp",
        _ => false,
    }
}

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "Absolute output path whose extension matches kind."},
            "data": {"type": "string", "description": "The file bytes, base64 encoded (a data: URL prefix is accepted)."},
            "kind": {"type": "string", "enum": ["png", "webm", "mp4", "gif"]}
        },
        "required": ["path", "data", "kind"],
        "additionalProperties": false
    })
}

fn write(_: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let kind = args::string(a, "kind")?;
    if !matches!(kind, "png" | "webm" | "mp4" | "gif") {
        return Err(bad("kind must be png, webm, mp4 or gif"));
    }
    let path = args::file_path(a, "path", Some(kind))?;
    let data = args::string(a, "data")?;
    if data.len() / 4 * 3 > MAX_BYTES + 3 {
        return Err(bad(format!("data is larger than {} MB", MAX_BYTES / (1024 * 1024))));
    }
    let bytes = decode_base64(data, MAX_BYTES)?;
    if bytes.is_empty() {
        return Err(bad("data is empty"));
    }
    if !magic_ok(kind, &bytes) {
        return Err(bad(format!("data is not a {kind} file")));
    }
    args::write_atomic(&path, &bytes)?;
    Ok(json!({"path": path, "kind": kind, "bytes": bytes.len()}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![Command {
        id: "capture.write",
        mcp: "capture_write",
        group: Group::Export,
        label: "Write Capture",
        description: "Write bytes captured in the window (a png snapshot, a webm or mp4 video, or a gif) to a file. The path must be absolute with the extension of the kind, the bytes must carry that format's signature, and the size is capped at 512 MB. The file is replaced atomically.",
        keys: &[],
        hints: EXTERNAL,
        schema,
        run: write,
    }]
}
