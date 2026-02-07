$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

[CmdletBinding()]
param(
  # If set, skips `npm install` even if `node_modules` is missing.
  [switch]$SkipInstall,
  # If set, skips `cargo clippy` and `cargo test`.
  [switch]$SkipRustChecks
)

function Invoke-External {
  param(
    [Parameter(Mandatory, Position = 0)]
    [string]$FilePath,
    [Parameter(Position = 1, ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
  )

  $argsText = if ($Arguments -and $Arguments.Count -gt 0) { $Arguments -join " " } else { "" }
  if ($argsText -ne "") {
    Write-Host ">>> $FilePath $argsText"
  } else {
    Write-Host ">>> $FilePath"
  }

  & $FilePath @Arguments
  $exitCode = $LASTEXITCODE
  if ($exitCode -ne 0) {
    Write-Error "$FilePath exited with code $exitCode"
    exit $exitCode
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

Write-Host "=== Deterrence — Production Build (Windows) ==="

# 1. Install frontend deps if needed
if (-not $SkipInstall) {
  if (-not (Test-Path -LiteralPath "node_modules" -PathType Container)) {
    Invoke-External npm install
  }
}

# 2. Rust checks
if (-not $SkipRustChecks) {
  Invoke-External cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
  Invoke-External cargo test --manifest-path src-tauri/Cargo.toml
}

# 3. Frontend build (tsc + vite)
Invoke-External npm run build

# 4. Tauri production build (compiles Rust release + bundles Windows installers)
Invoke-External npx tauri build

Write-Host ""
Write-Host "=== Build complete ==="

$bundleRoot = Join-Path "src-tauri" "target\release\bundle"
Write-Host "Artifacts (if bundling succeeded):"

if (-not (Test-Path -LiteralPath $bundleRoot -PathType Container)) {
  Write-Host "  (Bundle directory not found: $bundleRoot)"
  exit 0
}

$artifactExtensions = @("*.msi", "*.exe", "*.msix", "*.appx", "*.appxbundle")
$artifacts =
  Get-ChildItem -Path $bundleRoot -Recurse -File -ErrorAction SilentlyContinue |
  Where-Object { $artifactExtensions -contains ("*{0}" -f $_.Extension) }

if (-not $artifacts -or $artifacts.Count -eq 0) {
  Write-Host "  (No installer artifacts found under: $bundleRoot)"
  exit 0
}

foreach ($a in $artifacts) {
  Write-Host ("  - {0}" -f $a.FullName)
}

