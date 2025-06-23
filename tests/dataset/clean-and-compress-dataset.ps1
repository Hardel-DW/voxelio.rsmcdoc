# > powershell -ExecutionPolicy Bypass -File clean-and-compress-dataset.ps1
param(
    [string]$DatasetPath = "mcdoc",
    [string]$OutputPath = "mcdoc.json.gz"
)

if (-not (Test-Path $DatasetPath)) {
    Write-Error "Dataset path '$DatasetPath' not found"
    exit 1
}

function Remove-Comments {
    param([string]$Content)
    if (-not $Content) { return "" }
    
    $lines = $Content -split "`r?`n"
    $cleanedLines = @()
    
    foreach ($line in $lines) {
        if ($line -match '^\s*//') { continue }
        $cleanedLines += $line
    }
    
    return ($cleanedLines -join "`n").Trim()
}

Write-Host "Processing $DatasetPath..." -ForegroundColor Green

$jsonData = @{}
$files = Get-ChildItem $DatasetPath -Recurse -Filter "*.mcdoc"

foreach ($file in $files) {
    $relativePath = $file.FullName.Substring((Resolve-Path $DatasetPath).Path.Length + 1) -replace '\\', '/'
    $content = Get-Content $file.FullName -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
    $cleaned = Remove-Comments $content
    
    if ($cleaned) {
        $jsonData[$relativePath] = $cleaned
    }
}

Write-Host "Compressing $($jsonData.Count) files..." -ForegroundColor Green

$jsonString = $jsonData | ConvertTo-Json -Depth 10 -Compress
$jsonBytes = [System.Text.Encoding]::UTF8.GetBytes($jsonString)

Add-Type -AssemblyName System.IO.Compression
$outputStream = [System.IO.File]::Create($OutputPath)
$gzipStream = New-Object System.IO.Compression.GzipStream($outputStream, [System.IO.Compression.CompressionMode]::Compress)

$gzipStream.Write($jsonBytes, 0, $jsonBytes.Length)
$gzipStream.Close()
$outputStream.Close()

$size = [math]::Round((Get-Item $OutputPath).Length / 1KB, 1)
Write-Host "Done! $OutputPath ($size KB)" -ForegroundColor Cyan