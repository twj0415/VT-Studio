param(
  [switch]$SkipCargoSmoke
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tauriRoot = Join-Path $repoRoot "src-tauri"
$sidecarRoot = Join-Path $repoRoot "sidecars"

function Require-File {
  param(
    [string]$RelativePath
  )

  $path = Join-Path $repoRoot $RelativePath
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
    throw "Missing required template sidecar: $RelativePath"
  }
}

Write-Host "Checking template sidecars in $sidecarRoot"

Require-File "sidecars/node.exe"
Require-File "sidecars/chromium.exe"
Require-File "sidecars/playwright-driver.js"

$playwrightCore = Join-Path $sidecarRoot "node_modules/playwright-core"
if (-not (Test-Path -LiteralPath $playwrightCore -PathType Container)) {
  throw "Missing required template sidecar dependency: sidecars/node_modules/playwright-core"
}

if (-not $SkipCargoSmoke) {
  Push-Location $tauriRoot
  try {
    cargo test check_template_sidecars -- --nocapture
    cargo test real_template_sidecar_renders_default_cover_png -- --ignored --nocapture
  } finally {
    Pop-Location
  }
}

Write-Host "Template sidecar verification complete."
