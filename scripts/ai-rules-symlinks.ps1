$ErrorActionPreference = 'Stop'

function Write-Color($text, $color) { Write-Host $text -ForegroundColor $color }

$SourceDir = 'docs/agentrules'
$TargetDir = '.cursor/rules'
$WorkspaceRoot = Get-Location

Write-Color '=======================================================' Green
Write-Color '      AI Rules Symlinks Generator                     ' Green
Write-Color '=======================================================' Green

if (-not (Test-Path $SourceDir -PathType Container)) {
    Write-Color "Error: Source directory '$SourceDir' does not exist." Red
    exit 1
}

if (-not (Test-Path $TargetDir -PathType Container)) {
    Write-Color "Target directory '$TargetDir' does not exist. Creating it..." Yellow
    New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
    Write-Color "✓ Created target directory '$TargetDir'." Green
} else {
    Write-Color "✓ Target directory '$TargetDir' exists." Green
}

Write-Color "Removing existing .mdc files in '$TargetDir'..." Yellow
Get-ChildItem -Path $TargetDir -Filter *.mdc -Force -ErrorAction SilentlyContinue | ForEach-Object {
    Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue
    Write-Color "  Removed: $($_.FullName)" Yellow
}

Write-Color 'Creating symbolic links...' Green
$created = 0

Get-ChildItem -Path $SourceDir -Filter '*_mdc.txt' -File -Recurse | ForEach-Object {
    $fileName = $_.Name
    if ($fileName -notmatch '(.+)_mdc\.txt$') { return }
    $base = $matches[1]
    $target = Join-Path $TargetDir "$base.mdc"
    $src = $_.FullName
    # Create symlink; on Windows, requires admin or Developer Mode. Fallback: create a small file with a path note.
    try {
        New-Item -ItemType SymbolicLink -Path $target -Target $src -Force | Out-Null
        Write-Color "  ✓ Created link: $target -> $src" Green
        $created++
    } catch {
        # Fallback: copy content as a regular file to at least expose the rules
        Copy-Item -Path $src -Destination $target -Force
        Write-Color "  ⚠ Fallback copy: $target (symlink failed)" Yellow
        $created++
    }
}

if ($created -gt 0) {
    Write-Color "✓ Successfully created $created rule link(s)." Green
} else {
    Write-Color "! No matching *_mdc.txt files found in $SourceDir." Yellow
}

exit 0


