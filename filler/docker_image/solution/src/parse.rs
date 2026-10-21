//! Read the game engine's Anfield protocol.

use std::io::{self, BufRead};

/// A cell inside a piece, relative to the piece's top-left corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// One board snapshot. `cells` is indexed by row, then column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anfield {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<Vec<u8>>,
}

/// The shape to place. `blocks` lists every occupied cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<Vec<u8>>,
    pub blocks: Vec<Point>,
}

/// Who we are, the board, and the piece for this turn.
#[derive(Clone, Debug)]
pub struct Turn {
    pub player: i32,
    pub field: Anfield,
    pub piece: Piece,
}

/// Walks engine messages until each piece has been read.
pub struct Stream<R> {
    reader: R,
    player: i32,
    field: Option<Anfield>,
}

impl<R: BufRead> Stream<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            player: 0,
            field: None,
        }
    }

    /// The next piece, together with the latest board.
    /// `Ok(None)` means the engine closed the stream between turns.
    pub fn next_turn(&mut self) -> Result<Option<Turn>, String> {
        loop {
            let line = match read_line(&mut self.reader) {
                Ok(Some(line)) => line,
                Ok(None) => return Ok(None),
                Err(err) => return Err(err.to_string()),
            };
            if line.is_empty() {
                continue;
            }
            if line.starts_with("$$$") {
                self.player = parse_player(&line)?;
            } else if line.starts_with("Anfield") {
                self.field = Some(parse_anfield(&mut self.reader, &line)?);
            } else if line.starts_with("Piece") {
                let piece = parse_piece(&mut self.reader, &line)?;
                let field = self
                    .field
                    .clone()
                    .ok_or_else(|| "piece arrived before the anfield".to_string())?;
                if self.player == 0 {
                    return Err("piece arrived before the player".to_string());
                }
                return Ok(Some(Turn {
                    player: self.player,
                    field,
                    piece,
                }));
            } else {
                return Err(format!("unexpected line {}", line));
            }
        }
    }
}

/// Read the opening `$$$ exec pN` line.
pub fn parse_player(line: &str) -> Result<i32, String> {
    if line.contains("exec p1") {
        Ok(1)
    } else if line.contains("exec p2") {
        Ok(2)
    } else {
        Err(format!("bad player line {}", line))
    }
}

/// Read the board that follows an `Anfield W H:` header.
pub fn parse_anfield<R: BufRead>(reader: &mut R, header: &str) -> Result<Anfield, String> {
    let (width, height) = parse_size(header, "anfield")?;
    let mut cells = Vec::with_capacity(height as usize);
    while cells.len() < height as usize {
        let line = match read_line(reader).map_err(|err| err.to_string())? {
            Some(line) => line,
            None => break,
        };
        if line.is_empty() || is_ruler(&line) {
            continue;
        }
        let (_, row) = split_row(&line).ok_or_else(|| format!("bad anfield row {}", line))?;
        if row.len() != width as usize {
            return Err(format!(
                "anfield row has {} cells, want {}",
                row.len(),
                width
            ));
        }
        cells.push(row.as_bytes().to_vec());
    }
    if cells.len() != height as usize {
        return Err(format!(
            "anfield ended after {} of {} rows",
            cells.len(),
            height
        ));
    }
    Ok(Anfield {
        width,
        height,
        cells,
    })
}

/// Read the shape that follows a `Piece W H:` header.
pub fn parse_piece<R: BufRead>(reader: &mut R, header: &str) -> Result<Piece, String> {
    let (width, height) = parse_size(header, "piece")?;
    let mut rows = Vec::with_capacity(height as usize);
    while rows.len() < height as usize {
        let line = match read_line(reader).map_err(|err| err.to_string())? {
            Some(line) => line,
            None => break,
        };
        if line.is_empty() {
            return Err("empty piece row".to_string());
        }
        if line.len() != width as usize {
            return Err(format!("piece row has {} cells, want {}", line.len(), width));
        }
        rows.push(line.into_bytes());
    }
    if rows.len() != height as usize {
        return Err(format!(
            "piece ended after {} of {} rows",
            rows.len(),
            height
        ));
    }
    Ok(new_piece(width, height, rows))
}

/// Player 1 owns `a` and `@`. Player 2 owns `s` and `$`.
pub fn is_mine(player: i32, cell: u8) -> bool {
    match player {
        1 => cell == b'@' || cell == b'a',
        2 => cell == b'$' || cell == b's',
        _ => false,
    }
}

/// The cell belongs to the opponent.
pub fn is_enemy(player: i32, cell: u8) -> bool {
    match player {
        1 => is_mine(2, cell),
        2 => is_mine(1, cell),
        _ => false,
    }
}

fn new_piece(width: i32, height: i32, rows: Vec<Vec<u8>>) -> Piece {
    let mut blocks = Vec::new();
    for (y, row) in rows.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if is_block(*cell) {
                blocks.push(Point {
                    x: x as i32,
                    y: y as i32,
                });
            }
        }
    }
    Piece {
        width,
        height,
        cells: rows,
        blocks,
    }
}

fn is_block(cell: u8) -> bool {
    matches!(cell, b'O' | b'o' | b'#' | b'*')
}

