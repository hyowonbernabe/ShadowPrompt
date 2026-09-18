// Resize/encode pipeline: raw RGBA -> PNG bytes -> data: URL ContentPart for vision requests.

use base64::Engine;
use image::{ImageBuffer, Rgba};

use crate::llm::messages::{ContentPart, ImageUrl};

pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> anyhow::Result<Vec<u8>> {
    let img: ImageBuffer<Rgba<u8>, &[u8]> = ImageBuffer::from_raw(width, height, rgba)
        .ok_or_else(|| anyhow::anyhow!("rgba buffer doesn't match {width}x{height}"))?;
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
    Ok(buf)
}

/// Builds a vision-ready ContentPart::Image from PNG bytes, resizing down first if either
/// dimension exceeds `max_long_edge_px` (design doc: forms.max_image_long_edge_px, same cap
/// reused here for Screenshot Query).
pub fn prepare_for_request(png_bytes: &[u8], max_long_edge_px: u32) -> anyhow::Result<ContentPart> {
    let img = image::load_from_memory(png_bytes)?;
    let (w, h) = (img.width(), img.height());
    let resized = if w.max(h) > max_long_edge_px {
        img.resize(max_long_edge_px, max_long_edge_px, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let mut buf = Vec::new();
    resized.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(ContentPart::Image {
        image_url: ImageUrl { url: format!("data:image/png;base64,{b64}") },
    })
}
