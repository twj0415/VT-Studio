param(
  [switch]$SkipFrontendBuild,
  [switch]$SkipFullCargoTest
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

Run-Step "Rust format check" {
  Push-Location $tauriRoot
  try {
    cargo fmt --check
  } finally {
    Pop-Location
  }
}

Run-Step "Rust export/backup/diagnostic tests" {
  Push-Location $tauriRoot
  try {
    cargo test export
  } finally {
    Pop-Location
  }
}

Run-Step "Rust error registry tests" {
  Push-Location $tauriRoot
  try {
    cargo test task_error_registry_covers_todo_09_error_codes
  } finally {
    Pop-Location
  }
}

Run-Step "Rust log service tests" {
  Push-Location $tauriRoot
  try {
    cargo test log_service
  } finally {
    Pop-Location
  }
}

if (-not $SkipFullCargoTest) {
  Run-Step "Rust full test suite" {
    Push-Location $tauriRoot
    try {
      cargo test
    } finally {
      Pop-Location
    }
  }
}

Run-Step "Frontend typecheck" {
  Push-Location $repoRoot
  try {
    pnpm --dir src typecheck
  } finally {
    Pop-Location
  }
}

if (-not $SkipFrontendBuild) {
  Run-Step "Frontend build" {
    Push-Location $repoRoot
    try {
      pnpm --dir src build
    } finally {
      Pop-Location
    }
  }
}

Write-Host ""
Write-Host "Verification complete."