fn parse_size(header: &str, label: &str) -> Result<(i32, i32), String> {
    let header = header.trim().trim_end_matches(':');
    let mut parts = header.split_whitespace();
    let _name = parts.next();
    let width = parts
        .next()
        .ok_or_else(|| format!("bad {} header {}", label, header))?
        .parse::<i32>()
        .map_err(|_| format!("bad {} header {}", label, header))?;
    let height = parts
        .next()
        .ok_or_else(|| format!("bad {} header {}", label, header))?
        .parse::<i32>()
        .map_err(|_| format!("bad {} header {}", label, header))?;
    if width < 0 || height < 0 {
        return Err(format!("negative {} size", label));
    }
    Ok((width, height))
}

fn is_ruler(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && trimmed.bytes().all(|byte| byte.is_ascii_digit())
}

fn split_row(line: &str) -> Option<(&str, &str)> {
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
    let label_end = index;
    while index < bytes.len() && bytes[index] == b' ' {
        index += 1;
    }
    Some((&line[..label_end], &line[index..]))
}

/// One engine answer, paired with the board and piece it was played against.
pub struct LoggedMove {
    pub field: Anfield,
    pub piece: Piece,
    pub player: i32,
    pub x: i32,
    pub y: i32,
}

/// Pull `-> Answer` lines out of a verbose engine log.
pub fn logged_moves(text: &str) -> Result<Vec<LoggedMove>, String> {
    let mut reader = std::io::Cursor::new(text);
    let mut field = None;
    let mut piece = None;
    let mut moves = Vec::new();
    loop {
        let line = match read_line(&mut reader).map_err(|err| err.to_string())? {
            Some(line) => line,
            None => break,
        };
        if line.starts_with("Anfield") {
            field = Some(parse_anfield(&mut reader, &line)?);
        } else if line.starts_with("Piece") {
            piece = Some(parse_piece(&mut reader, &line)?);
        } else if let Some((player, x, y)) = parse_answer(&line) {
            let field = field
                .clone()
                .ok_or_else(|| "answer arrived before the anfield".to_string())?;
            let piece = piece
                .clone()
                .ok_or_else(|| "answer arrived before the piece".to_string())?;
            moves.push(LoggedMove {
                field,
                piece,
                player,
                x,
                y,
            });
        }
    }
    Ok(moves)
}

fn parse_answer(line: &str) -> Option<(i32, i32, i32)> {
    let rest = line.trim().strip_prefix("-> Answer (")?;
    let player = match rest.as_bytes().get(0) {
        Some(b'@') => 1,
        Some(b'$') => 2,
        _ => return None,
    };
    let coords = match rest.split_once("):") {
        Some((_, coords)) => coords.trim(),
        None => return None,
    };
    let mut parts = coords.split_whitespace();
    let x = match parts.next().and_then(|text| text.parse::<i32>().ok()) {
        Some(value) => value,
        None => return None,
    };
    let y = match parts.next().and_then(|text| text.parse::<i32>().ok()) {
        Some(value) => value,
        None => return None,
    };
    Some((player, x, y))
}

fn read_line<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    if line.ends_with('\n') {
        line.pop();
    }
    if line.ends_with('\r') {
        line.pop();
    }
    Ok(Some(line))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const USAGE_SAMPLE: &str = "\
$$$ exec p1 : [robots/bender]
Anfield 20 15:
    01234567890123456789
000 ....................
001 ....................
002 .........@..........
003 ....................
004 ....................
005 ....................
006 ....................
007 ....................
008 ....................
009 ....................
010 ....................
011 ....................
012 .........$..........
013 ....................
014 ....................
Piece 4 1:
.OO.
";

    #[test]
    fn parse_usage_sample() {
        let mut stream = Stream::new(Cursor::new(USAGE_SAMPLE));
        let turn = stream.next_turn().unwrap().unwrap();
        assert_eq!(turn.player, 1);
        assert_eq!(turn.field.width, 20);
        assert_eq!(turn.field.height, 15);
        assert_eq!(turn.field.cells[2][9], b'@');
        assert_eq!(turn.field.cells[12][9], b'$');
        assert_eq!(turn.piece.width, 4);
        assert_eq!(turn.piece.height, 1);
        assert_eq!(turn.piece.cells[0], b".OO.");
        assert_eq!(
            turn.piece.blocks,
            vec![Point { x: 1, y: 0 }, Point { x: 2, y: 0 }]
        );
    }

    #[test]
    fn parse_player_two_and_hash_piece() {
        let input = "\
$$$ exec p2 : [robots/wall_e]
Anfield 2 1:
    01
000 .$
Piece 2 2:
.#
#.
";
        let turn = Stream::new(Cursor::new(input)).next_turn().unwrap().unwrap();
        assert_eq!(turn.player, 2);
        assert_eq!(
            turn.piece.blocks,
            vec![Point { x: 1, y: 0 }, Point { x: 0, y: 1 }]
        );
    }

    #[test]
    fn parse_rejects_bad_row() {
        let input = "$$$ exec p1 : [x]\nAnfield 2 1:\n000 .\nPiece 1 1:\nO\n";
        let err = Stream::new(Cursor::new(input)).next_turn().unwrap_err();
        assert!(err.contains("cells"), "{}", err);
    }
}
