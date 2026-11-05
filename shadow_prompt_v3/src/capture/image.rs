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
///
/// Sent as JPEG, not PNG: measured on a 1920x1080 screen capture, PNG was 1,723 KB and JPEG at
/// quality 85 was 172 KB with text still crisp. Every screenshot in the conversation is re-sent
/// with each question, so on the slow, flaky connections this is used on, upload size is what
/// decides whether an answer arrives in seconds or minutes.
pub fn prepare_for_request(png_bytes: &[u8], max_long_edge_px: u32) -> anyhow::Result<ContentPart> {
    let img = image::load_from_memory(png_bytes)?;
    let (w, h) = (img.width(), img.height());
    let resized = if w.max(h) > max_long_edge_px {
        img.resize(max_long_edge_px, max_long_edge_px, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let buf = encode_jpeg_under(image::DynamicImage::ImageRgb8(resized.to_rgb8()), MAX_IMAGE_BYTES)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(ContentPart::Image {
        image_url: ImageUrl { url: format!("data:image/jpeg;base64,{b64}") },
    })
}

/// One screenshot's byte budget. Busy screens (photos, gradients) can still be large as JPEG, so
/// shrink to 3/4 size until under it — bounded to 4 steps.
pub const MAX_IMAGE_BYTES: usize = 1_200_000;
const JPEG_QUALITY: u8 = 85;

fn encode_jpeg_under(mut img: image::DynamicImage, max_bytes: usize) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    for _ in 0..4 {
        buf.clear();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY).encode_image(&img)?;
        if buf.len() <= max_bytes {
            break;
        }
        let (w, h) = (img.width() * 3 / 4, img.height() * 3 / 4);
        img = img.resize(w.max(1), h.max(1), image::imageops::FilterType::Triangle);
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshots_go_out_as_small_jpeg() {
        // A noisy 1600x900 image: the worst case for size.
        let rgba: Vec<u8> = (0..1600u64 * 900 * 4).map(|i| ((i * 7919) % 251) as u8).collect();
        let png = encode_png(1600, 900, &rgba).unwrap();
        let ContentPart::Image { image_url } = prepare_for_request(&png, 1600).unwrap() else { panic!() };
        assert!(image_url.url.starts_with("data:image/jpeg;base64,"));
        let b64 = image_url.url.trim_start_matches("data:image/jpeg;base64,");
        assert!(b64.len() * 3 / 4 <= MAX_IMAGE_BYTES, "{} bytes", b64.len() * 3 / 4);
    }
}
