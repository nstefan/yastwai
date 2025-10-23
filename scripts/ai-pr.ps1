$ErrorActionPreference = 'Stop'

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-pr.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  --title TITLE         PR title (required)"
    Write-Host "  --overview TEXT       Brief overview (required)"
    Write-Host "  --key-changes TEXT    Comma-separated list of key changes"
    Write-Host "  --implementation TEXT Comma-separated implementation details"
    Write-Host "  --base BRANCH         Base branch (default: main)"
    Write-Host "  --draft               Create PR as draft"
    Write-Host "  --model MODEL         Technical model name (required)"
    Write-Host "  --no-browser          Do not open browser after creation"
    Write-Host "  --help                Display this help message"
}

function Write-Log([string]$Message) {
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$ts] $Message"
}

# Defaults
$PRTitle = ''
$Overview = ''
$KeyChanges = ''
$Implementation = ''
$Draft = $false
$Model = ''
$BaseBranch = 'main'
$OpenBrowser = $true

# Parse args
for ($i = 0; $i -lt $args.Length; $i++) {
    switch -Regex ($args[$i]) {
        '^--help$' { Show-Usage; exit 1 }
        '^--title$' { if ($i+1 -ge $args.Length -or $args[$i+1].StartsWith('--')) { Show-Usage; exit 1 } $PRTitle = $args[++$i] }
        '^--overview$' { if ($i+1 -ge $args.Length -or $args[$i+1].StartsWith('--')) { Show-Usage; exit 1 } $Overview = $args[++$i] }
        '^--key-changes$' { if ($i+1 -ge $args.Length -or $args[$i+1].StartsWith('--')) { Show-Usage; exit 1 } $KeyChanges = $args[++$i] }
        '^--implementation$' { if ($i+1 -ge $args.Length -or $args[$i+1].StartsWith('--')) { Show-Usage; exit 1 } $Implementation = $args[++$i] }
        '^--base$' { if ($i+1 -ge $args.Length -or $args[$i+1].StartsWith('--')) { Show-Usage; exit 1 } $BaseBranch = $args[++$i] }
        '^--model$' { if ($i+1 -ge $args.Length -or $args[$i+1].StartsWith('--')) { Show-Usage; exit 1 } $Model = $args[++$i] }
        '^--draft$' { $Draft = $true }
        '^--no-browser$' { $OpenBrowser = $false }
        default { Write-Log "Unknown option: $($args[$i])"; Show-Usage; exit 1 }
    }
}

if ([string]::IsNullOrWhiteSpace($PRTitle)) { Write-Log "Error: PR title is required"; Show-Usage; exit 1 }
if ([string]::IsNullOrWhiteSpace($Overview)) { Write-Log "Error: Overview is required"; Show-Usage; exit 1 }
if ([string]::IsNullOrWhiteSpace($Model)) { Write-Log "Error: Model parameter is required"; Show-Usage; exit 1 }

# Check gh CLI
if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Log "Error: GitHub CLI (gh) is not installed. See https://github.com/cli/cli#installation"
    exit 1
}
try { gh auth status *> $null } catch { Write-Log "Error: GitHub CLI (gh) is not authenticated. Run: gh auth login"; exit 1 }

# Create PR description file
$prBody = New-TemporaryFile

# Build description
Add-Content $prBody "## 📌 Overview"
Add-Content $prBody $Overview
Add-Content $prBody ""

Add-Content $prBody "## 🔍 Key Changes"
if (-not [string]::IsNullOrWhiteSpace($KeyChanges)) {
    foreach ($c in $KeyChanges.Split(',')) { Add-Content $prBody ("- " + $c.Trim()) }
} else { Add-Content $prBody "<!-- No key changes specified -->" }
Add-Content $prBody ""

