# NOCTURNE

An arena survival shooter where **you never choose your weapon — you plan around
the queue**. Five moonlight arms in a fixed rotation; each has limited ammo, and
when one runs dry it rotates to the back of the line. The skill isn't aiming.
It's knowing what's coming two weapons from now and positioning for it.

Built in Rust with [Bevy 0.17](https://bevy.org) as a vibe-coding exercise —
every line written by an AI agent, verified headlessly with screenshots before
each commit.

## The queue

| Weapon | Ammo | Behavior |
|---|---|---|
| LUMEN | 18 | Baseline rifle — steady gold rounds |
| PIERCE | 6 | Railgun lance — slow fire, heavy damage, goes through everything |
| SCATTER | 8 | Moonburst — 5 pellets in a cone, short range |
| GRAVEN | 10 | Gravity orb — chills enemies it hits to 35% speed |
| CRESCENT | 6 | Returning blade — boomerangs back to you, hits both ways |

One trigger pull = one round, even for multi-pellet weapons. When a weapon
empties it rotates out and the next one is live immediately.

## Controls

- **WASD / arrows** — move
- **Mouse** — aim, **hold left click** — fire
- **R** — restart after you're eclipsed

You have 3 HP. Husks spawn at the arena edges and chase you; spawn pressure
ramps from 1.4s to 0.45s per husk over about two minutes.

## Run it

```sh
cd nocturne
cargo run
```

First build takes several minutes (it's Bevy); after that it's seconds.
Audio and gamepad features are disabled so it builds on headless/minimal
Linux boxes without ALSA/udev dev headers.

See [FIELD_NOTES.md](FIELD_NOTES.md) for the agent-observation log from the
build session.
