//! Territory search. The engine counts every cell that ends closer to one
//! player, so a move is scored by how that count changes, not by the cells
//! the piece itself paints.

use crate::parse::{is_enemy, is_mine, Anfield, Piece, Point};
use crate::place::legal_moves;

const CANDIDATE_CAP: usize = 96;

struct Scored {
    origin: Point,
    heat: i32,
    fresh: i32,
    delta: i32,
    scored: bool,
}

/// Pick one legal origin. A stuck position answers `0 0`, which the engine
/// accepts and then keeps sending pieces until the stream ends.
pub fn choose(field: &Anfield, piece: &Piece, player: i32) -> (i32, i32) {
    let moves = legal_moves(field, piece, player);
    if moves.is_empty() {
        return (0, 0);
    }
    let enemy = distances_from(field, player, true, true);
    let mut ranked: Vec<Scored> = moves
        .iter()
        .map(|origin| {
            let (heat, fresh) = heat_of(field, piece, player, *origin, &enemy);
            Scored {
                origin: *origin,
                heat,
                fresh,
                delta: 0,
                scored: false,
            }
        })
        .collect();
    ranked.sort_by(|left, right| {
        right
            .heat
            .cmp(&left.heat)
            .then(right.fresh.cmp(&left.fresh))
            .then(left.origin.y.cmp(&right.origin.y))
            .then(left.origin.x.cmp(&right.origin.x))
    });

    let mut fullest_index = 0;
    for index in 1..ranked.len() {
        let fresher = ranked[index].fresh > ranked[fullest_index].fresh;
        let same_cells = ranked[index].fresh == ranked[fullest_index].fresh
            && lower(ranked[index].origin, ranked[fullest_index].origin);
        if fresher || same_cells {
            fullest_index = index;
        }
    }

    let before = margin(field, player);
    let limit = CANDIDATE_CAP.min(ranked.len());
    for index in 0..limit {
        ranked[index].delta = margin_after(field, piece, player, ranked[index].origin) - before;
        ranked[index].scored = true;
    }
    if !ranked[fullest_index].scored {
        let origin = ranked[fullest_index].origin;
        ranked[fullest_index].delta = margin_after(field, piece, player, origin) - before;
        ranked[fullest_index].scored = true;
    }

    let mut best = 0;
    for index in 1..ranked.len() {
        if !ranked[index].scored {
            continue;
        }
        if better(&ranked[index], &ranked[best]) {
            best = index;
        }
    }
    if ranked[best].delta <= 0 && better(&ranked[fullest_index], &ranked[best]) {
        return (ranked[fullest_index].origin.x, ranked[fullest_index].origin.y);
    }
    (ranked[best].origin.x, ranked[best].origin.y)
}

fn lower(left: Point, right: Point) -> bool {
    left.y < right.y || (left.y == right.y && left.x < right.x)
}

fn better(left: &Scored, right: &Scored) -> bool {
    if left.delta != right.delta {
        return left.delta > right.delta;
    }
    if left.fresh != right.fresh {
        return left.fresh > right.fresh;
    }
    if left.heat != right.heat {
        return left.heat > right.heat;
    }
    lower(left.origin, right.origin)
}

fn heat_of(
    field: &Anfield,
    piece: &Piece,
    player: i32,
    origin: Point,
    enemy: &[i32],
) -> (i32, i32) {
    let width = field.width as usize;
    let span = (field.width + field.height) as i32;
    let mut heat = 0;
    let mut fresh = 0;
    for block in &piece.blocks {
        let x = origin.x + block.x;
        let y = origin.y + block.y;
        if is_mine(player, field.cells[y as usize][x as usize]) {
            continue;
        }
        fresh += 1;
        let distance = enemy[y as usize * width + x as usize];
        if distance >= 0 {
            heat += span - distance;
        }
    }
    (heat, fresh)
}

fn margin_after(field: &Anfield, piece: &Piece, player: i32, origin: Point) -> i32 {
    let mut next = field.clone();
    for block in &piece.blocks {
        let x = (origin.x + block.x) as usize;
        let y = (origin.y + block.y) as usize;
        if !is_mine(player, next.cells[y][x]) {
            next.cells[y][x] = if player == 1 { b'a' } else { b's' };
        }
    }
    margin(&next, player)
}

