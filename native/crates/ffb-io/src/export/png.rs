pub struct Image {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub data: Vec<u8>,
}

pub fn decode(bytes: &[u8]) -> Result<Image, String> {
    let mut decoder = ::png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(::png::Transformations::EXPAND | ::png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let (w, h) = (reader.info().width as usize, reader.info().height as usize);
    if w == 0 || h == 0 || w.saturating_mul(h) > 1 << 26 {
        return Err("the texture is empty or larger than 64 megapixels".into());
    }
    let mut data = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut data).map_err(|e| e.to_string())?;
    let channels = match info.color_type {
        ::png::ColorType::Grayscale => 1,
        ::png::ColorType::GrayscaleAlpha => 2,
        ::png::ColorType::Rgb => 3,
        ::png::ColorType::Rgba => 4,
        ::png::ColorType::Indexed => return Err("indexed texture was not expanded".into()),
    };
    data.truncate(info.buffer_size());
    Ok(Image { width: w, height: h, channels, data })
}

pub fn encode(width: u32, height: u32, channels: u8, data: &[u8]) -> Result<Vec<u8>, String> {
    let color = match channels {
        1 => ::png::ColorType::Grayscale,
        2 => ::png::ColorType::GrayscaleAlpha,
        3 => ::png::ColorType::Rgb,
        4 => ::png::ColorType::Rgba,
        _ => return Err("channels must be 1, 2, 3 or 4".into()),
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
