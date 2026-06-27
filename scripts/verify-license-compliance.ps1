param(
  [switch]$AllowMissingSidecars
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

function Fail($Message) {
  throw "License compliance check failed: $Message"
}

function Require-File($RelativePath, $Reason) {
  $path = Join-Path $repoRoot $RelativePath
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
    Fail "missing $RelativePath. $Reason"
  }
}

function Require-Directory($RelativePath, $Reason) {
  $path = Join-Path $repoRoot $RelativePath
  if (-not (Test-Path -LiteralPath $path -PathType Container)) {
    Fail "missing $RelativePath. $Reason"
  }
}

Require-File "plan/license-compliance.md" "Release license checklist must be documented."
Require-File "resources/licenses/THIRD_PARTY_NOTICES.md" "Third-party notices must be bundled."

$tauriConfig = Get-Content -LiteralPath (Join-Path $repoRoot "src-tauri/tauri.conf.json") -Raw | ConvertFrom-Json
$resources = @($tauriConfig.bundle.resources)
if (-not ($resources -contains "../resources/licenses/**/*")) {
  Fail "src-tauri/tauri.conf.json must bundle resources/licenses."
}

$templateRoot = Join-Path $repoRoot "templates/builtin"
if (Test-Path -LiteralPath $templateRoot) {
  $externalReferences = Get-ChildItem -LiteralPath $templateRoot -Recurse -File |
    Where-Object { $_.Extension -in @(".html", ".css", ".js") } |
    ForEach-Object {
      $content = Get-Content -LiteralPath $_.FullName -Raw
      if ($content -match '(?i)https?://|file://') {
        $_.FullName
      }
    }
  if ($externalReferences) {
    Fail "builtin templates must not reference external or file URLs: $($externalReferences -join ', ')"
  }
}

$sidecarRequirements = @(
  @("resources/bin/ffmpeg.exe", "FFmpeg binary source and license must be recorded before release."),
  @("resources/bin/ffprobe.exe", "FFprobe binary source and license must be recorded before release."),
  @("resources/bin/node.exe", "Node.js license notices must be recorded before release."),
  @("resources/bin/chromium", "Chromium license and third-party notices must be recorded before release."),
  @("resources/bin/node_modules/playwright-core", "Playwright license, NOTICE, and ThirdPartyNotices must be bundled.")
)

if (-not $AllowMissingSidecars) {
  foreach ($item in $sidecarRequirements) {
    $relativePath = $item[0]
    $reason = $item[1]
    $path = Join-Path $repoRoot $relativePath
    if (-not (Test-Path -LiteralPath $path)) {
      Fail "missing $relativePath. $reason"
    }
  }
} else {
  Write-Host "Skipping sidecar presence checks. This is allowed only for checklist/script validation, not for release."
}

Write-Host "License compliance checklist passed."
