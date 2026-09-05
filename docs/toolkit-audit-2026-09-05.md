# Toolkit audit — 5 September 2026

The original review had no named migration finding. Current inspection confirms:

- `src/data.rs` loads all seven catalogues through labeled toolkit parsing.
- `src/save.rs` uses shared versioned slots, existence/deletion and version
  inspection. The game's first-release schema policy rejects unsupported
  versions explicitly; offline timestamps and state validation remain local.
- Gameplay and exploration use SeededRng; achievements use the shared model.
- Game orchestration uses EventBus, FloatingTextLayer, notifications, settings
  and ScrollArea. UI uses VirtualUi, shared text, surfaces and visibility checks.
- Idle-number formatting re-exports shared amount/rate formatting. Main uses
  the toolkit capture lifecycle.

No remaining generic JSON loader, save backend, wrapping loop, RNG primitive,
sound bank or particle implementation was found. Dragon progression, prestige,
exploration and presentation remain game-owned.

Validation also refreshed this standalone game's lockfile for the current
shared toolkit native HTTP dependencies. Final validation: 61 checks,
formatting, strict all-target/all-feature Clippy and Rust source-size limits.
Default `publish.ps1` passed Windows/WebGL release builds, Preview deployment
and Project Roost tracking without compiler or publisher warnings.
