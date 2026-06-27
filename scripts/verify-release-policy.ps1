param()

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

function Fail($Message) {
  throw "Release policy check failed: $Message"
}

function Read-Json($RelativePath) {
  $path = Join-Path $repoRoot $RelativePath
  if (-not (Test-Path -LiteralPath $path)) {
    Fail "missing $RelativePath"
  }
  return Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
}

$rootPackage = Read-Json "package.json"
$frontendPackage = Read-Json "src/package.json"
$tauriConfig = Read-Json "src-tauri/tauri.conf.json"

$cargoToml = Get-Content -LiteralPath (Join-Path $repoRoot "src-tauri/Cargo.toml") -Raw
$cargoVersionMatch = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $cargoVersionMatch.Success) {
  Fail "src-tauri/Cargo.toml version was not found"
}
$cargoVersion = $cargoVersionMatch.Groups[1].Value

$versions = @(@(
  $rootPackage.version,
  $frontendPackage.version,
  $tauriConfig.version,
  $cargoVersion
) | Select-Object -Unique)

if ($versions.Count -ne 1) {
  Fail "package.json, src/package.json, src-tauri/tauri.conf.json and src-tauri/Cargo.toml versions must match"
}

$updaterConfig = $null
if ($tauriConfig.plugins -and $tauriConfig.plugins.updater) {
  $updaterConfig = $tauriConfig.plugins.updater
  Fail "Tauri updater plugin is not enabled for the first Windows release"
}

$updaterText = if ($updaterConfig) { $updaterConfig | ConvertTo-Json -Depth 10 } else { "" }
if ($updaterText -match '(?i)http://') {
  Fail "release/update URLs must not use http://"
}

$configText = Get-Content -LiteralPath (Join-Path $repoRoot "src-tauri/tauri.conf.json") -Raw
if ($updaterText -match '(?i)(api[_-]?key|secret|token|password|private[_-]?key)' -or $configText -match '(?i)(api[_-]?key|secret|token|password|private[_-]?key)') {
  Fail "release/update configuration must not contain secrets or secret-like keys"
}

Write-Host "Release policy check passed. Version: $($versions[0]); auto-update remains disabled."