if (-not [string]::IsNullOrWhiteSpace($Implementation)) {
    Add-Content $prBody "## 🧩 Implementation Details"
    foreach ($d in $Implementation.Split(',')) { Add-Content $prBody ("- " + $d.Trim()) }
    Add-Content $prBody ""
}

Add-Content $prBody "## 🤖 AI Model"
Add-Content $prBody $Model

Write-Log "Generated PR description:"
Write-Log "---------------------------------------------"
Get-Content $prBody | Out-Host
Write-Log "---------------------------------------------"

$currentBranch = (git branch --show-current 2>$null | Out-String).Trim()
if ([string]::IsNullOrWhiteSpace($currentBranch)) { Write-Log "Error: Not on any branch"; exit 1 }

if (-not [string]::IsNullOrWhiteSpace((git status --porcelain 2>$null | Out-String).Trim())) {
    Write-Log "Error: You have uncommitted changes. Please commit or stash before creating a PR."
    exit 1
}

Write-Log "Current branch: $currentBranch"
Write-Log "Base branch: $BaseBranch"

function Safe-Push {
    $attempts = 0
    $max = 3
    while ($attempts -lt $max) {
        git push -u origin "$currentBranch" 2>$null | Out-Host
        if ($LASTEXITCODE -eq 0) { return $true }
        $attempts++
        if ($attempts -lt $max) { Write-Log "Push failed. Retrying in 2 seconds... ($attempts/$max)"; Start-Sleep -Seconds 2 }
    }
    return $false
}

$remoteExists = (git ls-remote --heads origin "$currentBranch" 2>$null | Out-String).Trim()
if ([string]::IsNullOrWhiteSpace($remoteExists)) {
    Write-Log "Remote branch does not exist. Pushing changes..."
    if (-not (Safe-Push)) { Remove-Item $prBody -Force; exit 1 }
} else {
    $behind = [int](git rev-list --count "$currentBranch..origin/$currentBranch" 2>$null | Out-String)
    $ahead  = [int](git rev-list --count "origin/$currentBranch..$currentBranch" 2>$null | Out-String)
    if ($behind -gt 0) {
        Write-Log "Your branch is behind remote by $behind commit(s). Attempting rebase..."
        git pull --rebase origin "$currentBranch" 2>$null | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Log "Error: Automatic rebase failed. Resolve manually."; Remove-Item $prBody -Force; exit 1 }
    }
    if ($ahead -gt 0) {
        Write-Log "Your branch is ahead by $ahead commit(s). Pushing changes..."
        if (-not (Safe-Push)) { Remove-Item $prBody -Force; exit 1 }
    } else {
        Write-Log "Branch is up to date with remote."
    }
}

if ([string]::IsNullOrWhiteSpace($BaseBranch)) { $BaseBranch = 'main'; Write-Log "Base branch empty; using default: $BaseBranch" }
$commitCount = [int](git rev-list --count "$BaseBranch..$currentBranch" 2>$null | Out-String)
Write-Log "Found $commitCount commits between $BaseBranch and $currentBranch"
if ($commitCount -eq 0) { Write-Log "Warning: No commits found between $BaseBranch and $currentBranch" }

Write-Log "Creating PR using GitHub CLI (gh)..."
$ghArgs = @('pr','create','--title', $PRTitle, '--body-file', $prBody, '--base', $BaseBranch)
if ($Draft) { $ghArgs += '--draft' }

Write-Log ("Executing: gh " + ($ghArgs -join ' '))
$prUrl = & gh @ghArgs
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace(($prUrl | Out-String).Trim())) {
    Write-Log "Error: Failed to create pull request"
    Remove-Item $prBody -Force
    exit 1
}
Write-Log ("Successfully created PR: " + ($prUrl | Out-String).Trim())

if ($OpenBrowser) {
    try { Start-Process $prUrl | Out-Null } catch { Write-Log "Could not open browser automatically" }
}

Remove-Item $prBody -Force
Write-Log "PR creation process completed successfully."
exit 0


