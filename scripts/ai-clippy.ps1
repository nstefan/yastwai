$ErrorActionPreference = 'Stop'

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-clippy.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  --check-only        Only run checks (default if no flags)"
    Write-Host "  --fix               Run cargo fix and clippy --fix"
    Write-Host "  --verbose           Show detailed output"
    Write-Host "  --help              Display this help message"
}

# Defaults
$CheckOnly = $false
$Fix = $false
$Verbose = $false

# Parse args
foreach ($a in $args) {
    switch ($a) {
        '--help' { Show-Usage; exit 1 }
        '--check-only' { $CheckOnly = $true }
        '--fix' { $Fix = $true }
        '--verbose' { $Verbose = $true }
        default { Write-Host "Unknown option: $a"; Show-Usage; exit 1 }
    }
}

if (-not $CheckOnly -and -not $Fix) {
    $CheckOnly = $true
    Write-Host "No specific mode selected, defaulting to check-only mode"
}

if ($CheckOnly) {
    Write-Host "Running Clippy checks..."
    $lints = "-A clippy::uninlined_format_args -A clippy::redundant_closure_for_method_calls"
    if ($Verbose) { Write-Host "Using lint exceptions: $lints" }
    & cargo clippy -- -D warnings $lints
    if ($LASTEXITCODE -ne 0) { Write-Host "Clippy check failed with exit code $LASTEXITCODE"; exit $LASTEXITCODE } else { Write-Host "Clippy check passed successfully." }
}

if ($Fix) {
    Write-Host "Running Clippy auto-fix..."
    & cargo fix --allow-dirty --allow-staged
    if ($LASTEXITCODE -ne 0) { Write-Host "Clippy auto-fix failed with exit code $LASTEXITCODE"; exit $LASTEXITCODE } else { Write-Host "Clippy auto-fix completed successfully." }

    Write-Host "Running additional clippy fixes..."
    & cargo clippy --fix --allow-dirty --allow-staged
    if ($LASTEXITCODE -ne 0) { Write-Host "Additional clippy fixes completed with warnings, code: $LASTEXITCODE" } else { Write-Host "Additional clippy fixes completed successfully." }
}

Write-Host "Clippy process completed."
exit 0


