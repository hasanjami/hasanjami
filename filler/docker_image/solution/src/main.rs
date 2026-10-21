use std::io::{self, Write};
use std::process::exit;

fn main() {
    if let Err(err) = filler::play(io::stdin(), io::stdout()) {
        let _ = writeln!(io::stderr(), "filler: {}", err);
        exit(1);
    }
}
