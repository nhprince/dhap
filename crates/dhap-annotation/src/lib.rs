use dhap_core::{Annotation, AnnotationKind, DhapError, DhapResult, Redaction, RedactionStyle};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

pub struct AnnotationRenderer;

impl AnnotationRenderer {
    /// Renders vector annotations and irreversible redactions into an image buffer.
    /// Redactions are destructively written over the pixels so they cannot be retrieved.
    pub fn process_image_bytes(
        input_bytes: &[u8],
        annotations: &[Annotation],
        redactions: &[Redaction],
    ) -> DhapResult<Vec<u8>> {
        let img = image::load_from_memory(input_bytes)
            .map_err(|e| DhapError::AnnotationError(format!("Failed to decode image: {}", e)))?;
        let mut rgba_img = img.to_rgba8();

        // 1. Bake irreversible redactions first (so sensitive data is permanently erased)
        Self::bake_redactions(&mut rgba_img, redactions)?;

        // 2. Bake vector annotations on top
        Self::bake_annotations(&mut rgba_img, annotations)?;

        // 3. Encode back to PNG
        let mut output_bytes = Vec::new();
        DynamicImage::ImageRgba8(rgba_img)
            .write_to(&mut Cursor::new(&mut output_bytes), ImageFormat::Png)
            .map_err(|e| DhapError::AnnotationError(format!("Failed to encode image: {}", e)))?;

        Ok(output_bytes)
    }

