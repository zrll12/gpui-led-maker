use image::Rgb;

pub type Matrix = Vec<Vec<Option<Rgb<u8>>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
	Top,
	Center,
	Bottom,
}

pub fn concat_horizontal(left: &Matrix, right: &Matrix) -> Matrix {
	concat_horizontal_aligned(left, right, VerticalAlign::Top)
}

pub fn concat_horizontal_aligned(left: &Matrix, right: &Matrix, align: VerticalAlign) -> Matrix {
	// For non-Top alignment, use content height (rows without trailing/leading blanks)
	// so that glyphs with different visual heights align correctly.
	let left_content_rows = content_rows(left, align);
	let right_content_rows = content_rows(right, align);
	let rows = left.len().max(right.len());
	let content_max = left_content_rows.max(right_content_rows);

	// Offset within the full `rows` container for each side's content.
	let left_offset = vertical_offset(content_max, left_content_rows, align)
		+ vertical_offset(rows, content_max, align);
	let right_offset = vertical_offset(content_max, right_content_rows, align)
		+ vertical_offset(rows, content_max, align);

	if matrix_debug_enabled() {
		eprintln!(
			"[matrix/concat] align={:?} left={}x{}(content_h={}) right={}x{}(content_h={}) rows={} left_offset={} right_offset={}",
			align,
			matrix_width(left),
			matrix_height(left),
			left_content_rows,
			matrix_width(right),
			matrix_height(right),
			right_content_rows,
			rows,
			left_offset,
			right_offset
		);
	}

	// Pre-compute widths so absent rows (due to vertical offset or source running out)
	// are padded with None (transparent) to keep all output rows the same total width.
	// Rows that *exist* in the source are used as-is (no intra-row padding).
	let left_w = left.iter().map(|r| r.len()).max().unwrap_or(0);
	let right_w = right.iter().map(|r| r.len()).max().unwrap_or(0);

	let mut out = Vec::with_capacity(rows);
	for y in 0..rows {
		let mut row = Vec::with_capacity(left_w + right_w);

		// Left side: use source row if present, else fill with None (transparent)
		let left_src_y = y.checked_sub(left_offset);
		match left_src_y.and_then(|sy| left.get(sy)) {
			Some(lr) => row.extend_from_slice(lr),
			None => row.resize(left_w, None),
		}

		// Right side: use source row if present, else fill with None (transparent)
		let right_src_y = y.checked_sub(right_offset);
		match right_src_y.and_then(|sy| right.get(sy)) {
			Some(rr) => row.extend_from_slice(rr),
			None => row.resize(row.len() + right_w, None),
		}

		out.push(row);
	}

	if matrix_debug_enabled() {
		eprintln!(
			"[matrix/concat] output={}x{}",
			matrix_width(&out),
			matrix_height(&out)
		);
	}

	out
}

fn vertical_offset(container_rows: usize, source_rows: usize, align: VerticalAlign) -> usize {
	if source_rows >= container_rows {
		return 0;
	}

	let extra = container_rows - source_rows;
	match align {
		VerticalAlign::Top => 0,
		VerticalAlign::Center => extra / 2,
		VerticalAlign::Bottom => extra,
	}
}

