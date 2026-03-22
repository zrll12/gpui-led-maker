use std::collections::HashMap;
use std::fs;
use std::path::Path;

use image::Rgb;

use crate::render::matrix::{concat_horizontal_aligned, Matrix, VerticalAlign};

pub const GLYPH_HEIGHT: usize = 16;

pub type GlyphMap = HashMap<String, String>;

pub fn load_unifont_hex<P: AsRef<Path>>(filename: P) -> std::io::Result<GlyphMap> {
    let content = fs::read_to_string(filename)?;
    let mut glyphs = HashMap::new();

    for line in content.lines() {
        if let Some((codepoint, bitmap)) = line.split_once(':') {
            glyphs.insert(codepoint.to_uppercase(), bitmap.trim().to_string());
        }
    }

    Ok(glyphs)
}

pub fn load_bdf<P: AsRef<Path>>(filename: P) -> std::io::Result<GlyphMap> {
    let content = fs::read_to_string(filename)?;
    let mut glyphs = HashMap::new();

    let mut encoding: Option<u32> = None;
    let mut bbx_width: Option<usize> = None;
    let mut in_bitmap = false;
    let mut bitmap_rows: Vec<String> = Vec::new();

    let flush_glyph = |glyphs: &mut GlyphMap,
                       encoding: Option<u32>,
                       bbx_width: Option<usize>,
                       bitmap_rows: &[String]| {
        let Some(codepoint) = encoding else {
            return;
        };
        if bitmap_rows.is_empty() {
            return;
        }

        let max_row_len = bitmap_rows.iter().map(|row| row.len()).max().unwrap_or(0);
        if max_row_len > 4 {
            return;
        }

        let target_row_hex_len = match bbx_width {
            Some(width) if width <= 8 && max_row_len <= 2 => 2,
            Some(width) if width <= 16 && max_row_len <= 4 => 4,
            Some(_) => return,
            None if max_row_len <= 2 => 2,
            None if max_row_len <= 4 => 4,
            None => return,
        };

        let mut normalized_rows = vec!["0".repeat(target_row_hex_len); GLYPH_HEIGHT];
        for (i, row) in bitmap_rows.iter().take(GLYPH_HEIGHT).enumerate() {
            normalized_rows[i] = format!("{:0>width$}", row, width = target_row_hex_len);
        }

        let bitmap = normalized_rows.join("");
        glyphs.insert(format!("{:04X}", codepoint), bitmap);
    };

    for raw_line in content.lines() {
        let line = raw_line.trim();

        if line.starts_with("STARTCHAR") {
            encoding = None;
            bbx_width = None;
            in_bitmap = false;
            bitmap_rows.clear();
            continue;
        }

        if in_bitmap {
            if line == "ENDCHAR" {
                flush_glyph(&mut glyphs, encoding, bbx_width, &bitmap_rows);
                encoding = None;
                bbx_width = None;
                in_bitmap = false;
                bitmap_rows.clear();
            } else if !line.is_empty() {
                bitmap_rows.push(line.to_uppercase());
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("ENCODING ")
            && let Ok(parsed) = value.trim().parse::<i32>()
        {
            if parsed >= 0 {
                encoding = Some(parsed as u32);
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("BBX ") {
            if let Some(width) = value.split_whitespace().next() {
                bbx_width = width.parse::<usize>().ok();
            }
            continue;
        }

        if line == "BITMAP" {
            in_bitmap = true;
        }
    }

    Ok(glyphs)
}

pub fn bitmap_to_matrix(bitmap: &str, color: Rgb<u8>) -> Option<Matrix> {
    let (width, row_hex_len) = match bitmap.len() {
        32 => (8, 2),
        64 => (16, 4),
        _ => return None,
    };

    let mut matrix = Vec::with_capacity(GLYPH_HEIGHT);

    for y in 0..GLYPH_HEIGHT {
        let start = y * row_hex_len;
        let end = start + row_hex_len;
        let row_hex = &bitmap[start..end];
        let value = u16::from_str_radix(row_hex, 16).ok()?;

        let mut row = Vec::with_capacity(width);
        for bit_index in (0..width).rev() {
            let bit = ((value >> bit_index) & 1) as u8;
            let pixel = if bit == 1 {
                Some(color)
            } else {
                None  // 透明背景
            };
            row.push(pixel);
        }
        matrix.push(row);
    }

    Some(matrix)
}

pub fn text_to_matrix(
    text: &str,
    glyphs: &GlyphMap,
    color: Rgb<u8>,
    align: VerticalAlign,
) -> Matrix {
    if matrix_debug_enabled() {
        eprintln!("[matrix/text] input=\"{}\" align={:?}", text, align);
    }

    let mut merged: Matrix = vec![];

    for ch in text.chars() {
        let codepoint = format!("{:04X}", ch as u32);
        let Some(bitmap) = glyphs.get(&codepoint) else {
            if matrix_debug_enabled() {
                eprintln!("[matrix/text] glyph missing: char='{}' codepoint={}", ch, codepoint);
            }
            continue;
        };
        let Some(matrix) = bitmap_to_matrix(bitmap, color) else {
            if matrix_debug_enabled() {
                eprintln!(
                    "[matrix/text] glyph invalid bitmap: char='{}' codepoint={} len={}",
                    ch,
                    codepoint,
                    bitmap.len()
                );
            }
            continue;
        };

        if matrix_debug_enabled() {
            eprintln!(
                "[matrix/text] glyph size: char='{}' codepoint={} size={}x{}",
                ch,
                codepoint,
                matrix_width(&matrix),
                matrix_height(&matrix)
            );
        }

        merged = concat_horizontal_aligned(&merged, &matrix, align);
    }

    if matrix_debug_enabled() {
        eprintln!(
            "[matrix/text] output size={}x{}",
            matrix_width(&merged),
            matrix_height(&merged)
        );
    }

    merged
}

fn matrix_height(matrix: &Matrix) -> usize {
    matrix.len()
}

fn matrix_width(matrix: &Matrix) -> usize {
    matrix.first().map_or(0, |row| row.len())
}

fn matrix_debug_enabled() -> bool {
    std::env::var_os("LED_MAKER_MATRIX_DEBUG").is_some()
}
