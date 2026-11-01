pub fn encode(width: u32, height: u32, channels: u8, data: &[u8]) -> Result<Vec<u8>, String> {
    let color = match channels {
        1 => ::png::ColorType::Grayscale,
        3 => ::png::ColorType::Rgb,
        4 => ::png::ColorType::Rgba,
        _ => return Err("channels must be 1, 3 or 4".into()),
    };
    if data.len() != width as usize * height as usize * channels as usize || width == 0 || height == 0 {
        return Err("image data does not match its size".into());
    }
    let mut out = Vec::new();
    {
        let mut encoder = ::png::Encoder::new(&mut out, width, height);
        encoder.set_color(color);
        encoder.set_depth(::png::BitDepth::Eight);
        encoder.set_compression(::png::Compression::Fast);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(data).map_err(|e| e.to_string())?;
    }
    Ok(out)
}
