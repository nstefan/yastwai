$ErrorActionPreference = 'Stop'

param(
    [switch] $Verbose,
    [switch] $Quiet,
    [switch] $Help
)

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-cursor-model.ps1 [--verbose|-v] [--quiet|-q] [--help|-h]"
}

if ($Help) { Show-Usage; exit 1 }

function Log([string]$msg) { if ($Verbose -and -not $Quiet) { $ts = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'; Write-Host "[$ts] $msg" } }

# Environment variables
foreach ($var in @('CURSOR_CURRENT_MODEL','AI_CURSOR_MODEL','AI_MODEL','MODEL_NAME')) {
    $val = [Environment]::GetEnvironmentVariable($var)
    if (-not [string]::IsNullOrWhiteSpace($val)) { Log "Found model from environment $var: $val"; Write-Output $val; exit 0 }
}

# Try to read Cursor database on Windows
try {
    $dbPath = Join-Path $env:APPDATA 'Cursor\User\globalStorage\state.vscdb'
    if (-not (Test-Path $dbPath)) {
        # macOS path for completeness if running on WSL
        $macDb = Join-Path $env:HOME 'Library/Application Support/Cursor/User/globalStorage/state.vscdb'
        if (Test-Path $macDb) { $dbPath = $macDb }
    }
    if (Test-Path $dbPath -PathType Leaf -ErrorAction SilentlyContinue) {
        if (Get-Command sqlite3 -ErrorAction SilentlyContinue) {
            Log "Querying Cursor database for model information at $dbPath"
            $hex = sqlite3 -readonly "$dbPath" "PRAGMA query_only=ON; SELECT hex(value) FROM cursorDiskKV WHERE key GLOB 'composerData:*' ORDER BY rowid DESC LIMIT 10;"
            if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($hex)) {
                # Convert hex to bytes then to string
                $bytes = for ($i=0; $i -lt $hex.Length; $i+=2) { [Convert]::ToByte($hex.Substring($i,2),16) }
                $str = [System.Text.Encoding]::UTF8.GetString($bytes)
                $m = [regex]::Match($str, '"modelName":"([^"]+)"')
                if ($m.Success) { Write-Output $m.Groups[1].Value; exit 0 }
                $m = [regex]::Match($str, '"composerModel":"([^"]+)"')
                if ($m.Success) { Write-Output $m.Groups[1].Value; exit 0 }
            }
        } else { Log 'sqlite3 not available' }
    } else { Log "Cursor database not found" }
} catch { Log "Error reading Cursor DB: $_" }

Write-Output 'N/A'
exit 0


