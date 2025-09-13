# Simple test
Write-Host "Testing PPM installer..." -ForegroundColor Green

function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

Write-Host "Checking for cargo..."
if (Test-Command "cargo") {
    Write-Host "Cargo found!" -ForegroundColor Green
    cargo --version
} else {
    Write-Host "Cargo not found" -ForegroundColor Red
}

Write-Host "Checking for ppm..."
if (Test-Command "ppm") {
    Write-Host "PPM found!" -ForegroundColor Green
    ppm --version
} else {
    Write-Host "PPM not found" -ForegroundColor Yellow
}

Write-Host "Test complete"
