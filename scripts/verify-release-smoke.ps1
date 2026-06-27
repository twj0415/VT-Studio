param(
  [switch]$SkipTauriBuild
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tauriRoot = Join-Path $repoRoot "src-tauri"

function Run-Step {
  param(
    [string]$Name,
    [scriptblock]$Command
  )

  Write-Host ""
  Write-Host "==> $Name"
  & $Command
}

function Invoke-Native {
  param(
    [Parameter(Mandatory = $true)]
    [string]$FilePath,

    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
  )

  & $FilePath @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Command failed with exit code ${LASTEXITCODE}: $FilePath $($Arguments -join ' ')"
  }
}

Run-Step "Release resource manifest" {
  Push-Location $repoRoot
  try {
    Invoke-Native pnpm run verify:release-resources
  } finally {
    Pop-Location
  }
}

Run-Step "Release policy" {
  Push-Location $repoRoot
  try {
    Invoke-Native pnpm run verify:release-policy
  } finally {
    Pop-Location
  }
}

Run-Step "License compliance" {
  Push-Location $repoRoot
  try {
    Invoke-Native pnpm run verify:license-compliance
  } finally {
    Pop-Location
  }
}

Run-Step "Rust format check" {
  Push-Location $tauriRoot
  try {
    Invoke-Native cargo fmt --check
  } finally {
    Pop-Location
  }
}

Run-Step "Rust check" {
  Push-Location $tauriRoot
  try {
    Invoke-Native cargo check
  } finally {
    Pop-Location
  }
}

Run-Step "Frontend typecheck" {
  Push-Location $repoRoot
  try {
    Invoke-Native pnpm --dir src typecheck
  } finally {
    Pop-Location
  }
}

Run-Step "Frontend build" {
  Push-Location $repoRoot
  try {
    Invoke-Native pnpm --dir src build
  } finally {
    Pop-Location
  }
}

if (-not $SkipTauriBuild) {
  Run-Step "Windows NSIS build" {
    Push-Location $repoRoot
    try {
      Invoke-Native pnpm run tauri:build:windows
    } finally {
      Pop-Location
    }
  }
}

Write-Host ""
Write-Host "Release smoke preflight complete. Continue with manual installer smoke using plan/release-smoke-test.md."
