$ErrorActionPreference = 'Stop'

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-update-main.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  --check-only       Only check for updates without applying them"
    Write-Host "  --rebase-current   Also rebase current branch onto updated main"
    Write-Host "  --help             Display this help message"
}

function Write-Log([string]$Message) {
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$ts] $Message"
}

# Defaults
$CheckOnly = $false
$RebaseCurrent = $false

# Parse args
foreach ($a in $args) {
    switch ($a) {
        '--help' { Show-Usage; exit 1 }
        '--check-only' { $CheckOnly = $true }
        '--rebase-current' { $RebaseCurrent = $true }
        default { Write-Log "Unknown option: $a"; Show-Usage; exit 1 }
    }
}

$currentBranch = (git branch --show-current | Out-String).Trim()
$mainBranch = 'main'
Write-Log "Current branch: $currentBranch"

Write-Log "Fetching latest changes from remote..."
git fetch --all | Out-Host

$localMainRev = (git rev-parse $mainBranch 2>$null | Out-String).Trim()
$remoteMainRev = (git rev-parse origin/$mainBranch 2>$null | Out-String).Trim()

if ($localMainRev -eq $remoteMainRev -and -not [string]::IsNullOrWhiteSpace($localMainRev)) {
    Write-Log "Main branch is already up to date with origin."
    if ($CheckOnly) { exit 0 }
} else {
    Write-Log "Updates available for main branch."
    if ($CheckOnly) {
        Write-Log "New commits available (showing last 5):"
        git log --oneline -n 5 $localMainRev..origin/$mainBranch | Out-Host
        exit 0
    }

    if (-not [string]::IsNullOrWhiteSpace((git status --porcelain | Out-String).Trim())) {
        Write-Log "Error: You have uncommitted changes. Please commit or stash them before updating."
        exit 1
    }

    $returnTo = $currentBranch
    if ($currentBranch -ne $mainBranch) {
        Write-Log "Switching to $mainBranch branch..."
        git checkout $mainBranch 2>$null | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Log "Error: Failed to switch to branch '$mainBranch'"; exit 1 }
    }

    Write-Log "Pulling latest changes with rebase..."
    git pull --rebase origin $mainBranch 2>&1 | Out-Host
    if ($LASTEXITCODE -ne 0) { Write-Log "Error: Failed to update main branch. There might be conflicts."; exit 1 }
    Write-Log "Main branch successfully updated!"

    if ($RebaseCurrent -and $returnTo -ne $mainBranch) {
        Write-Log "Rebasing $returnTo onto updated $mainBranch..."
        git checkout $returnTo 2>$null | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Log "Error: Failed to switch back to '$returnTo'"; exit 1 }
        git rebase $mainBranch 2>&1 | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Log "Error: Rebase conflicts detected. Resolve and run: git rebase --continue"; exit 1 }
        Write-Log "Successfully rebased $returnTo onto updated $mainBranch!"
    } elseif ($returnTo -ne $mainBranch) {
        Write-Log "Switching back to $returnTo..."
        git checkout $returnTo 2>$null | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Log "Error: Failed to switch back to '$returnTo'"; exit 1 }
    }
}

Write-Log "Current git status:"
git status | Out-Host
exit 0


