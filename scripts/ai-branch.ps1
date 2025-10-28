$ErrorActionPreference = 'Stop'

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-branch.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  --check-only          Only check current branch status"
    Write-Host "  --new-branch NAME     Create a new branch with the specified name from main"
    Write-Host "  --is-related BOOL     Specify if new work is related to current branch (true/false)"
    Write-Host "  --force               Force branch creation even with uncommitted changes"
    Write-Host "  --help                Display this help message"
}

function Write-Log([string]$Message) {
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$ts] $Message"
}

# Defaults
$CheckOnly = $false
$NewBranch = $null
$IsRelated = 'true'
$Force = $false

# Parse args
for ($i = 0; $i -lt $args.Length; $i++) {
    switch -Regex ($args[$i]) {
        '^--help$' { Show-Usage; exit 1 }
        '^--check-only$' { $CheckOnly = $true }
        '^--new-branch$' {
            if ($i + 1 -ge $args.Length -or $args[$i+1].StartsWith('--')) { Write-Log "Error: --new-branch requires a name parameter"; Show-Usage; exit 1 }
            $NewBranch = $args[$i+1]; $i++
        }
        '^--is-related$' {
            if ($i + 1 -ge $args.Length -or ($args[$i+1] -notin @('true','false'))) { Write-Log "Error: --is-related requires 'true' or 'false'"; Show-Usage; exit 1 }
            $IsRelated = $args[$i+1]; $i++
        }
        '^--force$' { $Force = $true }
        default { Write-Log "Unknown option: $($args[$i])"; Show-Usage; exit 1 }
    }
}

$currentBranch = (git branch --show-current | Out-String).Trim()
$mainBranch = 'main'
Write-Log "Current branch: $currentBranch"

if ($currentBranch -eq $mainBranch) {
    Write-Log "Currently on main branch."
    if ($CheckOnly) { Write-Log "Check-only mode: Not creating a new branch."; exit 0 }
    if ([string]::IsNullOrWhiteSpace($NewBranch)) { Write-Log "Error: On main branch but no --new-branch name provided."; exit 1 }
    git checkout -b "$NewBranch" 2>$null | Out-Host
    if ($LASTEXITCODE -ne 0) { Write-Log "Error: Failed to create branch '$NewBranch'"; exit 1 }
    Write-Log "Created and switched to new branch: $NewBranch"
} else {
    Write-Log "Currently on branch: $currentBranch"
    if ($CheckOnly) { Write-Log "Check-only mode: Not creating a new branch."; exit 0 }
    $isRelatedBool = ($IsRelated -eq 'true')
    if ($isRelatedBool) {
        Write-Log "Work is related to current branch. Continuing on: $currentBranch"
    } else {
        Write-Log "Work not related to current branch. Need to create a new branch from main."
        $hasChanges = -not [string]::IsNullOrWhiteSpace((git status --porcelain | Out-String).Trim())
        if ($hasChanges -and -not $Force) {
            Write-Log "Error: You have uncommitted changes. Use --force to override or commit/stash first."
            exit 1
        }
        if ([string]::IsNullOrWhiteSpace($NewBranch)) { Write-Log "Error: No new branch name provided with --new-branch."; exit 1 }
        git checkout "$mainBranch" 2>$null | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Log "Error: Failed to switch to branch '$mainBranch'"; exit 1 }
        git pull | Out-Host
        git checkout -b "$NewBranch" 2>$null | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Log "Error: Failed to create branch '$NewBranch'"; exit 1 }
        Write-Log "Created and switched to new branch: $NewBranch"
    }
}

Write-Log "Current git status:"
git status | Out-Host
exit 0


