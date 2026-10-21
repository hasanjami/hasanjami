//! Replay an Anfield match on a pitch-colored board.

pub const WIDTH: usize = 960;
pub const HEIGHT: usize = 640;

const NIGHT: u32 = 0x0c1c13;
const STRIPE_A: u32 = 0x143d24;
const STRIPE_B: u32 = 0x102e1c;
const GOLD: u32 = 0xf0c14a;
const GOLD_NEW: u32 = 0xffe7a3;
const ICE: u32 = 0x7ec8e3;
const ICE_NEW: u32 = 0xd7f6ff;
const LINE: u32 = 0x214d32;
const INK: u32 = 0xe7f2ea;

/// One Anfield snapshot. Scores are the cells each player already owns.
#[derive(Clone, Debug)]
pub struct Frame {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<u8>,
    pub p1: i32,
    pub p2: i32,
}

/// Boards in the order the engine printed them.
pub fn parse_frames(text: &str) -> Result<Vec<Frame>, String> {
    let mut frames = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let header = line.trim();
        if !header.starts_with("Anfield") {
            continue;
        }
        let (width, height) = parse_size(header)?;
        let mut cells = Vec::with_capacity((width * height) as usize);
        let mut rows = 0;
        while rows < height {
            let row = match lines.next() {
                Some(row) => row,
                None => break,
            };
            if row.trim().is_empty() || is_ruler(row) {
                continue;
            }
            let grid = match split_row(row) {
                Some(grid) => grid,
                None => return Err(format!("bad anfield row {}", row)),
            };
            if grid.len() != width as usize {
                return Err(format!("anfield row has {} cells, want {}", grid.len(), width));
            }
            cells.extend(grid.bytes());
            rows += 1;
        }
        if rows != height {
            return Err(format!("anfield ended after {} of {} rows", rows, height));
        }
        let mut p1 = 0;
        let mut p2 = 0;
        for cell in &cells {
            match *cell {
                b'@' | b'a' => p1 += 1,
                b'$' | b's' => p2 += 1,
                _ => {}
            }
        }
        frames.push(Frame {
            width,
            height,
            cells,
            p1,
            p2,
        });
    }
    if frames.is_empty() {
        return Err("the log has no Anfield".to_string());
    }
    Ok(frames)
}

/// Draw one frame. `index` is zero-based.
pub fn render(frame: &Frame, index: usize, total: usize) -> Vec<u32> {
    let mut pixels = vec![NIGHT; WIDTH * HEIGHT];
    fill_rect(&mut pixels, 0, 0, WIDTH, 4, GOLD);
    draw_text(&mut pixels, 28, 22, "ANFIELD", 3, INK);
    draw_text(
        &mut pixels,
        220,
        24,
        &format!("{}", frame.p1),
        3,
        GOLD_NEW,
    );
    draw_text(
        &mut pixels,
        320,
        24,
        &format!("{}", frame.p2),
        3,
        ICE_NEW,
    );
    draw_text(
        &mut pixels,
        720,
        24,
        &format!("{}/{}", index + 1, total.max(1)),
        2,
        INK,
    );

    let (origin_x, origin_y, cell) = layout(frame);
    for row in 0..frame.height {
        for col in 0..frame.width {
            let x = origin_x + col as usize * cell;
            let y = origin_y + row as usize * cell;
            let stripe = if row % 2 == 0 { STRIPE_A } else { STRIPE_B };
            fill_rect(&mut pixels, x, y, cell, cell, stripe);
            if cell > 2 {
                fill_rect(&mut pixels, x, y, cell, 1, LINE);
                fill_rect(&mut pixels, x, y, 1, cell, LINE);
            }
            let value = frame.cells[(row * frame.width + col) as usize];
            let color = match value {
                b'@' => GOLD,
                b'a' => GOLD_NEW,
                b'$' => ICE,
                b's' => ICE_NEW,
                _ => continue,
            };
            let inset = (cell / 6).max(1);
            if cell > inset * 2 {
                fill_rect(
                    &mut pixels,
                    x + inset,
                    y + inset,
                    cell - inset * 2,
                    cell - inset * 2,
                    color,
                );
            }
        }
    }

    let mut dot_x = 28;
    let dots = total.min(48);
    for dot in 0..dots {
        let color = if total <= 48 && dot == index {
            GOLD
        } else if total > 48 && dot * total / 48 == index {
            GOLD
        } else {
            LINE
        };
        fill_rect(&mut pixels, dot_x, HEIGHT - 32, 10, 10, color);
        dot_x += 16;
    }
    pixels
}

/// Top-left pixel of a cell, and the cell size in pixels.
pub fn layout(frame: &Frame) -> (usize, usize, usize) {
    let max_w = WIDTH - 64;
    let max_h = HEIGHT - 120;
    let cell = (max_w / frame.width.max(1) as usize).min(max_h / frame.height.max(1) as usize);
    let cell = cell.max(1);
    let board_w = cell * frame.width as usize;
    let board_h = cell * frame.height as usize;
    let origin_x = (WIDTH - board_w) / 2;
    let origin_y = 72 + (max_h - board_h) / 2;
    (origin_x, origin_y, cell)
}

