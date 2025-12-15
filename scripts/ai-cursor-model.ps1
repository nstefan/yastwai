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

# Function to extract model from database content
function Get-ModelFromDbContent {
    param([string]$dbPath)
    
    try {
        Log "Reading database file: $dbPath"
        $bytes = [System.IO.File]::ReadAllBytes($dbPath)
        $str = [System.Text.Encoding]::UTF8.GetString($bytes)
        
        # Try modelConfig.modelName pattern (most reliable for current model)
        $matches = [regex]::Matches($str, '"modelConfig"\s*:\s*\{\s*"modelName"\s*:\s*"([^"]+)"')
        if ($matches.Count -gt 0) {
            $model = $matches[-1].Groups[1].Value
            Log "Found model from modelConfig: $model"
            return $model
        }
        
        # Try modelInfo.modelName pattern
        $matches = [regex]::Matches($str, '"modelInfo"\s*:\s*\{\s*"modelName"\s*:\s*"([^"]+)"')
        if ($matches.Count -gt 0) {
            $model = $matches[-1].Groups[1].Value
            Log "Found model from modelInfo: $model"
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
    } catch {
        Log "Error reading database: $_"
        return $null
    }
}

# Try to read Cursor database
$dbPaths = @()

# Windows paths
if ($env:APPDATA) {
    $dbPaths += Join-Path $env:APPDATA 'Cursor\User\globalStorage\state.vscdb.backup'
    $dbPaths += Join-Path $env:APPDATA 'Cursor\User\globalStorage\state.vscdb'
}

# macOS paths
if ($env:HOME) {
    $dbPaths += Join-Path $env:HOME 'Library/Application Support/Cursor/User/globalStorage/state.vscdb.backup'
    $dbPaths += Join-Path $env:HOME 'Library/Application Support/Cursor/User/globalStorage/state.vscdb'
}

# Linux paths
if ($env:HOME) {
    $dbPaths += Join-Path $env:HOME '.config/Cursor/User/globalStorage/state.vscdb.backup'
    $dbPaths += Join-Path $env:HOME '.config/Cursor/User/globalStorage/state.vscdb'
}

foreach ($dbPath in $dbPaths) {
    if (Test-Path $dbPath -PathType Leaf -ErrorAction SilentlyContinue) {
        Log "Found database at: $dbPath"
        $model = Get-ModelFromDbContent -dbPath $dbPath
        if ($model) {
            Write-Output $model
            exit 0
        }
    }
}

Log "Could not detect model from any source"
Write-Output 'N/A'
exit 0
