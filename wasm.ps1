#!/usr/bin/env pwsh
# Build script for WASM package
Write-Host "Building WASM module..." -ForegroundColor Yellow
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "WASM build failed" -ForegroundColor Red
    exit 1
}

Write-Host "Generating JS bindings..." -ForegroundColor Yellow
wasm-bindgen --out-dir package --web --typescript target/wasm32-unknown-unknown/release/voxel_rsmcdoc.wasm

if ($LASTEXITCODE -ne 0) {
    Write-Host "JS bindings generation failed" -ForegroundColor Red
    exit 1
}

Write-Host "WASM package ready in package/ folder!" -ForegroundColor Green
Write-Host "Package size:" -ForegroundColor Cyan
Get-ChildItem package/*.wasm | ForEach-Object { 
    $sizeKB = [math]::Round($_.Length / 1024, 1)
    Write-Host "   $($_.Name): ${sizeKB}KB" -ForegroundColor White
} 