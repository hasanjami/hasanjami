# Filler

Rust player for the Anfield game. The bot reads the board and the piece from stdin and answers with `X Y`.

## Setup

Download the official archive if `docker_image/game_engine` is not already there:

```powershell
.\scripts\setup.ps1
```

Build the image, compile the player with the image's Rust compiler, and open a shell in the container. Run these from the `filler` directory:

```powershell
docker build -t filler .\docker_image
.\scripts\audit.ps1 -BuildOnly
docker run -v "${PWD}/docker_image/solution:/filler/solution" -it filler
```

Inside the container:

```sh
./game_engine -f maps/map01 -p1 robots/bender -p2 robots/terminator
./game_engine -f maps/map01 -p1 solution/target/release/filler -p2 robots/bender
```

## Tests

```powershell
cargo test --manifest-path docker_image/solution/Cargo.toml
cargo test --manifest-path visualizer/Cargo.toml
.\scripts\audit.ps1
```

## Visualizer

The replay window is a Rust program. With no arguments it plays a built-in match. Pass a game-engine log to replay that match. Space plays and pauses. The left and right arrows step. Escape closes the window.

```powershell
cargo run --manifest-path visualizer/Cargo.toml
cargo run --manifest-path visualizer/Cargo.toml -- visualizer/matches/map00-wall_e.txt
```
