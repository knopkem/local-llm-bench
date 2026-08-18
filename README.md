# Local LLM Bench

A small, informal benchmark of local language models generating a native Rust arcade racer from a single prompt. The target was a playable, sound-enabled game inspired by the software-rendered style of the 1990 Amiga game *Lotus Esprit Turbo Challenge*.

## At a Glance

| Model | Quantization | Speed | Build time | Outcome |
| --- | --- | ---: | ---: | --- |
| [Qwen 3.8 27B](qwen3.8-27B-Q3-K-M/) | Q3-K-M | ~10 tok/s | ~12 h | **Playable** |
| [Ornith 1.0 35B](ornith-1-35B-IQ3-XXS/) | IQ3-XXS | ~50 tok/s | ~4 h | Runs, but not playable |
| [Muse-Glimmer 30B](muse-glimmer-30B-IQ3-M/) | IQ3-M | ~13 tok/s | ~5 h | Prototype only |
| [Gemma 4 31B](gemma-4-31b-it-Q3-K-S) | Q3-K-M | ~8 tok/s | ~2 h | Runs, but not playable |

## The Prompt

> Implement a racer game in the style of the 1990 Amiga game *Lotus Esprit Turbo Challenge*. Do not use 3D graphics; use the style and techniques of that time. Single-player mode only, with a fully playable game and sounds. Build a native Rust application.

## Method

- Plan first, then implement in one pass.
- Do not provide corrections after the initial implementation.
- Evaluate each result as generated.

## Test Environment

- MacBook Pro with M4 Max and 36 GB RAM
- LM Studio
- opencode

## Results

### Qwen 3.8 27B

- Unsloth version, 14.7 GB, 120k context window
- Runs at approximately 10 tok/s; build time was about 12 hours
- Engine: SDL2
- Complete, playable game with convincing visuals
- Car slows off-road and drifts through bends
- Minor issues: track selection is unavailable from the main menu, opponents are too slow, and collisions do not change speed
- Added headless helpers for visual checks and debugging

### Ornith 1.0 35B

- Unsloth version, 15.5 GB, 100k context window
- Runs at approximately 50 tok/s; build time was about 4 hours
- Engine: SDL2
- Application runs, but the game is not playable
- Visual output is broken

### Muse-Glimmer 30B

- Unsloth version, 15.5 GB, 100k context window
- Runs at approximately 13 tok/s; build time was about 5 hours
- Engine: SDL2
- Application runs and is technically playable
- Player is a dot that moves between three positions on the horizon
- Opponents are dots that move downward without meaningful interaction
- No menu, sound, or real gameplay systems

### Gemma 4 31B

- Unsloth version, 15.5 GB, 100k context window
- Runs at approximately 8 tok/s; build time was about 2 hours
- Engine: macroquad
- Application runs but not playable
- Player is a cube nothing else on screen
- No menu, sound, or real gameplay systems

## Notes

This is a practical snapshot rather than a controlled scientific benchmark. Build time includes generation and implementation time, and the results were not improved through follow-up correction passes.
