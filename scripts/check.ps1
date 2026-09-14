# Runs the same checks as CI (.github/workflows/ci.yml) on this machine.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\check.ps1
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Invoke-Step {
    param([string]$Name, [string]$Directory, [string]$Command, [string[]]$Arguments)
    Write-Host ""
    Write-Host "==> $Name"
    Push-Location $Directory
    try {
        & $Command @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "$Name failed (exit code $LASTEXITCODE)"
        }
    }
    finally {
        Pop-Location
    }
}

$ui = Join-Path $root 'ui'

if (-not (Test-Path (Join-Path $ui 'node_modules'))) {
    Invoke-Step 'npm ci (ui)' $ui 'npm' @('ci')
}

Invoke-Step 'svelte-check (ui)' $ui 'npm' @('run', 'check')

Invoke-Step 'interface tests (ui)' $ui 'npm' @('test')

# The Tauri context embeds the built interface, so the Rust checks need it.
Invoke-Step 'build interface (ui)' $ui 'npm' @('run', 'build')

Invoke-Step 'cargo fmt --check' $root 'cargo' @('fmt', '--all', '--', '--check')
Invoke-Step 'cargo clippy' $root 'cargo' @('clippy', '--workspace', '--all-targets', '--', '-D', 'warnings')
Invoke-Step 'cargo test' $root 'cargo' @('test', '--workspace')

Write-Host ""
Write-Host "All checks passed."
