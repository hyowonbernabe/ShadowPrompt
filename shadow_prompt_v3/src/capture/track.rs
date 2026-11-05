// Patch tracker for Screen Answer dots: a small reference patch cut from the screenshot the model
// saw is searched for in fresh captures, so a dot follows its answer when the page scrolls.
// Zero-mean normalized cross-correlation (NCC): insensitive to uniform brightness shifts (hover
// highlights), so "~90% similar" means score >= 0.9, not pixel-perfect.
//
// Safety bias (anti-goal: a confidently wrong dot): a flat patch (nothing to lock onto), a best
// match under the threshold, or a runner-up nearly as good as the best (repeated identical
// options) all return None — the dot hides rather than guesses.

/// Coarse search runs at 1/4 resolution; the winner(s) get refined at full resolution.
pub const COARSE_FACTOR: usize = 4;
/// Patch size at coarse resolution (full-res = x4). Wide and short: an option row is a radio +
/// its label text, laid out horizontally.
const PATCH_Q: (usize, usize) = (40, 16);
/// Coarse horizontal search reach either side of the last known spot (coarse px). Vertical search
/// covers the whole monitor — scrolling is the main thing being tracked.
// ponytail: naive O(positions x patch) scan, ~30M mult-adds per dot per tick. Integral images /
// FFT if many dots or slow CPUs ever make the tick visibly expensive.
const SEARCH_X_Q: i32 = 64;
/// Full-res refine reach around each coarse candidate.
const REFINE_PX: i32 = 6;
/// Below this std-dev (0..255 luminance) the patch is effectively blank background.
const MIN_PATCH_STD: f32 = 4.0;
/// Runner-up within this much of the best = ambiguous, hide.
const AMBIGUITY_MARGIN: f32 = 0.02;

#[derive(Clone)]
pub struct Gray {
    pub w: usize,
    pub h: usize,
    pub px: Vec<f32>,
}

impl Gray {
    /// RGBA -> luminance, box-downscaled by `factor` (1 = full res).
    pub fn from_rgba(w: usize, h: usize, rgba: &[u8], factor: usize) -> Self {
        let (ow, oh) = (w / factor, h / factor);
        let mut px = vec![0f32; ow * oh];
        let norm = 1.0 / (factor * factor) as f32;
        for oy in 0..oh {
            for ox in 0..ow {
                let mut acc = 0f32;
                for dy in 0..factor {
                    let row = (oy * factor + dy) * w;
                    for dx in 0..factor {
                        let i = (row + ox * factor + dx) * 4;
                        acc += 0.299 * rgba[i] as f32 + 0.587 * rgba[i + 1] as f32 + 0.114 * rgba[i + 2] as f32;
                    }
                }
                px[oy * ow + ox] = acc * norm;
            }
        }
        Self { w: ow, h: oh, px }
    }

    fn crop(&self, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut px = Vec::with_capacity(w * h);
        for r in y..y + h {
            px.extend_from_slice(&self.px[r * self.w + x..r * self.w + x + w]);
        }
        Self { w, h, px }
    }
}

/// A patch pre-processed for NCC: zero-mean values + their norm.
#[derive(Clone)]
struct Template {
    w: usize,
    h: usize,
    zm: Vec<f32>,
    norm: f32,
}

impl Template {
    fn new(g: &Gray) -> Option<Self> {
        let n = g.px.len() as f32;
        let mean = g.px.iter().sum::<f32>() / n;
        let zm: Vec<f32> = g.px.iter().map(|v| v - mean).collect();
        let ss: f32 = zm.iter().map(|v| v * v).sum();
        if (ss / n).sqrt() < MIN_PATCH_STD {
            return None;
        }
        Some(Self { w: g.w, h: g.h, zm, norm: ss.sqrt() })
    }

    /// NCC of this template against `img` with the template's top-left at (x, y).
    fn score(&self, img: &Gray, x: usize, y: usize) -> f32 {
        let n = (self.w * self.h) as f32;
        let (mut s, mut s2, mut sp) = (0f32, 0f32, 0f32);
        for r in 0..self.h {
            let row = &img.px[(y + r) * img.w + x..(y + r) * img.w + x + self.w];
            let trow = &self.zm[r * self.w..(r + 1) * self.w];
            for (v, t) in row.iter().zip(trow) {
                s += v;
                s2 += v * v;
                sp += v * t;
            }
        }
        let var = s2 - s * s / n;
        if var <= 1e-3 {
            return 0.0;
        }
        sp / (self.norm * var.sqrt())
    }
}