pub fn to_ppm(pixels: &[u32], width: usize, height: usize) -> Vec<u8> {
    let mut bytes = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    bytes.reserve(width * height * 3);
    for pixel in pixels {
        bytes.push(((pixel >> 16) & 0xff) as u8);
        bytes.push(((pixel >> 8) & 0xff) as u8);
        bytes.push((pixel & 0xff) as u8);
    }
    bytes
}

/// A four-frame match used when the visualizer is opened with no log.
pub fn sample_log() -> &'static str {
    SAMPLE
}

const SAMPLE: &str = "\
$$$ exec p1 : [visualizer]
Anfield 16 10:
    0123456789012345
000 ................
001 ................
002 ........@.......
003 ................
004 ................
005 ................
006 ................
007 ........$.......
008 ................
009 ................
Piece 2 1:
OO
-> Answer (@): 7 2
Anfield 16 10:
    0123456789012345
000 ................
001 ................
002 .......a@.......
003 ................
004 ................
005 ................
006 ................
007 ........$.......
008 ................
009 ................
Piece 2 1:
OO
-> Answer (@): 6 2
Anfield 16 10:
    0123456789012345
000 ................
001 ................
002 ......aa@.......
003 ................
004 ................
005 ................
006 ........s.......
007 ........$.......
008 ................
009 ................
Piece 1 2:
O
O
-> Answer (@): 8 1
Anfield 16 10:
    0123456789012345
000 ................
001 ........a.......
002 ......aa@.......
003 ................
004 ................
005 ................
006 ........s.......
007 ........$.......
008 ................
009 ................
Player1 (visualizer): 4
Player2 (robot): 2
Player1 won!
";

fn parse_size(header: &str) -> Result<(i32, i32), String> {
    let mut parts = header.trim().trim_end_matches(':').split_whitespace();
    let _name = parts.next();
    let width = match parts.next().and_then(|text| text.parse::<i32>().ok()) {
        Some(value) => value,
        None => return Err(format!("bad anfield header {}", header)),
    };
    let height = match parts.next().and_then(|text| text.parse::<i32>().ok()) {
        Some(value) => value,
        None => return Err(format!("bad anfield header {}", header)),
    };
    Ok((width, height))
}

fn is_ruler(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && trimmed.bytes().all(|byte| byte.is_ascii_digit())
}

fn split_row(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    if bytes.first().map(|byte| byte.is_ascii_digit()) != Some(true) {
        return None;
    }
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index >= bytes.len() || bytes[index] != b' ' {
        return None;
    }
    while index < bytes.len() && bytes[index] == b' ' {
        index += 1;
    }
    Some(&line[index..])
}

fn fill_rect(pixels: &mut [u32], x: usize, y: usize, width: usize, height: usize, color: u32) {
    for row in y..(y + height).min(HEIGHT) {
        let start = row * WIDTH + x;
        let end = row * WIDTH + (x + width).min(WIDTH);
        if start >= pixels.len() || end > pixels.len() || start >= end {
            continue;
        }
        for pixel in &mut pixels[start..end] {
            *pixel = color;
        }
    }
}

fn draw_text(pixels: &mut [u32], x: usize, y: usize, text: &str, scale: usize, color: u32) {
    let mut cursor = x;
    for ch in text.chars() {
        let glyph = glyph(ch);
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) == 0 {
                    continue;
                }
                fill_rect(
                    pixels,
                    cursor + col * scale,
                    y + row * scale,
                    scale,
                    scale,
                    color,
                );
            }
        }
        cursor += 4 * scale;
    }
}

fn glyph(ch: char) -> [u8; 5] {
    match ch {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b001, 0b001, 0b001],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'N' => [0b101, 0b111, 0b111, 0b101, 0b101],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        _ => [0, 0, 0, 0, 0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_has_four_frames_and_growing_score() {
        let frames = parse_frames(sample_log()).unwrap();
        assert_eq!(frames.len(), 4);
        assert_eq!((frames[0].p1, frames[0].p2), (1, 1));
        assert_eq!(frames[1].p1, 2);
        assert_eq!(frames[3].p1, 4);
        assert!(sample_log().contains("Player1 won!"));
    }

    #[test]
    fn render_paints_gold_and_ice() {
        let frames = parse_frames(sample_log()).unwrap();
        let pixels = render(&frames[0], 0, frames.len());
        assert_eq!(pixels.len(), WIDTH * HEIGHT);
        let (x, y, cell) = layout(&frames[0]);
        let gold = pixels[(y + 2 * cell + cell / 2) * WIDTH + (x + 8 * cell + cell / 2)];
        let ice = pixels[(y + 7 * cell + cell / 2) * WIDTH + (x + 8 * cell + cell / 2)];
        assert_eq!(gold, GOLD);
        assert_eq!(ice, ICE);
        assert_eq!(pixels[10 * WIDTH + 10], NIGHT);
    }

    #[test]
    fn recorded_log_parses_when_present() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/matches/map00-wall_e.txt");
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(_) => return,
        };
        let frames = parse_frames(&text).unwrap();
        assert!(frames.len() > 4, "frames {}", frames.len());
        assert!(frames.last().unwrap().p1 > frames.last().unwrap().p2);
    }
}
