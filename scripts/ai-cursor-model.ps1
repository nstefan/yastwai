param(
    [switch] $Verbose,
    [switch] $Quiet,
    [switch] $Help
)

$ErrorActionPreference = 'Stop'

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-cursor-model.ps1 [--verbose|-v] [--quiet|-q] [--help|-h]"
}

if ($Help) { Show-Usage; exit 1 }

function Log([string]$msg) { if ($Verbose -and -not $Quiet) { $ts = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'; Write-Host "[$ts] $msg" } }

# Environment variables
foreach ($var in @('CURSOR_CURRENT_MODEL','AI_CURSOR_MODEL','AI_MODEL','MODEL_NAME')) {
    $val = [Environment]::GetEnvironmentVariable($var)
    if (-not [string]::IsNullOrWhiteSpace($val)) { Log "Found model from environment ${var}: $val"; Write-Output $val; exit 0 }
}

# Function to read file bytes, handling locked files with FileShare.ReadWrite
function Read-FileBytesSafe {
    param([string]$path)
    
    try {
        # Try reading with ReadWrite sharing to handle locked files (e.g., Cursor has DB open)
        $fs = [System.IO.File]::Open($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
        try {
            $bytes = New-Object byte[] $fs.Length
            $fs.Read($bytes, 0, $fs.Length) | Out-Null
            return $bytes
        } finally {
            $fs.Close()
        }
    } catch {
        Log "Could not read file with sharing: $_"
        return $null
    }
}

# Function to extract model from database content
function Get-ModelFromDbContent {
    param([string]$dbPath)
    
    Log "Reading database file: $dbPath"
    $bytes = Read-FileBytesSafe -path $dbPath
    if (-not $bytes) { return $null }
    
    $str = [System.Text.Encoding]::UTF8.GetString($bytes)
    
    # Prioritize matches from the end of the file (most recent data)
    # Check last 2MB of file for most recent modelConfig entries
    $searchLength = [Math]::Min(2000000, $str.Length)
    $lastChunk = $str.Substring([Math]::Max(0, $str.Length - $searchLength))
    
    # Try modelConfig.modelName pattern in the last chunk (most reliable for current model)
    $matches = [regex]::Matches($lastChunk, '"modelConfig"\s*:\s*\{\s*"modelName"\s*:\s*"([^"]+)"')
    if ($matches.Count -gt 0) {
        # Take the last match from the chunk (most recent)
        $model = $matches[-1].Groups[1].Value
        Log "Found model from modelConfig (last chunk): $model"
        return $model
    }
    
    # Fallback: search entire file for modelConfig
    $matches = [regex]::Matches($str, '"modelConfig"\s*:\s*\{\s*"modelName"\s*:\s*"([^"]+)"')
    if ($matches.Count -gt 0) {
        $model = $matches[-1].Groups[1].Value
        Log "Found model from modelConfig (full file): $model"
        return $model
    }
    
    # Try modelInfo.modelName pattern in last chunk
    $matches = [regex]::Matches($lastChunk, '"modelInfo"\s*:\s*\{\s*"modelName"\s*:\s*"([^"]+)"')
    if ($matches.Count -gt 0) {
        $model = $matches[-1].Groups[1].Value
        Log "Found model from modelInfo (last chunk): $model"
        return $model
    }
    
    # Fallback: look for defaultModel pattern
    $matches = [regex]::Matches($str, '"defaultModel"\s*:\s*"([^"]+)"')
    if ($matches.Count -gt 0) {
        $model = $matches[-1].Groups[1].Value
        Log "Found model from defaultModel: $model"
        return $model
    }
    
    Log "No model pattern found in database"
    return $null
}

# Try to read Cursor database - prefer live DB over backup for real-time data
$dbPaths = @()

# Windows paths - live DB first, then backup
if ($env:APPDATA) {
    $dbPaths += Join-Path $env:APPDATA 'Cursor\User\globalStorage\state.vscdb'
    $dbPaths += Join-Path $env:APPDATA 'Cursor\User\globalStorage\state.vscdb.backup'
}

# macOS paths
if ($env:HOME) {
    $dbPaths += Join-Path $env:HOME 'Library/Application Support/Cursor/User/globalStorage/state.vscdb'
    $dbPaths += Join-Path $env:HOME 'Library/Application Support/Cursor/User/globalStorage/state.vscdb.backup'
}

# Linux paths
if ($env:HOME) {
    $dbPaths += Join-Path $env:HOME '.config/Cursor/User/globalStorage/state.vscdb'
    $dbPaths += Join-Path $env:HOME '.config/Cursor/User/globalStorage/state.vscdb.backup'
}

# Collect all available database files with their modification times
$availableDbs = @()
foreach ($dbPath in $dbPaths) {
    if (Test-Path $dbPath -PathType Leaf -ErrorAction SilentlyContinue) {
        $lastWrite = (Get-Item $dbPath).LastWriteTime
        $availableDbs += [PSCustomObject]@{ Path = $dbPath; LastWrite = $lastWrite }
    }
}

# Sort by modification time (most recent first) to prioritize freshest data
$availableDbs = $availableDbs | Sort-Object LastWrite -Descending

foreach ($db in $availableDbs) {
    Log "Checking database at: $($db.Path) (modified: $($db.LastWrite))"
    $model = Get-ModelFromDbContent -dbPath $db.Path
    if ($model) {
        Write-Output $model
        exit 0
    }
}

Log "Could not detect model from any source"
Write-Output 'N/A'
exit 0
