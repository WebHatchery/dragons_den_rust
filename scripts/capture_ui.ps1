<#
.SYNOPSIS
    Headless screenshot harness for Dragon's Den.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it once per scene through the env-var capture hook
    (DRAGONS_DEN_CAPTURE_*) wired in src/main.rs via macroquad_toolkit::capture.

    The shared script derives the package name, exe path, and env-var prefix
    from `cargo metadata`. This game's package is `dragons_den`, so the derived
    prefix is exactly DRAGONS_DEN — no -Prefix override is needed (the previous
    template wrapper hardcoded GAME_TEMPLATE, which the game ignored, so no PNG
    was ever written).

    Scenes map to src/game.rs::begin_capture_scene: `menu`, `hoard`, and
    `settings`; anything else boots a fresh gameplay session (seed 42). Each
    scene writes docs/verification/ui_<scene>.png.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Scenes hoard -SkipBuild
    ./scripts/capture_ui.ps1 -Scenes menu,hoard,settings -Frames 60
#>
param(
    [string[]]$Scenes = @("menu", "hoard", "settings"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

if (-not (Test-Path -LiteralPath $shared)) {
    throw "Shared capture script not found at '$shared'."
}

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
