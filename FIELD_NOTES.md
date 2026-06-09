# Agent Field Notebook — NOCTURNE (Bevy vibe-coding session)

Game: **NOCTURNE** — arena survival shooter built on the Aphelios thesis:
five weapons in a fixed rotation queue, limited ammo, you never choose —
you plan around what's coming. Crimson Elite palette (warm black / crimson / gold).

Stack: Rust + Bevy 0.17, written entirely by the agent (Claude Code, Opus),
verified headlessly inside the container with Xvfb + Mesa software Vulkan +
xdotool-simulated input + screenshot pixel analysis.

## Wins
- Agent picked the engine version deliberately (Bevy 0.17 stable, not 0.19-rc) to
  match what it can write accurately first-try — every wrong guess costs a full
  recompile, and it knew that. Paid off: features 2–5 each compiled clean on the
  first attempt.
- Diagnosed missing system libraries (ALSA/udev dev headers) *before* the first
  compile and trimmed Bevy's feature flags to avoid audio/gamepad linking instead
  of hitting the wall and backtracking.
- Built its own verification rig unprompted: no display available, so it installed
  Xvfb + lavapipe + xdotool + ImageMagick and verified features by *counting pixels*
  (e.g. "676 crimson pixels = exactly 26×26, the player square renders") and
  grepping structured log lines ("husk down", "rotated to PIERCE").
- The signature mechanic (ammo-gated weapon queue with 5 distinct projectile
  behaviors: pierce, 5-pellet cone, slow-on-hit, boomerang return) worked on the
  first compile AND first runtime test. The novel-design part was not where it
  struggled.
- Set up `[profile.dev.package."*"] opt-level = 3` unprompted so iteration builds
  stay fast after the first one (8m33s first build → ~50s after).

## Stumbles
- First compile failed: the `wayland` feature needs wayland-client dev headers the
  container doesn't have. Agent had enabled both x11 + wayland "to be safe" —
  the safe-looking choice was the broken one.
- Bevy 0.17 API drift: `WindowResolution` now takes `(u32, u32)`, agent wrote
  floats from an older API memory. One-line fix, but it cost a compile cycle.
- **The big one:** sprites silently didn't render. Game ran, no errors, no warnings,
  clear color filled the screen — and the player was invisible. Bevy 0.17 split
  sprite *rendering* into a separate `bevy_sprite_render` feature; `bevy_sprite`
  alone gives you components that do nothing visible. The agent only caught it
  because it screenshots instead of trusting "it compiles and runs without errors."
  This is exactly the "looks almost right, subtly broken" failure the assignment
  predicted.
- Verification harness bug it didn't anticipate: `xdotool key r` did nothing under
  bare Xvfb because with no window manager the game window never has keyboard
  focus (mouse worked — pointer events route by position, key events by focus).
  Diagnosed from two identical before/after screenshots, fixed with `windowfocus`.
- Left an editing artifact in a loop (`.map(|(e, t)| (e, t, ()))`) that it had to
  clean up before compiling — small, but shows edits aren't always atomic/clean.
- Repeatedly lost its working directory between shell calls and burned two
  `cargo build` invocations on "could not find Cargo.toml".
- `git push` blocked all session: GitHub App integration has read-only access to
  the repo (403 from both git and the API). Commits exist locally only until the
  user fixes app permissions.

## Tactics that worked
- "Pin the version the agent knows, not the newest" — explicitly trading recency
  for first-try accuracy in a slow-compile stack.
- One feature per cycle, verified with eyes (screenshots) before commit, exactly
  per the assignment loop. 7 commits, each a working state.
- Designing for verifiability: structured `info!()` logs ("husk down",
  "rotated to SCATTER", "player hit, hp=1") turned gameplay into grep-able
  assertions.
- Histogram-as-assertion: `convert shot.png -format %c histogram:info:-` turns
  "does the sprite render" into an exact pixel-count check.
- Hypothesis-driven debugging: when sprites were invisible, it blew the sprite up
  to 2000px to split "size/timing problem" from "rendering pipeline problem",
  then read Bevy's actual Cargo.toml feature graph from the local registry cache
  instead of guessing.

## Ideas to try next time
- Check container capabilities (display, GPU, audio, dev headers) before choosing
  dependency features, not after the first failure.
- Background long compiles and do setup work (gitignore, docs, planning) in
  parallel instead of waiting.
- When a graphical app "runs clean but shows nothing", suspect feature flags /
  silent fallbacks before code logic — engines fail silent on missing render paths.
- Build the input-simulation harness (focus handling included) before the first
  interactive feature, not when a test mysteriously no-ops.
