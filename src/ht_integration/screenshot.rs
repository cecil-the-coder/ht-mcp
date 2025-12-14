use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use font_kit::family_name::FamilyName;
use font_kit::properties::{Properties, Stretch, Style, Weight};
use font_kit::source::SystemSource;
use ht_core::avt;
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use std::io::Cursor;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const FONT_SIZE: f32 = 14.0;
const LINE_SPACING: f32 = 1.2;

pub struct ScreenshotRenderer {
    font_data: Vec<u8>,
    font_size: f32,
    char_width: u32,
    line_height: u32,
}

impl ScreenshotRenderer {
    pub fn new() -> Result<Self> {
        // Find a monospace font on the system
        let font_data = Self::load_system_font()?;

        // Load the font to calculate dimensions
        let font = FontRef::try_from_slice(&font_data)?;
        let scale = PxScale::from(FONT_SIZE);

        // Measure character width (use 'M' as it's typically the widest)
        let char_width = Self::measure_char_width(&font, scale);
        let line_height = (FONT_SIZE * LINE_SPACING) as u32;

        Ok(Self {
            font_data,
            font_size: FONT_SIZE,
            char_width,
            line_height,
        })
    }

    fn load_system_font() -> Result<Vec<u8>> {
        let source = SystemSource::new();

        // Try to find a monospace font
        // Priority order: system default monospace -> common monospace fonts
        let font_names = vec![
            FamilyName::Monospace,
            FamilyName::Title("DejaVu Sans Mono".to_string()),
            FamilyName::Title("Liberation Mono".to_string()),
            FamilyName::Title("Consolas".to_string()),
            FamilyName::Title("Menlo".to_string()),
            FamilyName::Title("Monaco".to_string()),
            FamilyName::Title("Courier New".to_string()),
        ];

        let properties = Properties {
            style: Style::Normal,
            weight: Weight::NORMAL,
            stretch: Stretch::NORMAL,
        };

        for family in font_names {
            if let Ok(handle) = source.select_best_match(&[family], &properties) {
                if let Ok(font) = handle.load() {
                    if let Some(data) = font.copy_font_data() {
                        return Ok(data.to_vec());
                    }
                }
            }
        }

        Err("Could not find a suitable monospace font on the system".into())
    }

    fn measure_char_width(font: &FontRef, scale: PxScale) -> u32 {
        // Measure the 'M' character as it's typically the widest in monospace fonts
        let scaled_font = font.as_scaled(scale);
        let glyph_id = font.glyph_id('M');
        let advance = scaled_font.h_advance(glyph_id);
        advance.ceil() as u32
    }

    pub fn render(&self, lines: &[String]) -> Result<Vec<u8>> {
        let font = FontRef::try_from_slice(&self.font_data)?;
        let scale = PxScale::from(self.font_size);

        // Calculate image dimensions
        let max_cols = lines.iter().map(|l| l.len()).max().unwrap_or(80);
        let rows = lines.len().max(1);

        let img_width = (max_cols as u32 * self.char_width).max(100);
        let img_height = rows as u32 * self.line_height;

        // Create image with black background
        let mut img = RgbaImage::from_pixel(
            img_width,
            img_height,
            Rgba([0, 0, 0, 255]),
        );

        // Render each line
        for (row_idx, line) in lines.iter().enumerate() {
            let y = row_idx as i32 * self.line_height as i32;

            // For now, render simple white text
            // TODO: Parse ANSI codes for colors
            draw_text_mut(
                &mut img,
                Rgba([255, 255, 255, 255]), // White text
                0,
                y,
                scale,
                &font,
                line,
            );
        }

        // Encode to PNG
        let mut png_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);

        img.write_to(&mut cursor, image::ImageFormat::Png)?;

