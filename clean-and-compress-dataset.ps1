param(
    [string]$DatasetPath = "tests/dataset",
    [string]$OutputPath = "dataset-compressed.tar.gz"
)

# Vérifier que le chemin dataset existe
if (-not (Test-Path $DatasetPath)) {
    Write-Error "Le chemin dataset '$DatasetPath' n'existe pas"
    exit 1
}

# Créer un dossier temporaire pour les fichiers nettoyés
$TempDir = "temp-cleaned-dataset"
if (Test-Path $TempDir) {
    Remove-Item $TempDir -Recurse -Force
}
New-Item -ItemType Directory -Path $TempDir | Out-Null

Write-Host "Nettoyage des commentaires /// dans les fichiers .mcdoc..." -ForegroundColor Green

# Fonction pour nettoyer un fichier
function Remove-McDocComments {
    param([string]$SourceFile, [string]$DestFile)
    
    $content = Get-Content $SourceFile -Encoding UTF8
    $cleanedContent = @()
    
    foreach ($line in $content) {
        # Ignorer les lignes qui commencent par /// (avec espaces optionnels avant)
        if ($line -notmatch '^\s*///') {
            $cleanedContent += $line
        }
    }
    
    # Créer le répertoire de destination si nécessaire
    $destDir = Split-Path $DestFile -Parent
    if (-not (Test-Path $destDir)) {
        New-Item -ItemType Directory -Path $destDir -Force | Out-Null
    }
    
    # Écrire le contenu nettoyé
    $cleanedContent | Out-File $DestFile -Encoding UTF8
}

# Copier la structure et nettoyer les fichiers .mcdoc
Get-ChildItem $DatasetPath -Recurse | ForEach-Object {
    $relativePath = $_.FullName.Substring($DatasetPath.Length + 1)
    $destPath = Join-Path $TempDir $relativePath
    
    if ($_.PSIsContainer) {
        # Créer le dossier
        if (-not (Test-Path $destPath)) {
            New-Item -ItemType Directory -Path $destPath | Out-Null
        }
    }
    else {
        # Exclure les fichiers JSON à la racine du dataset
        if ($_.Extension -eq ".json" -and $_.Directory.FullName -eq (Resolve-Path $DatasetPath).Path) {
            Write-Host "Exclusion: $relativePath (fichier JSON racine)" -ForegroundColor Red
            return
        }
        
        if ($_.Extension -eq ".mcdoc") {
            # Nettoyer les fichiers .mcdoc
            Write-Host "Nettoyage: $relativePath" -ForegroundColor Yellow
            Remove-McDocComments $_.FullName $destPath
        }
        else {
            # Copier les autres fichiers tels quels
            Write-Host "Copie: $relativePath" -ForegroundColor Cyan
            Copy-Item $_.FullName $destPath
        }
    }
}

Write-Host "Compression en format gzip..." -ForegroundColor Green

# Vérifier si tar est disponible (Windows 10/11 inclut tar)
if (Get-Command tar -ErrorAction SilentlyContinue) {
    # Utiliser tar natif de Windows
    $originalLocation = Get-Location
    try {
        Set-Location $TempDir
        tar -czf "../$OutputPath" *
        Set-Location $originalLocation
    }
    catch {
        Set-Location $originalLocation
        throw
    }
}
else {
    # Fallback: utiliser 7-Zip si disponible
    if (Get-Command 7z -ErrorAction SilentlyContinue) {
        7z a -tgzip "$OutputPath" "$TempDir\*"
    }
    else {
        Write-Warning "Ni tar ni 7-Zip ne sont disponibles. Création d'une archive ZIP à la place..."
        Compress-Archive -Path "$TempDir\*" -DestinationPath ($OutputPath -replace '\.tar\.gz$', '.zip')
        $OutputPath = $OutputPath -replace '\.tar\.gz$', '.zip'
    }
}

# Nettoyer le dossier temporaire
Remove-Item $TempDir -Recurse -Force

# Afficher les statistiques
$originalSize = (Get-ChildItem $DatasetPath -Recurse -File | Measure-Object -Property Length -Sum).Sum
$compressedSize = (Get-Item $OutputPath).Length
$compressionRatio = [math]::Round(($compressedSize / $originalSize) * 100, 2)

Write-Host "`nTerminé!" -ForegroundColor Green
Write-Host "Fichier de sortie: $OutputPath"
Write-Host "Taille originale: $([math]::Round($originalSize / 1KB, 2)) KB"
Write-Host "Taille compressée: $([math]::Round($compressedSize / 1KB, 2)) KB"
Write-Host "Ratio de compression: $compressionRatio%"
Write-Host "`nStatistiques:" -ForegroundColor Green
Write-Host "Fichiers: $(Get-ChildItem $DatasetPath -Recurse -File | Measure-Object).Count"
Write-Host "Dossiers: $(Get-ChildItem $DatasetPath -Recurse -Directory | Measure-Object).Count"
Write-Host "Taille totale: $([math]::Round($originalSize / 1MB, 2)) MB"
Write-Host "Taille compressée: $([math]::Round($compressedSize / 1MB, 2)) MB"
Write-Host "Ratio de compression: $compressionRatio%"