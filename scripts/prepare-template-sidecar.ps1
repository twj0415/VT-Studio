param(
  [Parameter(Mandatory = $true)]
  [string]$NodeExePath,

  [Parameter(Mandatory = $true)]
  [string]$ChromiumExePath,

  [Parameter(Mandatory = $true)]
  [string]$PlaywrightCoreDir
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$resourceBin = Join-Path $repoRoot "resources/bin"
$runtimeSidecars = Join-Path $repoRoot "sidecars"
$driverSource = Join-Path $runtimeSidecars "playwright-driver.js"

function Require-Leaf {
  param([string]$PathValue, [string]$Label)
  if (-not (Test-Path -LiteralPath $PathValue -PathType Leaf)) {
    throw "$Label does not point to an existing file: $PathValue"
  }
}

function Require-Directory {
  param([string]$PathValue, [string]$Label)
  if (-not (Test-Path -LiteralPath $PathValue -PathType Container)) {
    throw "$Label does not point to an existing directory: $PathValue"
  }
}

function Copy-RequiredFile {
  param([string]$Source, [string]$FileName)
  Copy-Item -LiteralPath $Source -Destination (Join-Path $resourceBin $FileName) -Force
  Copy-Item -LiteralPath $Source -Destination (Join-Path $runtimeSidecars $FileName) -Force
}

function Copy-ChromiumDirectory {
  param([string]$ChromiumExe)

  $sourceDir = Split-Path -Parent $ChromiumExe
  $resourceChromium = Join-Path $resourceBin "chromium"
  $runtimeChromium = Join-Path $runtimeSidecars "chromium"

  if (Test-Path -LiteralPath $resourceChromium) {
    Remove-Item -LiteralPath $resourceChromium -Recurse -Force
  }
  if (Test-Path -LiteralPath $runtimeChromium) {
    Remove-Item -LiteralPath $runtimeChromium -Recurse -Force
  }

  Copy-Item -LiteralPath $sourceDir -Destination $resourceChromium -Recurse -Force
  Copy-Item -LiteralPath $sourceDir -Destination $runtimeChromium -Recurse -Force
}

Require-Leaf $NodeExePath "Node executable"
Require-Leaf $ChromiumExePath "Chromium executable"
Require-Directory $PlaywrightCoreDir "playwright-core directory"
Require-Leaf $driverSource "Playwright driver"

New-Item -ItemType Directory -Force -Path $resourceBin | Out-Null
New-Item -ItemType Directory -Force -Path $runtimeSidecars | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $resourceBin "node_modules") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $runtimeSidecars "node_modules") | Out-Null

Copy-RequiredFile $NodeExePath "node.exe"
Copy-RequiredFile $ChromiumExePath "chromium.exe"
Copy-ChromiumDirectory $ChromiumExePath
Copy-RequiredFile $driverSource "playwright-driver.js"

$resourcePlaywright = Join-Path $resourceBin "node_modules/playwright-core"
$runtimePlaywright = Join-Path $runtimeSidecars "node_modules/playwright-core"
if (Test-Path -LiteralPath $resourcePlaywright) {
  Remove-Item -LiteralPath $resourcePlaywright -Recurse -Force
}
if (Test-Path -LiteralPath $runtimePlaywright) {
  Remove-Item -LiteralPath $runtimePlaywright -Recurse -Force
}
Copy-Item -LiteralPath $PlaywrightCoreDir -Destination $resourcePlaywright -Recurse -Force
Copy-Item -LiteralPath $PlaywrightCoreDir -Destination $runtimePlaywright -Recurse -Force

Write-Host "Template sidecars prepared:"
Write-Host "  resources/bin/node.exe"
Write-Host "  resources/bin/chromium.exe"
Write-Host "  resources/bin/chromium/"
Write-Host "  resources/bin/playwright-driver.js"
Write-Host "  resources/bin/node_modules/playwright-core"
Write-Host "  sidecars/node.exe"
Write-Host "  sidecars/chromium.exe"
Write-Host "  sidecars/chromium/"
Write-Host "  sidecars/playwright-driver.js"
Write-Host "  sidecars/node_modules/playwright-core"