/// Our estimated cells minus the opponent's. Empty cells belong to whoever
/// is strictly closer. A tie belongs to neither player.
fn margin(field: &Anfield, player: i32) -> i32 {
    let mine = distances_from(field, player, false, false);
    let enemy = distances_from(field, player, true, false);
    let mut ours = 0;
    let mut theirs = 0;
    for y in 0..field.height {
        for x in 0..field.width {
            let cell = field.cells[y as usize][x as usize];
            if is_mine(player, cell) {
                ours += 1;
                continue;
            }
            if is_enemy(player, cell) {
                theirs += 1;
                continue;
            }
            let index = (y as usize) * (field.width as usize) + x as usize;
            let own_dist = mine[index];
            let enemy_dist = enemy[index];
            if own_dist >= 0 && (enemy_dist < 0 || own_dist < enemy_dist) {
                ours += 1;
            } else if enemy_dist >= 0 && (own_dist < 0 || enemy_dist < own_dist) {
                theirs += 1;
            }
        }
    }
    ours - theirs
}

/// Four-direction distance from every source.
///
/// Territory walks only through empty cells, so each player blocks the other.
/// The heatmap's enemy distance walks through occupied cells as well.
fn distances_from(
    field: &Anfield,
    player: i32,
    from_enemy: bool,
    through_occupied: bool,
) -> Vec<i32> {
    let width = field.width as usize;
    let height = field.height as usize;
    let mut dist = vec![-1; width * height];
    let mut queue = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let cell = field.cells[y][x];
            let source = if from_enemy {
                is_enemy(player, cell)
            } else {
                is_mine(player, cell)
            };
            if source {
                let index = y * width + x;
                dist[index] = 0;
                queue.push(index);
            }
        }
    }
    let mut head = 0;
    while head < queue.len() {
        let index = queue[head];
        head += 1;
        let x = (index % width) as i32;
        let y = (index / width) as i32;
        let next = dist[index] + 1;
        let neighbors = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
        for (nx, ny) in neighbors {
            if nx < 0 || ny < 0 || nx >= field.width || ny >= field.height {
                continue;
            }
            let next_index = ny as usize * width + nx as usize;
            if dist[next_index] >= 0 {
                continue;
            }
            if !through_occupied && field.cells[ny as usize][nx as usize] != b'.' {
                continue;
            }
            dist[next_index] = next;
            queue.push(next_index);
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{Anfield, Piece, Stream};
    use std::io::Cursor;
    use std::time::Instant;

    fn board(rows: &[&str]) -> Anfield {
        Anfield {
            width: rows[0].len() as i32,
            height: rows.len() as i32,
            cells: rows.iter().map(|row| row.as_bytes().to_vec()).collect(),
        }
    }

    fn shape(rows: &[&str]) -> Piece {
        crate::parse::parse_piece(
            &mut Cursor::new(rows.join("\n") + "\n"),
            &format!("Piece {} {}:", rows[0].len(), rows.len()),
        )
        .unwrap()
    }

    #[test]
    fn choose_advances_toward_open_ground() {
        let field = board(&[".@...$"]);
        let piece = shape(&[".OO"]);
        assert_eq!(choose(&field, &piece, 1), (0, 0));
    }

    #[test]
    fn choose_returns_origin_when_stuck() {
        let field = board(&["@@"]);
        let piece = shape(&["OO"]);
        assert_eq!(choose(&field, &piece, 1), (0, 0));
    }

    #[test]
    fn choose_on_the_usage_board_is_legal() {
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
        let turn = Stream::new(Cursor::new(input)).next_turn().unwrap().unwrap();
        let (x, y) = choose(&turn.field, &turn.piece, turn.player);
        assert!(
            crate::place::is_legal(&turn.field, &turn.piece, 1, Point { x, y }),
            "chose {} {}",
            x,
            y
        );
        assert_eq!((x, y), (8, 2));
    }

    #[test]
    fn choose_on_a_large_board_stays_fast() {
        let width = 99;
        let height = 100;
        let mut cells = vec![vec![b'.'; width]; height];
        for x in 0..50 {
            cells[20][x] = b'@';
            cells[70][x] = b'$';
        }
        let field = Anfield {
            width: width as i32,
            height: height as i32,
            cells,
        };
        let piece = shape(&["OOO.", ".O..", "OO.."]);
        let started = Instant::now();
        let (x, y) = choose(&field, &piece, 1);
        assert!(started.elapsed().as_secs() < 3);
        assert!(crate::place::is_legal(&field, &piece, 1, Point { x, y }));
    }
}
