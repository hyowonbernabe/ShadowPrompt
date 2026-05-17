// Hex color parsing + COLORREF helpers.

use windows::Win32::Foundation::COLORREF;

pub fn parse_hex(s: &str) -> anyhow::Result<(u8, u8, u8)> {
    let h = s.trim_start_matches('#');
    if h.len() != 6 {
        anyhow::bail!("color '{s}' must be #RRGGBB");
    }
    let r = u8::from_str_radix(&h[0..2], 16)?;
    let g = u8::from_str_radix(&h[2..4], 16)?;
    let b = u8::from_str_radix(&h[4..6], 16)?;
    Ok((r, g, b))
}

pub fn colorref(rgb: (u8, u8, u8)) -> COLORREF {
    let (r, g, b) = rgb;
    COLORREF(((b as u32) << 16) | ((g as u32) << 8) | (r as u32))
}