        Ok(png_bytes)
    }

    pub fn render_with_colors(
        &self,
        lines: &[avt::Line],
    ) -> Result<Vec<u8>> {
        let font = FontRef::try_from_slice(&self.font_data)?;
        let scale = PxScale::from(self.font_size);

        // Calculate image dimensions
        let cols = lines.first().map(|l| l.len()).unwrap_or(80);
        let rows = lines.len().max(1);

        let img_width = cols as u32 * self.char_width;
        let img_height = rows as u32 * self.line_height;

        // Create image with black background (default terminal background)
        let mut img = RgbaImage::from_pixel(
            img_width,
            img_height,
            Rgba([0, 0, 0, 255]),
        );

        // Render each line with cell-by-cell coloring
        for (row_idx, line) in lines.iter().enumerate() {
            let y = row_idx as u32 * self.line_height;

            // Iterate through each cell in the line
            // cells() returns an iterator of (char, Pen) tuples
            for (col_idx, (ch, pen)) in line.cells().enumerate() {
                let x = col_idx as u32 * self.char_width;

                // Get foreground and background colors
                let fg_color = pen.foreground()
                    .map(|c| color_to_rgba(&c))
                    .unwrap_or(Rgba([255, 255, 255, 255])); // Default white

                let bg_color = pen.background()
                    .map(|c| color_to_rgba(&c))
                    .unwrap_or(Rgba([0, 0, 0, 255])); // Default black

                // Draw background rectangle for this cell
                for py in 0..self.line_height {
                    for px in 0..self.char_width {
                        let img_x = x + px;
                        let img_y = y + py;
                        if img_x < img_width && img_y < img_height {
                            img.put_pixel(img_x, img_y, bg_color);
                        }
                    }
                }

                // Draw character if not whitespace
                if !ch.is_whitespace() {
                    draw_text_mut(
                        &mut img,
                        fg_color,
                        x as i32,
                        y as i32,
                        scale,
                        &font,
                        &ch.to_string(),
                    );
                }
            }
        }

        // Encode to PNG
        let mut png_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);

        img.write_to(&mut cursor, image::ImageFormat::Png)?;

        Ok(png_bytes)
    }
}

/// Convert avt::Color to image::Rgba
fn color_to_rgba(color: &avt::Color) -> Rgba<u8> {
    match color {
        avt::Color::Indexed(idx) => indexed_to_rgba(*idx),
        avt::Color::RGB(rgb8) => {
            // RGB8 struct has r, g, b fields
            Rgba([rgb8.r, rgb8.g, rgb8.b, 255])
        }
    }
}

/// Map ANSI 256-color index to RGB
fn indexed_to_rgba(index: u8) -> Rgba<u8> {
    match index {
        // Standard colors (0-7)
        0 => Rgba([0, 0, 0, 255]),           // Black
        1 => Rgba([205, 0, 0, 255]),         // Red
        2 => Rgba([0, 205, 0, 255]),         // Green
        3 => Rgba([205, 205, 0, 255]),       // Yellow
        4 => Rgba([0, 0, 238, 255]),         // Blue
        5 => Rgba([205, 0, 205, 255]),       // Magenta
        6 => Rgba([0, 205, 205, 255]),       // Cyan
        7 => Rgba([229, 229, 229, 255]),     // White

        // Bright colors (8-15)
        8 => Rgba([127, 127, 127, 255]),     // Bright Black (Gray)
        9 => Rgba([255, 0, 0, 255]),         // Bright Red
        10 => Rgba([0, 255, 0, 255]),        // Bright Green
        11 => Rgba([255, 255, 0, 255]),      // Bright Yellow
        12 => Rgba([92, 92, 255, 255]),      // Bright Blue
        13 => Rgba([255, 0, 255, 255]),      // Bright Magenta
        14 => Rgba([0, 255, 255, 255]),      // Bright Cyan
        15 => Rgba([255, 255, 255, 255]),    // Bright White

        // 216 color cube (16-231): 6x6x6 RGB cube
        16..=231 => {
            let idx = index - 16;
            let r = idx / 36;
            let g = (idx % 36) / 6;
            let b = idx % 6;

            let to_rgb = |v: u8| -> u8 {
                if v == 0 { 0 } else { 55 + v * 40 }
            };

            Rgba([to_rgb(r), to_rgb(g), to_rgb(b), 255])
        }

        // Grayscale (232-255): 24 shades of gray
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            Rgba([gray, gray, gray, 255])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_colors_basic() {
        assert_eq!(indexed_to_rgba(0), Rgba([0, 0, 0, 255])); // Black
        assert_eq!(indexed_to_rgba(1), Rgba([205, 0, 0, 255])); // Red
        assert_eq!(indexed_to_rgba(15), Rgba([255, 255, 255, 255])); // Bright White
    }

    #[test]
    fn test_ansi_colors_grayscale() {
        let gray_start = indexed_to_rgba(232);
        let gray_end = indexed_to_rgba(255);
        assert!(gray_start[0] < gray_end[0]); // Should get darker to lighter
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = ScreenshotRenderer::new();
        assert!(renderer.is_ok(), "Should be able to create renderer with system font");
    }
}
