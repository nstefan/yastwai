$ErrorActionPreference = 'Stop'

param(
    [switch] $Quiet,
    [switch] $DryRun,
    [switch] $Help
)

function Show-Usage {
    Write-Host "Usage: pwsh -File scripts/ai-readme.ps1 [--quiet] [--dry-run] [--help]"
}

if ($Help) { Show-Usage; exit 0 }

function Log([string]$msg) { if (-not $Quiet) { Write-Host $msg } }

$ScriptDir = Split-Path -Parent $PSCommandPath
$ProjectRoot = Split-Path -Parent $ScriptDir

Log "Starting README generator..."

function Get-FirstTomlString($path, $key, $fallback) {
    if (-not (Test-Path $path)) { return $fallback }
    $line = Select-String -Path $path -Pattern "^\s*$key\s*=\s*\"(.*)\"" -SimpleMatch:$false -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($line -and $line.Matches[0].Groups[1].Value) { return $line.Matches[0].Groups[1].Value } else { return $fallback }
}

$cargoToml = Join-Path $ProjectRoot 'Cargo.toml'
$projectName = Get-FirstTomlString $cargoToml 'name' 'yastwai'
$projectVersion = Get-FirstTomlString $cargoToml 'version' '0.1.0'
$rustMin = Get-FirstTomlString $cargoToml 'rust-version' '1.85.0'
$license = Get-FirstTomlString $cargoToml 'license' 'MIT'

# Providers from app_config.rs
$providerConfig = Join-Path $ProjectRoot 'src/app_config.rs'
$providersPretty = ''
if (Test-Path $providerConfig) {
    $content = Get-Content $providerConfig -Raw
    $enumStart = [regex]::Match($content, 'pub\s+enum\s+TranslationProvider').Index
    if ($enumStart -ge 0) {
        $after = $content.Substring($enumStart)
        $enumEnd = [regex]::Match($after, "\n}\").Index
        if ($enumEnd -gt 0) {
            $enumBody = $after.Substring(0, $enumEnd)
            $variants = [regex]::Matches($enumBody, '([A-Za-z]+),') | ForEach-Object { $_.Groups[1].Value } | Where-Object { $_ -ne '' }
            $dispStart = [regex]::Match($content, 'pub\s+fn\s+display_name').Index
            if ($dispStart -ge 0) {
                $dispAfter = $content.Substring($dispStart)
                $dispEnd = [regex]::Match($dispAfter, "\n\s*}\").Index
                if ($dispEnd -gt 0) {
                    $dispBody = $dispAfter.Substring(0, $dispEnd)
                    $names = @()
                    foreach ($v in $variants) {
                        $m = [regex]::Match($dispBody, "Self::$v\s*=>\s*\"([^\"]*)\"")
                        if ($m.Success) { $names += $m.Groups[1].Value }
                    }
                    if ($names.Count -gt 0) { $providersPretty = ($names -join ', ') }
                }
            }
        }
    }
}

$exampleConfig = Join-Path $ProjectRoot 'conf.example.json'
if (Test-Path $exampleConfig) { $exampleJson = Get-Content $exampleConfig -Raw } else { $exampleJson = '{
  "source_language": "en",
  "target_language": "fr",
  "translation": { "provider": "ollama" }
}' }

# Escape for fenced code block (keep as-is)
$exampleJsonForMd = $exampleJson

$features = @()
if (Select-String -Path (Join-Path $ProjectRoot 'src') -Pattern 'ffmpeg' -SimpleMatch -Quiet) { $features += '- 🎯 **Extract & Translate** - Pull subtitles from videos and translate in one step' }
if (-not [string]::IsNullOrWhiteSpace($providersPretty)) { $features += "- 🌐 **Multiple AI Providers** - Support for $providersPretty" } else { $features += '- 🌐 **Multiple AI Providers** - Support for various AI translation backends' }
if (Select-String -Path (Join-Path $ProjectRoot 'src') -Pattern 'tokio::spawn|async' -Quiet) { $features += '- ⚡ **Concurrent Processing** - Efficient batch translation for faster results' }
$features += '- 🧠 **Smart Processing** - Preserves formatting and timing of your subtitles'
$features += '- 🔄 **Direct Translation** - Translate existing SRT files without needing video'
$features += '- 📊 **Progress Tracking** - See real-time progress for lengthy translations'

$featuresMd = ($features -join "`n")

$readme = @"
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<div align="center">
  <h1>YASTwAI</h1>
  <p><strong>Y</strong>et <strong>A</strong>nother <strong>S</strong>ub<strong>t</strong>itle translator <strong>w</strong>ith <strong>AI</strong></p>
  <p>
    <a href="#about">About</a> •
    <a href="#key-features">Features</a> •
    <a href="#installation">Installation</a> •
    <a href="#quick-start">Quick Start</a> •
    <a href="#configuration">Configuration</a> •
    <a href="#contributing">Contributing</a> •
    <a href="#license">License</a>
  </p>
</div>

## About

YASTwAI is a command-line tool that extracts subtitles from videos and translates them using AI. Built with Rust for performance, it preserves formatting and timing, and supports multiple translation providers.

## Key Features
$featuresMd

## Installation

### Prerequisites

* Rust and Cargo ($rustMin or newer)
* FFmpeg installed on your system
* GitHub CLI (gh) for pull request operations

### Build from Source

```sh
git clone https://github.com/nstefan/yastwai.git
cd yastwai
cargo build --release
```

## Quick Start

```sh
cp conf.example.json conf.json
./target/release/yastwai video.mkv
./target/release/yastwai videos/
./target/release/yastwai subtitles.srt
./target/release/yastwai -f video.mkv
```

## Configuration

```json
$exampleJsonForMd
```

### Translation Providers

- Ollama (Default, Local)
- OpenAI
- Anthropic

## Contributing

Helper scripts in `scripts/`: `ai-commit`, `ai-update-main`, `ai-pr`, `ai-clippy`

## License

Distributed under the $license License. See `LICENSE` for details.

[contributors-shield]: https://img.shields.io/github/contributors/nstefan/yastwai.svg?style=for-the-badge
[contributors-url]: https://github.com/nstefan/yastwai/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/nstefan/yastwai.svg?style=for-the-badge
[forks-url]: https://github.com/nstefan/yastwai/network/members
[stars-shield]: https://img.shields.io/github/stars/nstefan/yastwai.svg?style=for-the-badge
[stars-url]: https://github.com/nstefan/yastwai/stargazers
[issues-shield]: https://img.shields.io/github/issues/nstefan/yastwai.svg?style=for-the-badge
[issues-url]: https://github.com/nstefan/yastwai/issues
[license-shield]: https://img.shields.io/github/license/nstefan/yastwai.svg?style=for-the-badge
[license-url]: https://github.com/nstefan/yastwai/blob/master/LICENSE
"@

if ($DryRun) {
    Write-Output $readme
    Log "README content generated successfully (dry run)"
} else {
    $out = Join-Path $ProjectRoot 'README.md'
    Set-Content -Path $out -Value $readme -Encoding UTF8
    Log "README.md generated at $out"
}

exit 0


