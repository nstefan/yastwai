param(
    [Parameter(Mandatory = $true)] [string] $Model,
    [Parameter(Mandatory = $true, Position = 0)] [string] $Title,
    [Parameter(Mandatory = $true, Position = 1)] [string] $Description,
    [Parameter(Mandatory = $true, Position = 2)] [string] $Prompt,
    [Parameter(Position = 3)] [string] $ThoughtProcess,
    [Parameter(Position = 4)] [string] $Discussion,
    [switch] $Help
)

$ErrorActionPreference = 'Stop'

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-commit.ps1 -Model \"model-name\" \"Title\" \"Description\" \"Prompt\" [\"Thought Process\"] [\"Discussion\"]"
}

if ($Help) { Show-Usage; exit 1 }

function Write-Log([string]$Message) {
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$ts] $Message"
}

if (-not $Title -or -not $Description -or -not $Prompt -or -not $Model) {
    Write-Log "Error: Missing required arguments."
    Show-Usage
    exit 1
}

$tempFile = New-TemporaryFile
Set-Content -Path $tempFile -Value $Title -Encoding UTF8
Add-Content -Path $tempFile -Value ""
Add-Content -Path $tempFile -Value "Short description: $Description"
Add-Content -Path $tempFile -Value ""
Add-Content -Path $tempFile -Value "Model: $Model"
Add-Content -Path $tempFile -Value ""
Add-Content -Path $tempFile -Value "Prompt: $Prompt"
Add-Content -Path $tempFile -Value ""
if ($ThoughtProcess) {
    Add-Content -Path $tempFile -Value "Thought Process: "
    Add-Content -Path $tempFile -Value $ThoughtProcess
    Add-Content -Path $tempFile -Value ""
}
if ($Discussion) {
    Add-Content -Path $tempFile -Value "Discussion: "
    Add-Content -Path $tempFile -Value $Discussion
}

Write-Log "Generated commit message:"
Write-Log "---------------------------------------------"
Get-Content $tempFile | ForEach-Object { Write-Host "    $_" }
Write-Log "---------------------------------------------"

Write-Log "Staging all changes..."
git add -A | Out-Host

git commit -F $tempFile | Out-Host
if ($LASTEXITCODE -ne 0) {
    Write-Log "Commit failed."
    Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
    exit $LASTEXITCODE
}

Write-Log "Commit created successfully!"
Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
exit 0


