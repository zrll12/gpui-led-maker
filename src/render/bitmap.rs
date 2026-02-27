use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const GLYPH_HEIGHT: usize = 16;

pub type Matrix = Vec<Vec<u8>>;
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

pub fn bitmap_to_matrix(bitmap: &str) -> Option<Matrix> {
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
			row.push(bit);
		}
		matrix.push(row);
	}

	Some(matrix)
}

pub fn text_to_matrix(text: &str, glyphs: &GlyphMap) -> Matrix {
	let mut merged = vec![Vec::new(); GLYPH_HEIGHT];

	for ch in text.chars() {
		let codepoint = format!("{:04X}", ch as u32);
		let Some(bitmap) = glyphs.get(&codepoint) else {
			continue;
		};
		let Some(matrix) = bitmap_to_matrix(bitmap) else {
			continue;
		};

		for y in 0..GLYPH_HEIGHT {
			merged[y].extend_from_slice(&matrix[y]);
		}
	}

	merged
}
