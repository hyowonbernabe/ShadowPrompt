// Screen capture, clipboard, image pipeline. No ocr.rs — OCR is fully removed in v3 (design doc
// §3), pure vision handles what OCR used to.

pub mod clipboard;
pub mod image;
pub mod screen;
