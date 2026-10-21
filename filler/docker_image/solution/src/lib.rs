//! Anfield protocol, placement rules, and the move picker.

pub mod parse;
pub mod place;
pub mod strategy;

use std::io::{self, BufReader, Read, Write};

/// Play until the engine closes stdin. Every coordinate is flushed, because a
/// delayed line is a timeout and a timeout loses the match.
pub fn play(input: impl Read, output: impl Write) -> io::Result<()> {
    let mut stream = parse::Stream::new(BufReader::new(input));
    let mut output = std::io::BufWriter::new(output);
    let debug = std::env::var_os("FILLER_DEBUG").is_some();
    loop {
        match stream.next_turn() {
            Ok(None) => return output.flush(),
            Ok(Some(turn)) => {
                let (x, y) = strategy::choose(&turn.field, &turn.piece, turn.player);
                if debug {
                    eprintln!("play {} {}", x, y);
                }
                write!(output, "{}", place::format_move(x, y))?;
                output.flush()?;
            }
            Err(err) => {
                eprintln!("filler: {}", err);
                write!(output, "{}", place::format_move(0, 0))?;
                return output.flush();
            }
        }
    }
}

/// Count answers in a verbose engine log.
/// Returns `(checked, stuck, illegal)`. A stuck turn is `0 0` when no legal
/// origin exists, which is the required reply.
pub fn audit_log(text: &str) -> Result<(usize, usize, usize), String> {
    let mut checked = 0;
    let mut stuck = 0;
    let mut illegal = 0;
    for turn in parse::logged_moves(text)? {
        checked += 1;
        let origin = parse::Point {
            x: turn.x,
            y: turn.y,
        };
        if place::is_legal(&turn.field, &turn.piece, turn.player, origin) {
            continue;
        }
        if place::legal_moves(&turn.field, &turn.piece, turn.player).is_empty() {
            stuck += 1;
        } else {
            illegal += 1;
        }
    }
    Ok((checked, stuck, illegal))
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
    fn play_prints_one_coordinate_line() {
        let mut output = Vec::new();
        play(Cursor::new(USAGE_SAMPLE), &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert_eq!(text, "8 2\n");
        let turn = parse::Stream::new(Cursor::new(USAGE_SAMPLE))
            .next_turn()
            .unwrap()
            .unwrap();
        assert!(place::is_legal(
            &turn.field,
            &turn.piece,
            1,
            parse::Point { x: 8, y: 2 }
        ));
    }

    #[test]
    fn play_answers_zero_when_stuck() {
        let input = "\
$$$ exec p1 : [x]
Anfield 2 1:
  01
000 @@
Piece 2 1:
OO
";
        let mut output = Vec::new();
        play(Cursor::new(input), &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "0 0\n");
    }

    #[test]
    fn recorded_match_has_no_illegal_answers() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../visualizer/matches/map00-wall_e.txt"
        );
        let text = std::fs::read_to_string(path).expect(path);
        let (checked, stuck, illegal) = audit_log(&text).unwrap();
        assert!(checked >= 100, "checked {}", checked);
        assert_eq!(illegal, 0, "stuck {}", stuck);
    }
}
