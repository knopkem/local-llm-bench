use super::framebuffer::Renderer;

/// 3x5 bitmap font (period style). Each glyph: 5 rows of 3-bit values,
/// bit2 = leftmost pixel.
fn glyph(c: u8) -> Option<[u8; 5]> {
    Some(match c {
        b'A' => [2, 5, 7, 5, 5],
        b'B' => [6, 5, 6, 5, 6],
        b'C' => [3, 4, 4, 4, 3],
        b'D' => [6, 5, 5, 5, 6],
        b'E' => [7, 4, 6, 4, 7],
        b'F' => [7, 4, 6, 4, 4],
        b'G' => [3, 4, 5, 5, 3],
        b'H' => [5, 5, 7, 5, 5],
        b'I' => [7, 2, 2, 2, 7],
        b'J' => [1, 1, 1, 5, 2],
        b'K' => [5, 5, 6, 5, 5],
        b'L' => [4, 4, 4, 4, 7],
        b'M' => [5, 7, 7, 5, 5],
        b'N' => [5, 7, 5, 5, 5],
        b'O' => [2, 5, 5, 5, 2],
        b'P' => [6, 5, 6, 4, 4],
        b'Q' => [2, 5, 5, 6, 3],
        b'R' => [6, 5, 6, 5, 5],
        b'S' => [3, 4, 2, 1, 6],
        b'T' => [7, 2, 2, 2, 2],
        b'U' => [5, 5, 5, 5, 3],
        b'V' => [5, 5, 5, 5, 2],
        b'W' => [5, 5, 5, 7, 5],
        b'X' => [5, 5, 2, 5, 5],
        b'Y' => [5, 5, 2, 2, 2],
        b'Z' => [7, 1, 2, 4, 7],
        b'0' => [2, 5, 5, 5, 2],
        b'1' => [2, 6, 2, 2, 7],
        b'2' => [3, 1, 2, 4, 7],
        b'3' => [3, 1, 3, 1, 3],
        b'4' => [5, 5, 7, 1, 1],
        b'5' => [7, 4, 6, 1, 6],
        b'6' => [3, 4, 6, 5, 2],
        b'7' => [7, 1, 1, 2, 2],
        b'8' => [2, 5, 7, 5, 2],
        b'9' => [2, 5, 5, 1, 3],
        b' ' => [0, 0, 0, 0, 0],
        b'.' => [0, 0, 0, 0, 2],
        b',' => [0, 0, 0, 2, 4],
        b':' => [0, 2, 0, 2, 0],
        b'/' => [1, 1, 2, 4, 4],
        b'-' => [0, 0, 7, 0, 0],
        b'+' => [0, 2, 7, 2, 0],
        b'!' => [2, 2, 2, 0, 2],
        b'?' => [3, 1, 2, 0, 2],
        b'\'' => [2, 2, 0, 0, 0],
        b'>' => [4, 2, 1, 2, 4],
        b'<' => [1, 2, 4, 2, 1],
        b'=' => [0, 7, 0, 7, 0],
        b'%' => [5, 1, 2, 4, 5],
        _ => return None,
    })
}

pub fn text_width(s: &str, scale: i32) -> i32 {
    s.len() as i32 * 4 * scale - scale
}

/// Draw text at (x, y) with the given palette color and integer scale.
pub fn draw_text(r: &mut Renderer, x: i32, y: i32, s: &str, c: u8, scale: i32) {
    let mut cx = x;
    for ch in s.chars() {
        if ch as u64 >= 128 {
            cx += 4 * scale;
            continue;
        }
        if let Some(g) = glyph(ch as u8) {
            for (row, bits) in g.iter().enumerate() {
                for col in 0..3 {
                    if (bits >> (2 - col)) & 1 == 1 {
                        r.rect(
                            cx + col * scale,
                            y + row as i32 * scale,
                            cx + col * scale + scale - 1,
                            y + row as i32 * scale + scale - 1,
                            c,
                        );
                    }
                }
            }
        }
        cx += 4 * scale;
    }
}