/// One tracked dot. Coordinates are monitor-local, full-res pixels.
#[derive(Clone)]
pub struct Anchor {
    coarse: Template,
    full: Template,
    /// Dot position relative to the full-res patch's top-left.
    offset: (i32, i32),
    /// Full-res top-left of the patch where it was last found.
    last: (i32, i32),
}

/// Cuts a reference patch centred on the anchor point (clamped inside the image) and remembers
/// where the target dot sits relative to it. None if the patch is blank (nothing to track).
pub fn make_anchor(full: &Gray, coarse: &Gray, anchor: (i32, i32), target: (i32, i32)) -> Option<Anchor> {
    let (pwq, phq) = PATCH_Q;
    if coarse.w < pwq || coarse.h < phq {
        return None;
    }
    let qx = (anchor.0 / COARSE_FACTOR as i32 - pwq as i32 / 2).clamp(0, (coarse.w - pwq) as i32) as usize;
    let qy = (anchor.1 / COARSE_FACTOR as i32 - phq as i32 / 2).clamp(0, (coarse.h - phq) as i32) as usize;
    let (fx, fy) = (qx * COARSE_FACTOR, qy * COARSE_FACTOR);
    let coarse_t = Template::new(&coarse.crop(qx, qy, pwq, phq))?;
    let full_t = Template::new(&full.crop(fx, fy, pwq * COARSE_FACTOR, phq * COARSE_FACTOR))?;
    Some(Anchor {
        coarse: coarse_t,
        full: full_t,
        offset: (target.0 - fx as i32, target.1 - fy as i32),
        last: (fx as i32, fy as i32),
    })
}

