# PPM Installation Script for Windows
# Usage: Invoke-WebRequest https://raw.githubusercontent.com/VesperAkshay/polypm/main/install.ps1 -UseBasicParsing | Invoke-Expression

param(
    [string]$Version = "latest",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

function Install-PPMBinary {
    Write-Host "ℹ️ Attempting to install pre-compiled binary..." -ForegroundColor Blue
    
    try {
        $arch = $env:PROCESSOR_ARCHITECTURE
        if ($arch -eq "AMD64") {
            $arch = "x86_64"
        } else {
            Write-Host "⚠️ Unsupported architecture: $arch" -ForegroundColor Yellow
            return $false
        }
        
        $assetName = "ppm-windows-$arch.exe"
        $downloadUrl = "https://github.com/VesperAkshay/polypm/releases/latest/download/$assetName"
        $installPath = "$env:USERPROFILE\.cargo\bin\ppm.exe"
        
        # Create install directory
        $installDir = Split-Path $installPath -Parent
        if (-not (Test-Path $installDir)) {
            New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        }
        
        Write-Host "ℹ️ Downloading from: $downloadUrl" -ForegroundColor Blue
        
        # Download binary
        Invoke-WebRequest -Uri $downloadUrl -OutFile $installPath -UseBasicParsing
        
        Write-Host "✅ PPM binary installed successfully!" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "⚠️ Failed to download pre-compiled binary: $_" -ForegroundColor Yellow
        return $false
    }
}

Write-Host @"

╔═══════════════════════════════════════╗
║        PPM Installer for Windows      ║
║    Polyglot Package Manager v1.0.0    ║
╚═══════════════════════════════════════╝

"@ -ForegroundColor Cyan

Write-Host "ℹ️ Starting PPM installation..." -ForegroundColor Blue

# Check if already installed
if ((Test-Command "ppm") -and (-not $Force)) {
    $existingVersion = & ppm --version 2>$null
    Write-Host "⚠️ PPM is already installed: $existingVersion" -ForegroundColor Yellow
    Write-Host "ℹ️ Use -Force parameter to reinstall" -ForegroundColor Blue
    exit 0
}

# Check for Cargo and try installation
Write-Host "ℹ️ Checking installation options..." -ForegroundColor Blue

# Try binary installation first (no Rust required)
if (Install-PPMBinary) {
    # Verify installation
    if (Test-Command "ppm") {
        $version = & ppm --version 2>$null
        Write-Host "✅ PPM $version is installed and working!" -ForegroundColor Green
    }
    
    Write-Host @"

🎉 Installation Complete!

Quick Start:
  ppm --help                    # Show help
  ppm init --name my-project    # Create new project
  ppm add react flask           # Add dependencies
  ppm install                   # Install dependencies

Documentation: https://github.com/VesperAkshay/polypm#readme

"@ -ForegroundColor Green
} elseif (-not (Test-Command "cargo")) {
    Write-Host "⚠️ Cargo (Rust package manager) not found" -ForegroundColor Yellow
    Write-Host "ℹ️ Please install Rust from https://rustup.rs/ first" -ForegroundColor Blue
    exit 1
} else {
    Write-Host "ℹ️ Installing PPM from GitHub..." -ForegroundColor Blue

    try {
        $result = Start-Process -FilePath "cargo" -ArgumentList "install", "--git", "https://github.com/VesperAkshay/polypm" -Wait -PassThru -NoNewWindow
        
        if ($result.ExitCode -eq 0) {
            Write-Host "✅ PPM installed successfully!" -ForegroundColor Green
            
            # Verify installation
            if (Test-Command "ppm") {
                $version = & ppm --version 2>$null
                Write-Host "✅ PPM $version is installed and working!" -ForegroundColor Green
            }
            
            Write-Host @"

🎉 Installation Complete!

Quick Start:
  ppm --help                    # Show help
  ppm init --name my-project    # Create new project
  ppm add react flask           # Add dependencies
  ppm install                   # Install dependencies

Documentation: https://github.com/VesperAkshay/polypm#readme

"@ -ForegroundColor Green
            
        } else {
            Write-Host "❌ Failed to install PPM" -ForegroundColor Red
            exit 1
        }
    } catch {
        Write-Host "❌ Failed to install PPM: $_" -ForegroundColor Red
        exit 1
    }
}
