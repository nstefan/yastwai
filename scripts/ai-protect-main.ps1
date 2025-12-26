$ErrorActionPreference = 'Stop'

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-protect-main.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  --auto-branch NAME   Automatically create a feature branch if on main"
    Write-Host "  --no-auto-branch     Only check; exit with error if on main"
    Write-Host "  --help               Display this help message"
}

function Write-Log([string]$Message) {
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$ts] $Message"
}

# Parse arguments
$AutoBranch = $null
$NoAutoBranch = $false
for ($i = 0; $i -lt $args.Length; $i++) {
    switch -Regex ($args[$i]) {
        '^--help$' { Show-Usage; exit 1 }
        '^--no-auto-branch$' { $NoAutoBranch = $true }
        '^--auto-branch$' {
            if ($i + 1 -ge $args.Length -or $args[$i+1].StartsWith('--')) {
                Write-Log "Error: --auto-branch requires a name parameter"; Show-Usage; exit 1
            }
            $AutoBranch = $args[$i+1]; $i++
        }
        default { Write-Log "Error: Unknown option: $($args[$i])"; Show-Usage; exit 1 }
    }
}

# Determine current branch
$currentBranch = (git branch --show-current).Trim()
Write-Log "Current branch: $currentBranch"

if ($currentBranch -eq 'main') {
    Write-Log "WARNING: Currently on main branch. Direct work on main is prohibited."

    if ($NoAutoBranch) {
        Write-Log "CRITICAL: Create a feature branch immediately before proceeding!"
        Write-Log "Run: pwsh -File scripts/ai-branch.ps1 --new-branch 'feature-name' --is-related false"
        exit 1
    }

    if ([string]::IsNullOrWhiteSpace($AutoBranch)) {
        Write-Log "ERROR: Must provide either --auto-branch NAME or --no-auto-branch"
        Show-Usage
        exit 1
    }

    Write-Log "PROTECTION: Automatically creating feature branch: $AutoBranch"

    $hasChanges = -not [string]::IsNullOrWhiteSpace((git status --porcelain))
    if ($hasChanges) {
        Write-Log "Uncommitted changes detected. Stashing before creating branch..."
        git stash -u | Out-Host
    }

    try {
        & "$PSScriptRoot/ai-branch.ps1" --new-branch "$AutoBranch" --is-related false | Out-Host
    } catch {
        Write-Log "Failed to create branch. Please create a feature branch manually."
        if ($hasChanges) { git stash pop | Out-Host }
        exit 1
    }

    if ($hasChanges) {
        Write-Log "Applying stashed changes to new branch..."
        git stash pop | Out-Host
    }

    Write-Log ("Now working on branch: " + (git branch --show-current).Trim())
} else {
    Write-Log "OK: Working on branch '$currentBranch' (not main) - Proceeding safely"
}

exit 0


