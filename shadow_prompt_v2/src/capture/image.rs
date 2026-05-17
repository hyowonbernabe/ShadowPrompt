// Image pipeline: resize to <=1568px long edge, base64 encode as data URL.

use crate::llm::messages::{ContentPart, ImageUrl};

pub const MAX_LONG_EDGE: u32 = 1568;

pub fn prepare_for_request(png_bytes: &[u8]) -> anyhow::Result<ContentPart> {
    let img = image::load_from_memory(png_bytes)?;
    let resized = if img.width().max(img.height()) > MAX_LONG_EDGE {
        img.resize(MAX_LONG_EDGE, MAX_LONG_EDGE, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut out = Vec::new();
    resized.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&out);
    Ok(ContentPart::Image {
        image_url: ImageUrl {
            url: format!("data:image/png;base64,{}", b64),
        },
    })
}
