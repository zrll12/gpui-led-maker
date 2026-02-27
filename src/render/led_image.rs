use image::{Rgb, RgbImage};
use imageproc::drawing::draw_filled_circle_mut;

use crate::render::bitmap::{GLYPH_HEIGHT, Matrix};

pub struct LedRenderOptions {
    pub led_size: u32,
    pub spacing: u32,
    pub on_color: [u8; 3],
    pub off_color: [u8; 3],
    pub bg_color: [u8; 3],
}

impl Default for LedRenderOptions {
    fn default() -> Self {
        Self {
            led_size: 12,
            spacing: 4,
            on_color: [255, 60, 60],
            off_color: [40, 0, 0],
            bg_color: [10, 0, 0],
        }
    }
}

pub fn render_led_matrices(matrix: &Matrix, options: &LedRenderOptions) -> RgbImage {
    let total_cols = matrix.first().map_or(0, |row| row.len() as u32);

    let rows = GLYPH_HEIGHT as u32;
    let cell = options.led_size + options.spacing;
    let img_width = total_cols.max(1) * cell;
    let img_height = rows * cell;

    let mut img = RgbImage::from_pixel(img_width, img_height, Rgb(options.bg_color));

    let radius = (options.led_size / 2) as i32;

    for (y, row) in matrix.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            let color = if value == 1 {
                Rgb(options.on_color)
            } else {
                Rgb(options.off_color)
            };

            let cx = (x as u32 * cell + options.led_size / 2) as i32;
            let cy = (y as u32 * cell + options.led_size / 2) as i32;

            draw_filled_circle_mut(&mut img, (cx, cy), radius, color);
        }
    }

    img
}