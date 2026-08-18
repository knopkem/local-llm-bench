# Testbed for local llms


## Prompt

"Implement a racer game in the style of the 1990 amiga game Lotus Esprit Turbo Challenge. Don't use 3D graphics use the style and technique of that time. Singleplayer mode only, full playable game with sounds. Build a native rust application."

## Approach

- plan mode first, then implementation in one go
- no correction afterwards, evaluation as is

## Testsystem
- Macbook Pro M4 Max 36GB
- LM Studio
- opencode

## Models

### Qwen 3.8-27B
- Q3-K-M
- unsloth version
- 14.7GB
- 120k context window
- ~10t/s
- build time: ~12h

Result:
- complete game runs and is playable
- correct car behavior (get slower outside track, drifts in bends)
- correct visuals
- minor issues only: main menue does not allow track selection, car oppenents way to slow, no speed change on collision
- ai wrote headless helpers to check visuals and debug issues

### Ornith 1.0-35B
- IQ3-XXS
- unsloth version
- 15.5GB
- 100k context window
- ~50t/s
- build time: ~4h

Result:
- runs but not playable
- visuals broken

### Muse-Glimmer-30B
- IQ3-M
- unsloth version
- 15.5GB
- 100k context window
- ~13t/s
- build time: ~5h


Result:
- app runs and is "playable"
- player is a dot that moves to 3 positions on a horizon
- opponents are dots that have no real influence and move down
- no menue, sound or gameplay
