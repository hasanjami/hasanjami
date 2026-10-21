use filler_visualizer::{parse_frames, render, sample_log, to_ppm, HEIGHT, WIDTH};
use minifb::{Key, Window, WindowOptions};
use std::env;
use std::fs;
use std::process::exit;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--check") {
        let text = load(args.get(1).map(String::as_str));
        match parse_frames(&text) {
            Ok(frames) => {
                let last = frames.last().unwrap();
                println!(
                    "frames {} last score {}-{}",
                    frames.len(),
                    last.p1,
                    last.p2
                );
            }
            Err(err) => {
                eprintln!("visualizer: {}", err);
                exit(1);
            }
        }
        return;
    }
    if args.first().map(String::as_str) == Some("--export") {
        let dest = args.get(1).map(String::as_str).unwrap_or("preview.ppm");
        let text = load(args.get(2).map(String::as_str));
        let frames = parse_frames(&text).unwrap_or_else(|err| {
            eprintln!("visualizer: {}", err);
            exit(1);
        });
        let index = frames.len() - 1;
        let pixels = render(&frames[index], index, frames.len());
        fs::write(dest, to_ppm(&pixels, WIDTH, HEIGHT)).unwrap_or_else(|err| {
            eprintln!("visualizer: {}", err);
            exit(1);
        });
        println!("wrote {} ({} frames)", dest, frames.len());
        return;
    }

    let text = load(args.first().map(String::as_str));
    let frames = parse_frames(&text).unwrap_or_else(|err| {
        eprintln!("visualizer: {}", err);
        exit(1);
    });
    let mut window = Window::new(
        "Filler",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|err| {
        eprintln!("visualizer: {}", err);
        exit(1);
    });
    window.set_target_fps(60);

    let mut index = 0;
    let mut playing = false;
    let mut last_step = Instant::now();
    let mut space_down = false;
    let mut left_down = false;
    let mut right_down = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let space = window.is_key_down(Key::Space);
        let left = window.is_key_down(Key::Left);
        let right = window.is_key_down(Key::Right);
        if space && !space_down {
            playing = !playing;
        }
        if left && !left_down {
            playing = false;
            if index > 0 {
                index -= 1;
            }
        }
        if right && !right_down {
            playing = false;
            if index + 1 < frames.len() {
                index += 1;
            }
        }
        space_down = space;
        left_down = left;
        right_down = right;
        if playing && last_step.elapsed() >= Duration::from_millis(450) {
            last_step = Instant::now();
            if index + 1 < frames.len() {
                index += 1;
            } else {
                playing = false;
            }
        }
        let pixels = render(&frames[index], index, frames.len());
        if window.update_with_buffer(&pixels, WIDTH, HEIGHT).is_err() {
            exit(1);
        }
    }
}

fn load(path: Option<&str>) -> String {
    match path {
        Some(path) => fs::read_to_string(path).unwrap_or_else(|err| {
            eprintln!("visualizer: {}", err);
            exit(1);
        }),
        None => sample_log().to_string(),
    }
}