    /// Destructively alters pixels within the redaction bounding boxes.
    pub fn bake_redactions(img: &mut RgbaImage, redactions: &[Redaction]) -> DhapResult<()> {
        let (img_w, img_h) = img.dimensions();

        for redaction in redactions {
            let r = &redaction.rect;
            let start_x = r.x.min(img_w);
            let start_y = r.y.min(img_h);
            let end_x = (r.x + r.width).min(img_w);
            let end_y = (r.y + r.height).min(img_h);

            if start_x >= end_x || start_y >= end_y {
                continue;
            }

            match &redaction.style {
                RedactionStyle::Solid { color_hex } => {
                    let color = Self::parse_hex_color(color_hex).unwrap_or(Rgba([0, 0, 0, 255]));
                    for y in start_y..end_y {
                        for x in start_x..end_x {
                            img.put_pixel(x, y, color);
                        }
                    }
                }
                RedactionStyle::Pixelate { block_size } => {
                    let b = (*block_size).max(4);
                    let mut y = start_y;
                    while y < end_y {
                        let block_h = b.min(end_y - y);
                        let mut x = start_x;
                        while x < end_x {
                            let block_w = b.min(end_x - x);

                            // Calculate average color in this block
                            let mut r_sum: u64 = 0;
                            let mut g_sum: u64 = 0;
                            let mut b_sum: u64 = 0;
                            let count = (block_w * block_h) as u64;

                            for py in y..(y + block_h) {
                                for px in x..(x + block_w) {
                                    let pixel = img.get_pixel(px, py);
                                    r_sum += pixel[0] as u64;
                                    g_sum += pixel[1] as u64;
                                    b_sum += pixel[2] as u64;
                                }
                            }

                            let avg_color = Rgba([
                                (r_sum / count) as u8,
                                (g_sum / count) as u8,
                                (b_sum / count) as u8,
                                255,
                            ]);

                            // Overwrite all pixels in block with average
                            for py in y..(y + block_h) {
                                for px in x..(x + block_w) {
                                    img.put_pixel(px, py, avg_color);
                                }
                            }

                            x += block_w;
                        }
                        y += block_h;
                    }
                }
                RedactionStyle::Blur { radius: _ } => {
                    // Fast box blur fallback for security
                    let avg_color = Rgba([40, 40, 40, 255]);
                    for y in start_y..end_y {
                        for x in start_x..end_x {
                            img.put_pixel(x, y, avg_color);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Renders vector annotations (callout circles, bounding boxes, highlights)
    pub fn bake_annotations(img: &mut RgbaImage, annotations: &[Annotation]) -> DhapResult<()> {
        let (img_w, img_h) = img.dimensions();

        for anno in annotations {
            let r = &anno.rect;
            let color = Self::parse_hex_color(&anno.color_hex).unwrap_or(Rgba([230, 80, 0, 255]));

            match &anno.kind {
                AnnotationKind::CalloutBadge { number: _ } => {
                    // Draw a solid circle / badge at center of rect
                    let cx = (r.x + r.width / 2).min(img_w);
                    let cy = (r.y + r.height / 2).min(img_h);
                    let radius = (r.width.min(r.height) / 2).max(12);

                    let min_x = cx.saturating_sub(radius);
                    let max_x = (cx + radius).min(img_w);
                    let min_y = cy.saturating_sub(radius);
                    let max_y = (cy + radius).min(img_h);

                    for y in min_y..max_y {
                        for x in min_x..max_x {
                            let dx = (x as i64) - (cx as i64);
                            let dy = (y as i64) - (cy as i64);
                            if (dx * dx + dy * dy) <= (radius as i64 * radius as i64) {
                                img.put_pixel(x, y, color);
                            }
                        }
                    }
                }
                AnnotationKind::Rectangle => {
                    // Draw 3px stroked border
                    let thickness = 3u32;
                    let x1 = r.x.min(img_w);
                    let y1 = r.y.min(img_h);
                    let x2 = (r.x + r.width).min(img_w);
                    let y2 = (r.y + r.height).min(img_h);

                    for y in y1..y2 {
                        for x in x1..x2 {
                            let on_border = (x < x1 + thickness)
                                || (x >= x2.saturating_sub(thickness))
                                || (y < y1 + thickness)
                                || (y >= y2.saturating_sub(thickness));
                            if on_border {
                                img.put_pixel(x, y, color);
                            }
                        }
                    }
                }
                AnnotationKind::Highlight => {
                    // 50% opacity blend rectangle
                    let x1 = r.x.min(img_w);
                    let y1 = r.y.min(img_h);
                    let x2 = (r.x + r.width).min(img_w);
                    let y2 = (r.y + r.height).min(img_h);

                    for y in y1..y2 {
                        for x in x1..x2 {
                            let orig = img.get_pixel(x, y);
                            let blended = Rgba([
                                ((orig[0] as u16 + color[0] as u16) / 2) as u8,
                                ((orig[1] as u16 + color[1] as u16) / 2) as u8,
                                ((orig[2] as u16 + color[2] as u16) / 2) as u8,
                                255,
                            ]);
                            img.put_pixel(x, y, blended);
                        }
                    }
                }
                AnnotationKind::Arrow { .. } | AnnotationKind::Text { .. } => {
                    // Fallback marker
                    let x1 = r.x.min(img_w);
                    let y1 = r.y.min(img_h);
                    let x2 = (r.x + 8).min(img_w);
                    let y2 = (r.y + 8).min(img_h);
                    for y in y1..y2 {
                        for x in x1..x2 {
                            img.put_pixel(x, y, color);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_hex_color(hex: &str) -> Option<Rgba<u8>> {
        let clean = hex.trim_start_matches('#');
        if clean.len() == 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
            let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
            let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
            Some(Rgba([r, g, b, 255]))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dhap_core::Rect;

    #[test]
    fn test_destructive_solid_redaction() {
        // Create 100x100 white image
        let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));

        let redaction = Redaction {
            id: "red_1".to_string(),
            rect: Rect {
                x: 10,
                y: 10,
                width: 20,
                height: 20,
            },
            style: RedactionStyle::Solid {
                color_hex: "#000000".to_string(),
            },
        };

        AnnotationRenderer::bake_redactions(&mut img, &[redaction]).unwrap();

        // Check that pixels inside the box are 100% black
        for y in 10..30 {
            for x in 10..30 {
                let p = img.get_pixel(x, y);
                assert_eq!(p[0], 0);
                assert_eq!(p[1], 0);
                assert_eq!(p[2], 0);
                assert_eq!(p[3], 255);
            }
        }

        // Check that pixels outside the box remain white
        let p_out = img.get_pixel(5, 5);
        assert_eq!(p_out[0], 255);
        assert_eq!(p_out[1], 255);
        assert_eq!(p_out[2], 255);
    }
}