/// Returns the number of rows that actually contain non-black pixels,
/// measured from the side relevant to `align`.
/// - Bottom: count rows from top until the last non-empty row (trim trailing blanks)
/// - Top: count rows from the first non-empty row downward (trim leading blanks)
/// - Center: use the full span from first to last non-empty row
/// Falls back to the raw row count when the matrix is all-blank.
fn content_rows(matrix: &Matrix, align: VerticalAlign) -> usize {
	let is_blank = |row: &Vec<Option<Rgb<u8>>>| row.iter().all(|p| p.is_none());

	let total = matrix.len();
	if total == 0 {
		return 0;
	}

	match align {
		VerticalAlign::Bottom => {
			// Find last non-blank row; content height = that index + 1.
			let last = matrix.iter().rposition(|row| !is_blank(row));
			last.map_or(total, |i| i + 1)
		}
		VerticalAlign::Top => {
			// Find first non-blank row; content height = total - that index.
			let first = matrix.iter().position(|row| !is_blank(row));
			first.map_or(total, |i| total - i)
		}
		VerticalAlign::Center => {
			let first = matrix.iter().position(|row| !is_blank(row));
			let last = matrix.iter().rposition(|row| !is_blank(row));
			match (first, last) {
				(Some(f), Some(l)) => l - f + 1,
				_ => total,
			}
		}
	}
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

/// 将 `overlay` 叠加到 `base` 上，支持有符号偏移（可为负），超出 base 边界的部分被裁剪，base 尺寸不变。
pub fn overlay_at_clipped(base: &Matrix, overlay: &Matrix, offset_x: i32, offset_y: i32) -> Matrix {
	let mut out = base.clone();
	let base_h = out.len() as i32;
	let base_w = out.first().map_or(0, |r| r.len()) as i32;

	for (y, overlay_row) in overlay.iter().enumerate() {
		let target_y = offset_y + y as i32;
		if target_y < 0 || target_y >= base_h {
			continue;
		}
		let ty = target_y as usize;
		for (x, pixel) in overlay_row.iter().enumerate() {
			let target_x = offset_x + x as i32;
			if target_x < 0 || target_x >= base_w {
				continue;
			}
			// None 表示透明，跳过不覆盖底层
			if pixel.is_some() {
				out[ty][target_x as usize] = *pixel;
			}
		}
	}

	out
}

pub fn overlay_at(base: &Matrix, overlay: &Matrix, offset_x: usize, offset_y: usize) -> Matrix {
	let mut out = base.clone();

	for (y, overlay_row) in overlay.iter().enumerate() {
		let target_y = offset_y + y;
		if target_y >= out.len() {
			continue;
		}

		if out[target_y].len() < offset_x + overlay_row.len() {
			out[target_y].resize(offset_x + overlay_row.len(), None);
		}

		for (x, pixel) in overlay_row.iter().enumerate() {
			// None 表示透明，跳过不覆盖底层
			if pixel.is_some() {
				out[target_y][offset_x + x] = *pixel;
			}
		}
	}

	out
}

#[cfg(test)]
mod tests {
	use super::{concat_horizontal, concat_horizontal_aligned, overlay_at, Matrix, VerticalAlign};
	use image::Rgb;

	fn p(v: u8) -> Rgb<u8> {
		Rgb([v, v, v])
	}

	fn s(v: u8) -> Option<Rgb<u8>> {
		Some(Rgb([v, v, v]))
	}

	#[test]
	fn concat_horizontal_joins_rows() {
		let left: Matrix = vec![vec![s(1), s(2)], vec![s(3)]];
		let right: Matrix = vec![vec![s(4)], vec![s(5), s(6)]];

		let merged = concat_horizontal(&left, &right);

		assert_eq!(merged[0], vec![s(1), s(2), s(4)]);
		assert_eq!(merged[1], vec![s(3), s(5), s(6)]);
	}

	#[test]
	fn concat_horizontal_center_aligns_shorter_matrix() {
		let left: Matrix = vec![vec![s(1)], vec![s(2)], vec![s(3)], vec![s(4)]];
		let right: Matrix = vec![vec![s(9)], vec![s(8)]];

		let merged = concat_horizontal_aligned(&left, &right, VerticalAlign::Center);

		// right (2 rows) is centered within 4: right_offset=1.
		// Absent-side rows are padded with None (transparent) so all rows are width=2.
		assert_eq!(merged[0], vec![s(1), None]);
		assert_eq!(merged[1], vec![s(2), s(9)]);
		assert_eq!(merged[2], vec![s(3), s(8)]);
		assert_eq!(merged[3], vec![s(4), None]);
	}

	#[test]
	fn concat_horizontal_bottom_aligns_shorter_matrix() {
		let left: Matrix = vec![vec![s(1)], vec![s(2)], vec![s(3)]];
		let right: Matrix = vec![vec![s(9)]];

		let merged = concat_horizontal_aligned(&left, &right, VerticalAlign::Bottom);

		// right (1 row) sits at bottom: right_offset=2.
		// Rows 0-1 have no right content → padded with None (transparent).
		assert_eq!(merged[0], vec![s(1), None]);
		assert_eq!(merged[1], vec![s(2), None]);
		assert_eq!(merged[2], vec![s(3), s(9)]);
	}

	#[test]
	fn concat_horizontal_bottom_aligns_padded_matrices() {
		// Simulate BDF-style: both matrices are 4 rows tall,
		// but left has content in all 4 rows while right only has content in rows 0-1,
		// with rows 2-3 being blank (None/transparent).
		let left: Matrix = vec![
			vec![s(1)],
			vec![s(2)],
			vec![s(3)],
			vec![s(4)],
		];
		let right: Matrix = vec![
			vec![s(9)],   // content row 0
			vec![s(8)],   // content row 1
			vec![None],   // blank/padding
			vec![None],   // blank/padding
		];

		let merged = concat_horizontal_aligned(&left, &right, VerticalAlign::Bottom);

		// right content_rows = 2 (last non-blank row is index 1 → idx+1=2)
		// left content_rows = 4
		// content_max = 4, rows = 4
		// left_offset = v_offset(4,4,Bot)+v_offset(4,4,Bot) = 0
		// right_offset = v_offset(4,2,Bot)+v_offset(4,4,Bot) = 2+0 = 2
		// So right[0] lands at merged[2], right[1] at merged[3].
		// Rows 0-1 have no right content → padded with None (transparent).
		assert_eq!(merged[0], vec![s(1), None], "row 0");
		assert_eq!(merged[1], vec![s(2), None], "row 1");
		assert_eq!(merged[2], vec![s(3), s(9)], "row 2: left s(3) + right s(9)");
		assert_eq!(merged[3], vec![s(4), s(8)], "row 3: left s(4) + right s(8)");
		assert_eq!(merged.len(), 4);
	}

	#[test]
	fn overlay_at_replaces_pixels_with_offset() {
		let base: Matrix = vec![
			vec![None, None, None],
			vec![None, None, None],
			vec![None, None, None],
		];
		let top: Matrix = vec![vec![s(7), s(8)], vec![s(9), s(1)]];

		let out = overlay_at(&base, &top, 1, 1);

		assert_eq!(out[0], vec![None, None, None]);
		assert_eq!(out[1], vec![None, s(7), s(8)]);
		assert_eq!(out[2], vec![None, s(9), s(1)]);
	}

	#[test]
	fn overlay_at_skips_transparent_pixels() {
		let base: Matrix = vec![
			vec![s(1), s(2), s(3)],
			vec![s(4), s(5), s(6)],
		];
		// overlay has transparent pixels mixed with colored ones
		let top: Matrix = vec![vec![None, s(9), None], vec![s(7), None, s(8)]];
		let _ = p(0); // suppress unused warning

		let out = overlay_at(&base, &top, 0, 0);

		// None in overlay should NOT overwrite base pixels
		assert_eq!(out[0], vec![s(1), s(9), s(3)]);
		assert_eq!(out[1], vec![s(7), s(5), s(8)]);
	}
}