/// Finds the anchor in a fresh capture. Returns the dot's new monitor-local position, or None
/// if not confidently found (see module comment).
pub fn locate(a: &mut Anchor, full: &Gray, coarse: &Gray, threshold: f32) -> Option<(i32, i32)> {
    let (pw, ph) = (a.coarse.w, a.coarse.h);
    if coarse.w < pw || coarse.h < ph {
        return None;
    }
    let last_qx = a.last.0 / COARSE_FACTOR as i32;
    let x0 = (last_qx - SEARCH_X_Q).max(0) as usize;
    let x1 = (last_qx + SEARCH_X_Q).min((coarse.w - pw) as i32) as usize;

    // Best and best-not-overlapping-the-best, at coarse res.
    let mut scores = Vec::with_capacity((x1 + 1 - x0) * (coarse.h - ph + 1));
    for y in 0..=coarse.h - ph {
        for x in x0..=x1 {
            scores.push((a.coarse.score(coarse, x, y), x, y));
        }
    }
    let best = *scores.iter().max_by(|p, q| p.0.total_cmp(&q.0))?;
    let runner = scores
        .iter()
        .filter(|(_, x, y)| x.abs_diff(best.1) > pw / 2 || y.abs_diff(best.2) > ph / 2)
        .max_by(|p, q| p.0.total_cmp(&q.0))
        .copied();

    let refine = |qx: usize, qy: usize| -> (f32, i32, i32) {
        let cx = (qx * COARSE_FACTOR) as i32;
        let cy = (qy * COARSE_FACTOR) as i32;
        let mut top = (f32::MIN, cx, cy);
        for y in (cy - REFINE_PX).max(0)..=(cy + REFINE_PX).min((full.h - a.full.h) as i32) {
            for x in (cx - REFINE_PX).max(0)..=(cx + REFINE_PX).min((full.w - a.full.w) as i32) {
                let s = a.full.score(full, x as usize, y as usize);
                if s > top.0 {
                    top = (s, x, y);
                }
            }
        }
        top
    };

    // Refine both candidates at full res (fine detail tells "Option 1" from "Option 2" better
    // than the blurred coarse image), then the stronger one must clearly beat the other.
    let primary = refine(best.1, best.2);
    let (win, other) = match runner {
        Some(r) => {
            let alt = refine(r.1, r.2);
            if alt.0 > primary.0 { (alt, Some(primary)) } else { (primary, Some(alt)) }
        }
        None => (primary, None),
    };
    if other.is_some_and(|o| o.0 > win.0 - AMBIGUITY_MARGIN) {
        return None;
    }
    if win.0 < threshold {
        return None;
    }
    a.last = (win.1, win.2);
    Some((win.1 + a.offset.0, win.2 + a.offset.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic textured "page": pseudo-random blocks, so every patch is unique.
    fn textured(w: usize, h: usize, seed: u32) -> Vec<u8> {
        let mut out = vec![255u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let v = ((x / 3) as u32).wrapping_mul(2654435761) ^ ((y / 3) as u32).wrapping_mul(40503) ^ seed;
                let g = (v.wrapping_mul(2246822519) >> 24) as u8;
                let i = (y * w + x) * 4;
                out[i..i + 3].copy_from_slice(&[g, g, g]);
            }
        }
        out
    }

    fn shift_down(rgba: &[u8], w: usize, h: usize, dy: usize) -> Vec<u8> {
        let mut out = vec![255u8; rgba.len()];
        out[dy * w * 4..].copy_from_slice(&rgba[..(h - dy) * w * 4]);
        out
    }

    fn grays(rgba: &[u8], w: usize, h: usize) -> (Gray, Gray) {
        (Gray::from_rgba(w, h, rgba, 1), Gray::from_rgba(w, h, rgba, COARSE_FACTOR))
    }

    #[test]
    fn follows_a_scroll() {
        let (w, h) = (640, 480);
        let page = textured(w, h, 7);
        let (f, c) = grays(&page, w, h);
        let mut a = make_anchor(&f, &c, (300, 120), (250, 125)).expect("textured patch is trackable");

        let scrolled = shift_down(&page, w, h, 57);
        let (f2, c2) = grays(&scrolled, w, h);
        assert_eq!(locate(&mut a, &f2, &c2, 0.9), Some((250, 182)));
    }

    #[test]
    fn survives_small_occlusion() {
        let (w, h) = (640, 480);
        let mut page = textured(w, h, 3);
        let (f, c) = grays(&page, w, h);
        let mut a = make_anchor(&f, &c, (320, 240), (320, 240)).unwrap();
        // A "mouse cursor" of 12x12 black pixels inside the patch.
        for y in 236..248 {
            for x in 316..328 {
                let i = (y * w + x) * 4;
                page[i..i + 3].copy_from_slice(&[0, 0, 0]);
            }
        }
        let (f2, c2) = grays(&page, w, h);
        assert_eq!(locate(&mut a, &f2, &c2, 0.9), Some((320, 240)));
    }

    #[test]
    fn blank_patch_is_untrackable() {
        let (w, h) = (640, 480);
        let page = vec![255u8; w * h * 4];
        let (f, c) = grays(&page, w, h);
        assert!(make_anchor(&f, &c, (320, 240), (320, 240)).is_none());
    }

    #[test]
    fn duplicate_pattern_is_ambiguous() {
        let (w, h) = (640, 480);
        let mut page = textured(w, h, 11);
        // Copy rows 100..200 onto rows 300..400: the patch now exists twice.
        let src = page[100 * w * 4..200 * w * 4].to_vec();
        page[300 * w * 4..400 * w * 4].copy_from_slice(&src);
        let (f, c) = grays(&page, w, h);
        let mut a = make_anchor(&f, &c, (320, 150), (320, 150)).unwrap();
        assert_eq!(locate(&mut a, &f, &c, 0.9), None);
    }

    #[test]
    fn gone_from_screen_is_none() {
        let (w, h) = (640, 480);
        let (f, c) = grays(&textured(w, h, 1), w, h);
        let mut a = make_anchor(&f, &c, (320, 240), (320, 240)).unwrap();
        let (f2, c2) = grays(&textured(w, h, 999), w, h);
        assert_eq!(locate(&mut a, &f2, &c2, 0.9), None);
    }
}
