# PPM Uninstall Script for Windows
# Usage: powershell -ExecutionPolicy Bypass -File .\uninstall.ps1

$ErrorActionPreference = "Stop"

Write-Host @"

╔═══════════════════════════════════════╗
║       PPM Uninstaller for Windows     ║
║    Polyglot Package Manager v1.0.0    ║
╚═══════════════════════════════════════╝

"@ -ForegroundColor Cyan

Write-Host "🗑️ Starting PPM uninstallation..." -ForegroundColor Blue

# Function to check if command exists
function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

# Check if PPM is installed
if (-not (Test-Command "ppm")) {
    Write-Host "⚠️ PPM is not found in your PATH" -ForegroundColor Yellow
    Write-Host "ℹ️ It may already be uninstalled or installed in a custom location" -ForegroundColor Blue
} else {
    $ppmPath = (Get-Command ppm).Source
    Write-Host "📍 Found PPM at: $ppmPath" -ForegroundColor Green
    
    # Remove the executable
    try {
        Remove-Item $ppmPath -Force
        Write-Host "✅ Removed PPM executable" -ForegroundColor Green
    } catch {
        Write-Host "❌ Failed to remove PPM executable: $_" -ForegroundColor Red
        Write-Host "ℹ️ You may need to run this script as Administrator" -ForegroundColor Blue
        exit 1
    }
}

# Remove from PATH if it was added to user PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -and $userPath.Contains(".cargo\bin")) {
    Write-Host "ℹ️ Note: .cargo\bin is still in your PATH (may be used by other Rust tools)" -ForegroundColor Blue
    Write-Host "ℹ️ If you don't use Rust, you can remove it from your PATH manually" -ForegroundColor Blue
}

# Check for PPM project files in current directory
if (Test-Path "project.toml") {
    Write-Host "📝 Found project.toml in current directory" -ForegroundColor Yellow
    $response = Read-Host "Would you like to keep PPM project files? (Y/n)"
    if ($response -match "^(n|no|N|NO)$") {
        if (Test-Path "node_modules") {
            Write-Host "🗑️ Removing node_modules..." -ForegroundColor Blue
            Remove-Item "node_modules" -Recurse -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path ".venv") {
            Write-Host "🗑️ Removing Python virtual environment..." -ForegroundColor Blue
            Remove-Item ".venv" -Recurse -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path "ppm.lock") {
            Write-Host "🗑️ Removing lock file..." -ForegroundColor Blue
            Remove-Item "ppm.lock" -Force -ErrorAction SilentlyContinue
        }
        Write-Host "✅ Removed PPM project files" -ForegroundColor Green
    }
}

Write-Host @"

🎉 PPM Uninstallation Complete!

What was removed:
  ✅ PPM executable
  ✅ Project files (if requested)

Manual cleanup (if needed):
  • Remove .cargo\bin from PATH if you don't use Rust
  • Delete any remaining PPM project directories
  • Remove global packages cache: $env:USERPROFILE\.ppm\

Thank you for using PPM! 

"@ -ForegroundColor Green
