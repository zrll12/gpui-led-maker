use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use image::{Rgb, RgbImage};

use crate::modal::config::NamedPath;
use crate::modal::project::{ComponentLayer, Frame, TextEffect};
use crate::render::bitmap::{GlyphMap, load_bdf, text_to_matrix};
use crate::render::led_image::{LedRenderOptions, render_led_matrices};
use crate::render::matrix::{Matrix, VerticalAlign, overlay_at_clipped};

static FONT_CACHE: LazyLock<Mutex<HashMap<PathBuf, GlyphMap>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 渲染帧为内存图片
pub fn render_frame_to_image(frame: &Frame, font_list: &[NamedPath]) -> Option<RgbImage> {
    if frame.width == 0 || frame.height == 0 {
        return None;
    }

    // 创建固定尺寸的透明画布
    let mut canvas: Matrix = vec![vec![None; frame.width as usize]; frame.height as usize];
    let mut has_content = false;

    for layer in &frame.contents {
        if let Some((layer_matrix, offset_x, offset_y)) = render_layer(&layer.content, font_list) {
            canvas = overlay_at_clipped(
                &canvas,
                &layer_matrix,
                layer.x + offset_x,
                layer.y + offset_y,
            );
            has_content = true;
        }
    }

    if !has_content {
        return None;
    }

    Some(render_led_matrices(&canvas, &LedRenderOptions::default()))
}


fn render_layer(layer: &ComponentLayer, font_list: &[NamedPath]) -> Option<(Matrix, i32, i32)> {
    match layer {
        ComponentLayer::Text(text) => {
            if text.text.is_empty() || text.font.is_empty() {
                return None;
            }
            let font_path = PathBuf::from(&text.font);
            // 优先从 font_list 中找路径（支持按名字查找）
            let resolved_path = font_list
                .iter()
                .find(|np| np.path == font_path || np.name == text.font)
                .map(|np| np.path.clone())
                .unwrap_or(font_path);

            let glyphs = {
                let mut cache = FONT_CACHE.lock().unwrap();
                if let Some(cached) = cache.get(&resolved_path) {
                    cached.clone()
                } else {
                    let g = load_bdf(&resolved_path).ok()?;
                    cache.insert(resolved_path.clone(), g.clone());
                    g
                }
            };
            let color = Rgb([text.color.0, text.color.1, text.color.2]);
            let mut matrix = text_to_matrix(&text.text, &glyphs, color, VerticalAlign::Bottom);
            let mut offset_x = 0;
            let mut offset_y = 0;

            for effect in &text.effects {
                match effect {
                    TextEffect::Outline(outline) => {
                        if outline.width > 0 {
                            let outline_color = Rgb([outline.color.0, outline.color.1, outline.color.2]);
                            matrix = apply_outline_effect(&matrix, outline.width as usize, outline_color);
                            offset_x -= outline.width as i32;
                            offset_y -= outline.width as i32;
                        }
                    }
                }
            }

            if matrix.is_empty() {
                None
            } else {
                Some((matrix, offset_x, offset_y))
            }
        }
        ComponentLayer::Rectangle(rect) => {
            if rect.width ==
             0 || rect.height == 0 {
                return None;
            }
            let color = Rgb([rect.color.0, rect.color.1, rect.color.2]);
            let matrix: Matrix = (0..rect.height as usize)
                .map(|_| vec![Some(color); rect.width as usize])
                .collect();
            Some((matrix, 0, 0))
        }
    }
}

fn apply_outline_effect(base: &Matrix, width: usize, color: Rgb<u8>) -> Matrix {
    if base.is_empty() || width == 0 {
        return base.clone();
    }

    let base_h = base.len();
    let base_w = base.iter().map(|row| row.len()).max().unwrap_or(0);
    if base_w == 0 {
        return base.clone();
    }

    let out_h = base_h + width * 2;
    let out_w = base_w + width * 2;
    let mut out: Matrix = vec![vec![None; out_w]; out_h];

    for (y, row) in base.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            if pixel.is_none() {
                continue;
            }

            let cx = x + width;
            let cy = y + width;

            for dy in -(width as isize)..=(width as isize) {
                for dx in -(width as isize)..=(width as isize) {
                    if dx.abs().max(dy.abs()) > width as isize {
                        continue;
                    }

                    let ny = cy as isize + dy;
                    let nx = cx as isize + dx;
                    if ny >= 0 && ny < out_h as isize && nx >= 0 && nx < out_w as isize {
                        out[ny as usize][nx as usize] = Some(color);
                    }
                }
            }
        }
    }

    for (y, row) in base.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            if pixel.is_some() {
                out[y + width][x + width] = *pixel;
            }
        }
    }

    out
}


