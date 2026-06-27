param(
  [switch]$SkipHeavySidecars
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$resourceBin = Join-Path $repoRoot "resources/bin"
$tauriConfig = Join-Path $repoRoot "src-tauri/tauri.conf.json"
$packageJson = Join-Path $repoRoot "package.json"

function Require-File {
  param(
    [string]$RelativePath,
    [string]$Reason
  )

  $path = Join-Path $repoRoot $RelativePath
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
    throw "Missing release resource: $RelativePath. $Reason"
  }
}

function Require-Directory {
  param(
    [string]$RelativePath,
    [string]$Reason
  )

  $path = Join-Path $repoRoot $RelativePath
  if (-not (Test-Path -LiteralPath $path -PathType Container)) {
    throw "Missing release resource directory: $RelativePath. $Reason"
  }
}

function Require-JsonText {
  param(
    [string]$PathValue,
    [string]$Needle,
    [string]$Reason
  )

  $text = Get-Content -LiteralPath $PathValue -Raw
  if (-not $text.Contains($Needle)) {
    throw "Release config check failed: $Needle not found in $PathValue. $Reason"
  }
}

Require-File "package.json" "Root scripts must be available for release build."
Require-File "src-tauri/tauri.conf.json" "Tauri bundle config is required."
Require-Directory "resources/bin" "Tauri bundle.resources points at resources/bin."

Require-JsonText $packageJson '"tauri:build:windows"' "Windows x64 release must have a dedicated build script."
Require-JsonText $packageJson 'x86_64-pc-windows-msvc' "The first release target must be Windows x64."
Require-JsonText $tauriConfig '"active": true' "Tauri bundle must be enabled for release packaging."
Require-JsonText $tauriConfig '"nsis"' "The first Windows installer target must be NSIS."
Require-JsonText $tauriConfig '../resources/bin/**/*' "Sidecar packaging source must be included in bundle resources."

if (-not $SkipHeavySidecars) {
  Require-File "resources/bin/ffmpeg.exe" "FFmpeg must be bundled; do not rely on a global ffmpeg."
  Require-File "resources/bin/ffprobe.exe" "FFprobe must be bundled; do not rely on a global ffprobe."
  Require-File "resources/bin/node.exe" "Template rendering must use bundled Node."
  Require-File "resources/bin/chromium.exe" "Template rendering keeps a launcher fallback for sidecar checks."
  Require-File "resources/bin/chromium/chrome.exe" "Template rendering must include the full Chromium directory, not only a copied launcher."
  Require-File "resources/bin/playwright-driver.js" "Template rendering must use bundled Playwright driver."
  Require-Directory "resources/bin/node_modules/playwright-core" "Template rendering must include playwright-core offline."
}

Write-Host "Release resource manifest check passed."
if ($SkipHeavySidecars) {
  Write-Host "Heavy sidecar file checks were skipped."
}
