$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

[CmdletBinding()]
param(
  # If set, skips `npm install` even if `node_modules` is missing.
  [switch]$SkipInstall,
  # If set, skips `cargo clippy` and `cargo test`.
  [switch]$SkipRustChecks,
  # If set, skips `tsc --noEmit`.
  [switch]$SkipTypeScriptCheck
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

Write-Host "=== Deterrence — Dev Build (Windows) ==="

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

# 3. TypeScript check
if (-not $SkipTypeScriptCheck) {
  Invoke-External npx tsc --noEmit
}

# 4. Launch dev app (Tauri handles frontend dev server + Rust build)
Invoke-External npx tauri dev

