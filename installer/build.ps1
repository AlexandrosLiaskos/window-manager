# Window Manager Installer Build Script
# Requires: Inno Setup 6.x installed

param(
    [switch]$SkipBuild,
    [switch]$Help
)

if ($Help) {
    Write-Host @"
Window Manager Installer Build Script

Usage: .\build.ps1 [-SkipBuild] [-Help]

Options:
  -SkipBuild    Skip cargo build, use existing exe
  -Help         Show this help message
"@
    exit 0
}

$ErrorActionPreference = "Stop"

# Get project root (parent of installer folder)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host "=== Window Manager Installer Build ===" -ForegroundColor Cyan
Write-Host "Project root: $ProjectRoot"
Write-Host ""

# Check for Inno Setup
$InnoPath = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $InnoPath) {
    Write-Host "ERROR: Inno Setup 6 not found!" -ForegroundColor Red
    Write-Host "Please install from: https://jrsoftware.org/isdl.php"
    exit 1
}

Write-Host "Found Inno Setup: $InnoPath" -ForegroundColor Green

# Check for icon
$IconPath = Join-Path $ProjectRoot "assets\icon.ico"
$SetupIss = Join-Path $ScriptDir "setup.iss"

if (-not (Test-Path $IconPath)) {
    Write-Host ""
    Write-Host "WARNING: No icon found at $IconPath" -ForegroundColor Yellow
    Write-Host "Run assets\create-icon.ps1 first, or installer will have no custom icon."
    Write-Host ""
} else {
    Write-Host "Found icon: $IconPath" -ForegroundColor Green
}

# Build release
if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "Building release..." -ForegroundColor Cyan
    Push-Location $ProjectRoot
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed!" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Pop-Location
    Write-Host "Build complete!" -ForegroundColor Green
}

# Check exe exists
$ExePath = Join-Path $ProjectRoot "target\release\window-manager.exe"
if (-not (Test-Path $ExePath)) {
    Write-Host "ERROR: Executable not found at $ExePath" -ForegroundColor Red
    exit 1
}

Write-Host "Found executable: $ExePath" -ForegroundColor Green

# Create output directory
$OutputDir = Join-Path $ProjectRoot "target\installer"
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}

# Run Inno Setup
Write-Host ""
Write-Host "Creating installer..." -ForegroundColor Cyan

& $InnoPath $SetupIss
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Installer creation failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== SUCCESS ===" -ForegroundColor Green
Write-Host "Installer created at: $OutputDir"
Write-Host ""

# List created files
Get-ChildItem $OutputDir -Filter "*.exe" | ForEach-Object {
    Write-Host "  $($_.Name) ($([math]::Round($_.Length / 1MB, 2)) MB)"
}
