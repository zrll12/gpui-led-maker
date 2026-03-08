use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use image::Rgb;

use crate::render::bitmap::{load_bdf, text_to_matrix};
use crate::render::led_image::{LedRenderOptions, render_led_matrices};
use crate::render::matrix::VerticalAlign;

const PREVIEW_TEXT: &str = "字A";
const PREVIEW_CACHE_VERSION: &str = "v2";

pub fn read_bdf_font_name(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix("FAMILY_NAME "))
        .map(|name| name.trim().trim_matches('"').to_string())
}

pub fn render_bdf_preview_image(path: &Path) -> Option<PathBuf> {
    let preview_path = preview_cache_path(path);
    let missing_marker = preview_path.with_extension("none");

    if preview_path.exists() {
        return Some(preview_path);
    }
    if missing_marker.exists() {
        return None;
    }

    let glyphs = load_bdf(path).ok()?;
    let matrix = text_to_matrix(PREVIEW_TEXT, &glyphs, Rgb([255, 80, 80]), VerticalAlign::Bottom);
    let has_pixels = matrix
        .iter()
        .any(|row| row.iter().any(|pixel| pixel.0 != [0, 0, 0]));
    if !has_pixels {
        let _ = std::fs::write(&missing_marker, b"missing");
        return None;
    }

    let image = render_led_matrices(
        &matrix,
        &LedRenderOptions {
            led_size: 8,
            spacing: 2,
            off_color: [35, 8, 8],
            bg_color: [8, 2, 2],
        },
    );
    let _ = std::fs::remove_file(&missing_marker);
    image.save(&preview_path).ok()?;

    Some(preview_path)
}

pub fn remove_bdf_preview_cache(path: &Path) {
    let preview_path = preview_cache_path(path);
    let missing_marker = preview_path.with_extension("none");
    let _ = std::fs::remove_file(preview_path);
    let _ = std::fs::remove_file(missing_marker);
}

fn preview_cache_path(path: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    PREVIEW_CACHE_VERSION.hash(&mut hasher);
    PREVIEW_TEXT.hash(&mut hasher);
    if let Ok(metadata) = std::fs::metadata(path)
        && let Ok(modified) = metadata.modified()
        && let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH)
    {
        duration.as_secs().hash(&mut hasher);
        duration.subsec_nanos().hash(&mut hasher);
    }

    let preview_dir = std::env::temp_dir().join("led-maker").join("font-previews");
    let _ = std::fs::create_dir_all(&preview_dir);
    preview_dir.join(format!("{:x}.png", hasher.finish()))
}
