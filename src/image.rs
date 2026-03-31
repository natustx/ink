use crate::layout::{SpanStyle, StyledLine, StyledSpan};
use std::path::Path;

/// Maximum image dimensions in character cells.
const MAX_IMAGE_WIDTH: u32 = 60;
const MAX_IMAGE_HEIGHT: u32 = 24; // Each row = 2 pixel rows with half-blocks

/// Load image data from a file path or URL.
///
/// For relative paths, resolves against `base_dir`.
/// Returns `None` if the image cannot be loaded (missing file, network error, etc.).
pub fn load_image(src: &str, base_dir: Option<&Path>) -> Option<Vec<u8>> {
    if src.starts_with("http://") || src.starts_with("https://") {
        // Download with a short timeout to avoid blocking the UI
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;
        client
            .get(src)
            .send()
            .ok()?
            .bytes()
            .ok()
            .map(|b| b.to_vec())
    } else {
        let path = if let Some(base) = base_dir {
            base.join(src)
        } else {
            std::path::PathBuf::from(src)
        };
        std::fs::read(&path).ok()
    }
}

/// Render image data to styled lines using Unicode half-block characters (▄).
///
/// This rendering method works in **any** terminal that supports 24-bit (true) color.
/// Each character cell represents 2 vertical pixels: the top pixel uses the background
/// color and the bottom pixel uses the foreground color of the `▄` character.
///
/// Returns `None` if the image cannot be decoded or has zero dimensions.
pub fn render_halfblock(data: &[u8], max_width: usize, margin: usize) -> Option<Vec<StyledLine>> {
    use image::GenericImageView;

    let img = image::load_from_memory(data).ok()?;
    let (w, h) = img.dimensions();

    if w == 0 || h == 0 {
        return None;
    }

    // Calculate target dimensions that fit within constraints
    let target_w = (max_width as u32).min(MAX_IMAGE_WIDTH).min(w);
    let scale = target_w as f64 / w as f64;
    let mut target_h = ((h as f64 * scale) as u32).min(MAX_IMAGE_HEIGHT * 2);

    // Make height even for half-block pairing
    if target_h % 2 == 1 {
        target_h += 1;
    }
    if target_h == 0 {
        target_h = 2;
    }

    let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Triangle);

    let margin_str = " ".repeat(margin);
    let mut lines = Vec::new();

    // Blank line before image
    lines.push(StyledLine::empty());

    for y in (0..target_h).step_by(2) {
        let mut line = StyledLine::new();

        if margin > 0 {
            line.push(StyledSpan {
                text: margin_str.clone(),
                style: SpanStyle::default(),
            });
        }

        for x in 0..target_w {
            let top = resized.get_pixel(x, y);
            let bottom = if y + 1 < target_h {
                resized.get_pixel(x, y + 1)
            } else {
                image::Rgba([0, 0, 0, 255])
            };

            line.push(StyledSpan {
                text: "▄".to_string(),
                style: SpanStyle {
                    fg: Some(format!(
                        "#{:02x}{:02x}{:02x}",
                        bottom[0], bottom[1], bottom[2]
                    )),
                    bg: Some(format!("#{:02x}{:02x}{:02x}", top[0], top[1], top[2])),
                    ..Default::default()
                },
            });
        }

        lines.push(line);
    }

    // Blank line after image
    lines.push(StyledLine::empty());

    Some(lines)
}
