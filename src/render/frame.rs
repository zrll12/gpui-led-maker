use std::path::PathBuf;

use image::{Rgb, RgbImage};

use crate::modal::config::NamedPath;
use crate::modal::project::{ComponentLayer, Frame};
use crate::render::bitmap::{load_bdf, text_to_matrix};
use crate::render::led_image::{LedRenderOptions, render_led_matrices};
use crate::render::matrix::{Matrix, VerticalAlign, overlay_at_clipped};

/// 渲染帧为内存图片
pub fn render_frame_to_image(frame: &Frame, font_list: &[NamedPath]) -> Option<RgbImage> {
    if frame.width == 0 || frame.height == 0 {
        return None;
    }

    // 创建固定尺寸的黑色画布
    let black = Rgb([0u8, 0, 0]);
    let mut canvas: Matrix = vec![vec![black; frame.width as usize]; frame.height as usize];
    let mut has_content = false;

    for layer in &frame.contents {
        if let Some(layer_matrix) = render_layer(&layer.content, font_list) {
            canvas = overlay_at_clipped(&canvas, &layer_matrix, layer.x, layer.y);
            has_content = true;
        }
    }

    if !has_content {
        return None;
    }

    Some(render_led_matrices(&canvas, &LedRenderOptions::default()))
}


fn render_layer(layer: &ComponentLayer, font_list: &[NamedPath]) -> Option<Matrix> {
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

            let glyphs = load_bdf(&resolved_path).ok()?;
            let color = Rgb([text.color.0, text.color.1, text.color.2]);
            let matrix = text_to_matrix(&text.text, &glyphs, color, VerticalAlign::Bottom);
            if matrix.is_empty() {
                None
            } else {
                Some(matrix)
            }
        }
        ComponentLayer::Rectangle(rect) => {
            if rect.width ==
             0 || rect.height == 0 {
                return None;
            }
            let color = Rgb([rect.color.0, rect.color.1, rect.color.2]);
            let matrix: Matrix = (0..rect.height as usize)
                .map(|_| vec![color; rect.width as usize])
                .collect();
            Some(matrix)
        }
    }
}


