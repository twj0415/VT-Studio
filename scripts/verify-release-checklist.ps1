param(
  [switch]$SkipTauriBuild
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

function Require-File($RelativePath, $Reason) {
  $path = Join-Path $repoRoot $RelativePath
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
    throw "Release checklist failed: missing $RelativePath. $Reason"
  }
}

Require-File "plan/release-checklist.md" "Release checklist documentation must exist."
Require-File "plan/release-smoke-test.md" "Installer smoke checklist must exist."
Require-File "plan/release-policy.md" "Release policy must exist."
Require-File "plan/license-compliance.md" "License checklist must exist."

Push-Location $repoRoot
try {
  if ($SkipTauriBuild) {
    powershell -ExecutionPolicy Bypass -File scripts/verify-release-smoke.ps1 -SkipTauriBuild
  } else {
    powershell -ExecutionPolicy Bypass -File scripts/verify-release-smoke.ps1
  }
  if ($LASTEXITCODE -ne 0) {
    throw "Release smoke preflight failed with exit code $LASTEXITCODE."
  }
} finally {
  Pop-Location
}

Write-Host "Release checklist preflight complete. Manual installer smoke remains required."
