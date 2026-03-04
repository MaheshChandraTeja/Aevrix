


use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    
    pub rgba: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ImgError {
    #[error("unsupported format or feature not enabled")]
    Unsupported,
    #[error("decode error: {0}")]
    Decode(String),
}


pub fn decode(bytes: &[u8]) -> Result<Image, ImgError> {
    if bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
        #[cfg(feature = "png")]
        { return decode_png(bytes); }
        #[cfg(not(feature = "png"))]
        { return Err(ImgError::Unsupported); }
    }

    
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        #[cfg(feature = "jpeg")]
        { return decode_jpeg(bytes); }
        #[cfg(not(feature = "jpeg"))]
        { return Err(ImgError::Unsupported); }
    }

    
    if bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a") {
        #[cfg(feature = "gif")]
        { return decode_gif(bytes); }
        #[cfg(not(feature = "gif"))]
        { return Err(ImgError::Unsupported); }
    }

    Err(ImgError::Unsupported)
}

#[cfg(feature = "png")]
fn decode_png(bytes: &[u8]) -> Result<Image, ImgError> {
    use png::{Decoder, ColorType};
    let mut dec = Decoder::new(bytes);
    dec.set_transformations(png::Transformations::EXPAND);
    let mut reader = dec.read_info().map_err(|e| ImgError::Decode(e.to_string()))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| ImgError::Decode(e.to_string()))?;
    let w = info.width;
    let h = info.height;

    let rgba = match info.color_type {
        ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        ColorType::Rgb => {
            let mut out = Vec::with_capacity((w*h*4) as usize);
            for chunk in buf[..info.buffer_size()].chunks_exact(3) {
                out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            out
        }
        ColorType::Grayscale => {
            let mut out = Vec::with_capacity((w*h*4) as usize);
            for &g in &buf[..info.buffer_size()] {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        ColorType::Indexed => {
            
            return Err(ImgError::Decode("indexed PNG without expansion".into()));
        }
        _ => return Err(ImgError::Decode("unsupported PNG color type".into())),
    };

    Ok(Image { width: w, height: h, rgba })
}

#[cfg(feature = "jpeg")]
fn decode_jpeg(bytes: &[u8]) -> Result<Image, ImgError> {
    use jpeg_decoder as jd;
    let mut dec = jd::Decoder::new(bytes);
    let pixels = dec.decode().map_err(|e| ImgError::Decode(e.to_string()))?;
    let meta = dec.info().ok_or_else(|| ImgError::Decode("no jpeg info".into()))?;
    let w = meta.width as u32;
    let h = meta.height as u32;

    let rgba = match meta.pixel_format {
        jd::PixelFormat::L8 => {
            let mut out = Vec::with_capacity((w*h*4) as usize);
            for &g in &pixels {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        jd::PixelFormat::RGB24 => {
            let mut out = Vec::with_capacity((w*h*4) as usize);
            for chunk in pixels.chunks_exact(3) {
                out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            out
        }
        _ => return Err(ImgError::Decode("unsupported JPEG pixel format".into())),
    };

    Ok(Image { width: w, height: h, rgba })
}

#[cfg(feature = "gif")]
fn decode_gif(bytes: &[u8]) -> Result<Image, ImgError> {
    use gif::SetParameter;
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = decoder.read_info(std::io::Cursor::new(bytes)).map_err(|e| ImgError::Decode(e.to_string()))?;

    
    if let Some(frame) = reader.read_next_frame().map_err(|e| ImgError::Decode(e.to_string()))? {
        let w = reader.width() as u32;
        let h = reader.height() as u32;
        let rgba = frame.buffer.to_vec();
        Ok(Image { width: w, height: h, rgba })
    } else {
        Err(ImgError::Decode("empty GIF".into()))
    }
}
