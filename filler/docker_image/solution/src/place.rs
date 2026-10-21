//! The one-overlap placement rule.

use crate::parse::{is_enemy, is_mine, Anfield, Piece, Point};

/// The only line the engine accepts: `X Y` and a newline.
pub fn format_move(x: i32, y: i32) -> String {
    format!("{} {}\n", x, y)
}

/// A move is legal when every occupied cell stays on the board, exactly one
/// lands on our territory, and none lands on the opponent. Empty padding may
/// hang off the edge, so the origin can be negative.
pub fn is_legal(field: &Anfield, piece: &Piece, player: i32, origin: Point) -> bool {
    if piece.blocks.is_empty() {
        return false;
    }
    let mut overlap = 0;
    for block in &piece.blocks {
        let x = origin.x + block.x;
        let y = origin.y + block.y;
        if x < 0 || y < 0 || x >= field.width || y >= field.height {
            return false;
        }
        let cell = field.cells[y as usize][x as usize];
        if is_enemy(player, cell) {
            return false;
        }
        if is_mine(player, cell) {
            overlap += 1;
        }
    }
    overlap == 1
}

/// Every origin whose occupied cells pass [`is_legal`].
pub fn legal_moves(field: &Anfield, piece: &Piece, player: i32) -> Vec<Point> {
    if piece.blocks.is_empty() {
        return Vec::new();
    }
    let (min_x, max_x, min_y, max_y) = block_span(piece);
    let mut moves = Vec::new();
    let mut y = -min_y;
    while y <= field.height - 1 - max_y {
        let mut x = -min_x;
        while x <= field.width - 1 - max_x {
            let origin = Point { x, y };
            if is_legal(field, piece, player, origin) {
                moves.push(origin);
            }
            x += 1;
        }
        y += 1;
    }
    moves
}

fn block_span(piece: &Piece) -> (i32, i32, i32, i32) {
    let mut min_x = piece.blocks[0].x;
    let mut max_x = piece.blocks[0].x;
    let mut min_y = piece.blocks[0].y;
    let mut max_y = piece.blocks[0].y;
    for block in &piece.blocks {
        min_x = min_x.min(block.x);
        max_x = max_x.max(block.x);
        min_y = min_y.min(block.y);
        max_y = max_y.max(block.y);
    }
    (min_x, max_x, min_y, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{Stream, Turn};
    use std::io::Cursor;

    fn sample() -> (Anfield, Piece) {
        let input = "\
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
        let Turn { field, piece, .. } = Stream::new(Cursor::new(input))
            .next_turn()
            .unwrap()
            .unwrap();
        (field, piece)
    }

    fn board(rows: &[&str]) -> Anfield {
        Anfield {
            width: rows[0].len() as i32,
            height: rows.len() as i32,
            cells: rows.iter().map(|row| row.as_bytes().to_vec()).collect(),
        }
    }

    fn shape(rows: &[&str]) -> Piece {
        crate::parse::parse_piece(&mut Cursor::new(rows.join("\n")), &format!("Piece {} {}:", rows[0].len(), rows.len()))
            .unwrap()
    }

    #[test]
    fn format_includes_newline() {
        assert_eq!(format_move(7, 2), "7 2\n");
        assert_eq!(format_move(0, 0), "0 0\n");
        assert_eq!(format_move(-1, 4), "-1 4\n");
    }

    #[test]
    fn one_overlap_and_rejections() {
        let (field, piece) = sample();
        assert!(is_legal(&field, &piece, 1, Point { x: 7, y: 2 }));
        assert!(is_legal(&field, &piece, 1, Point { x: 8, y: 2 }));
        assert!(!is_legal(&field, &piece, 1, Point { x: 6, y: 2 }));
        assert!(!is_legal(&field, &piece, 1, Point { x: 18, y: 2 }));

        let field = board(&["@@."]);
        let piece = shape(&["OO"]);
        assert!(!is_legal(&field, &piece, 1, Point { x: 0, y: 0 }));

        let field = board(&["@$."]);
        assert!(!is_legal(&field, &piece, 1, Point { x: 0, y: 0 }));

        let field = board(&["@"]);
        let piece = shape(&["O", "O"]);
        assert!(!is_legal(&field, &piece, 1, Point { x: 0, y: 0 }));

        let field = board(&["@.."]);
        let piece = shape(&["OO"]);
        assert!(!is_legal(&field, &piece, 1, Point { x: -1, y: 0 }));

        let field = board(&[".@."]);
        let piece = shape(&[".OO"]);
        assert!(is_legal(&field, &piece, 1, Point { x: -1, y: 0 }));

        let field = board(&[".$"]);
        let piece = shape(&["OO"]);
        assert!(is_legal(&field, &piece, 2, Point { x: 0, y: 0 }));
        let field = board(&[".@."]);
        let piece = shape(&["OO"]);
        assert!(!is_legal(&field, &piece, 2, Point { x: 0, y: 0 }));
    }

    #[test]
    fn legal_moves_keep_blocks_inside() {
        let (field, piece) = sample();
        for origin in legal_moves(&field, &piece, 1) {
            assert!(is_legal(&field, &piece, 1, origin));
            for block in &piece.blocks {
                let x = origin.x + block.x;
                let y = origin.y + block.y;
                assert!(x >= 0 && y >= 0 && x < field.width && y < field.height);
            }
        }
    }
}
