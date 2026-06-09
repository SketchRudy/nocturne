# Agent Field Notebook — NOCTURNE (Bevy vibe-coding session)

Game: **NOCTURNE** — arena survival shooter built on the Aphelios thesis:
five weapons in a fixed rotation queue, limited ammo, you never choose —
you plan around what's coming. Crimson Elite palette (warm black / crimson / gold).

## Wins
- Agent picked the engine version deliberately (Bevy 0.17 stable, not 0.19-rc) to
  match what it can write accurately first-try — every wrong guess costs a full
  recompile, and it knew that.
- Diagnosed missing system libraries (ALSA/udev dev headers) *before* the first
  compile and trimmed Bevy's feature flags to avoid audio/gamepad linking instead
  of hitting the wall and backtracking.
- Set up `[profile.dev.package."*"] opt-level = 3` unprompted so iteration builds
  stay fast after the first one.

## Stumbles
- First compile failed: the `wayland` feature needs wayland-client dev headers the
  container doesn't have. Agent had enabled both x11 + wayland "to be safe" —
  the safe-looking choice was the broken one. Fix was trivial (drop wayland).
- Ran a background `cargo check` from the wrong working directory once — shell
  state doesn't persist between calls the way it assumed.
- `git push` blocked: GitHub App integration has read-only access to this repo
  (403 on both git proxy and API). Not the agent's fault, but it burned several
  attempts confirming it before moving on.

## Tactics that worked
- "Pin the version the agent knows, not the newest" — explicitly trading recency
  for first-try accuracy in a slow-compile stack.
- Commit-per-feature discipline declared up front, before any code existed.
- MVP scoped to literally: window + colored square + WASD. Nothing else.

## Ideas to try next time
- Check container capabilities (display, GPU, audio) before choosing features,
  not after the first failure.
- Background long compiles and do setup work (gitignore, docs, planning) in
  parallel instead of waiting.
