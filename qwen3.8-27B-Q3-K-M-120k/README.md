# Lotus Racer

A playable retro arcade racer built with Rust and SDL2, inspired by the presentation and software-rendering techniques of early-1990s Amiga racing games. The game uses a low-resolution indexed framebuffer, projected road segments, scaled 2D sprites, and procedural audio rather than hardware-accelerated 3D graphics.

![Gameplay screenshot](screenshot.png)

## What Landed

- Two tracks: Coastal Circuit and Mountain Pass
- Race and time-trial modes with three difficulty levels
- Title screen, attract mode, event selection, countdown, pause, and results screens
- Fixed-timestep driving with acceleration, braking, reverse, speed-sensitive steering, centrifugal force, and off-road slowdown
- Three AI opponents with corner-aware speed control, lane movement, and collision avoidance
- Player-to-opponent collision response
- Curved and elevated pseudo-3D roads rendered as horizontal scanline strips
- Distance-scaled cars and roadside scenery with crest clipping and painter-order rendering
- Indexed 256-color framebuffer, nearest-neighbor scaling, parallax hills, and palette-cycled skies
- HUD with speed, lap, position, elapsed time, and best-lap information
- Persistent best laps stored in `~/.lotus_racer.json`
- Procedural engine audio, countdown and collision effects, and two chiptune-style music tracks
- Headless race simulation and deterministic PNG screenshot modes

All graphics and sounds are generated in code; the project has no external game asset files.

## Requirements

- Stable Rust toolchain with Cargo
- SDL2 development libraries discoverable through `pkg-config`
- A working audio device is optional; the game continues without audio if initialization fails

On macOS with Homebrew:

```sh
brew install rust sdl2 pkg-config
```

If Rust is already managed through `rustup`, only SDL2 and `pkg-config` are needed:

```sh
brew install sdl2 pkg-config
```

## Build and Run

From this directory:

```sh
cargo run --release
```

The game renders internally at 640x256 and opens a 2x window using nearest-neighbor scaling.

## Controls

| Action | Keys |
| --- | --- |
| Accelerate | Up arrow or W |
| Brake / reverse | Down arrow or S |
| Steer | Left/right arrows or A/D |
| Confirm / start | Enter |
| Change race mode | Tab or X |
| Pause | P |
| Mute audio | M |
| Back / quit | Escape |

On the event selection screen, Tab or X switches between Race and Time Trial.

## Headless Validation

Run an autopilot race simulation without opening a window:

```sh
cargo run --release -- --sim 300 --track 0
cargo run --release -- --sim 300 --track 1
```

Generate a representative gameplay frame without opening a window:

```sh
cargo run --release -- --shot 400 --track 0
cargo run --release -- --shot 400 --track 1
```

The screenshot is written to `/tmp/lotus_shot.png`. The `--shot` value is the number of fixed simulation steps; `--track` accepts `0` or `1`.

Standard Rust checks:

```sh
cargo fmt --check
cargo check
```

## Architecture

- `src/main.rs`: SDL window, input, fixed-timestep loop, audio integration, and headless commands
- `src/app.rs`: screens, menu flow, race scenes, rendering orchestration, and best-lap persistence
- `src/game/`: player physics, AI, race progression, and procedural track construction
- `src/render/`: indexed framebuffer, road projection, backgrounds, sprites, font, and HUD
- `src/audio/`: runtime audio system and procedural sound/music synthesis
- `src/assets.rs`: palettes and code-generated cars, scenery, hills, and track themes
- `src/png.rs`: dependency-light PNG output for screenshot validation

The simulation advances at a fixed 60 Hz. Rendering writes palette indices into a software framebuffer, converts the completed frame to RGBA, and uploads it to an SDL streaming texture. The road is composed of projected track segments and rasterized one horizontal line at a time; all cars and scenery remain 2D sprites.

## Current Scope

The game supports keyboard input only. The selection screen contains track and difficulty options, but its directional menu actions are not currently connected to SDL key events, so interactive races use the default track and difficulty. Both tracks remain available through the headless commands. Best laps use a fixed two-track save format, screenshots always overwrite `/tmp/lotus_shot.png`, and there is no in-game control remapping or settings screen.