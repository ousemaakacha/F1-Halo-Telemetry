[CmdletBinding()]
param([switch]$SkipBuild)

$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (-not $SkipBuild) { & (Join-Path $PSScriptRoot 'Build.ps1') -Test }
$destination = Join-Path $workspace 'dist\F1-Halo-Telemetry'
New-Item -ItemType Directory -Force -Path $destination | Out-Null
Copy-Item -LiteralPath (Join-Path $workspace 'target\release\f1-halo-telemetry.exe') -Destination $destination -Force
Copy-Item -LiteralPath (Join-Path $workspace 'README.md') -Destination $destination -Force
Copy-Item -LiteralPath (Join-Path $workspace 'LICENSE') -Destination $destination -Force
$docs = Join-Path $destination 'docs'
New-Item -ItemType Directory -Force -Path $docs | Out-Null
Get-ChildItem -LiteralPath (Join-Path $workspace 'docs') -File | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination $docs -Force
}
$binaryHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $destination 'f1-halo-telemetry.exe')).Hash
if (Test-Path -LiteralPath (Join-Path $workspace 'artifacts')) {
    $reports = Join-Path $destination 'artifacts'
    New-Item -ItemType Directory -Force -Path $reports | Out-Null
    Get-ChildItem -LiteralPath (Join-Path $workspace 'artifacts') -File | Where-Object { $_.Extension -in @('.json', '.csv') } | ForEach-Object {
        Copy-Item -LiteralPath $_.FullName -Destination $reports -Force
    }
}
"$binaryHash  f1-halo-telemetry.exe" | Set-Content -LiteralPath (Join-Path $destination 'SHA256.txt') -Encoding ascii
$archive = Join-Path $workspace 'dist\F1-Halo-Telemetry.zip'
Compress-Archive -LiteralPath $destination -DestinationPath $archive -Force
$sourcePaths = @('Cargo.toml', 'Cargo.lock', '.gitignore', 'LICENSE', 'README.md', 'assets', 'src', 'tests', 'benches', 'examples', 'scripts', 'docs', 'artifacts') | ForEach-Object { Join-Path $workspace $_ }
$sourceArchive = Join-Path $workspace 'dist\F1-Halo-Telemetry-source.zip'
Compress-Archive -LiteralPath $sourcePaths -DestinationPath $sourceArchive -Force
Write-Output $archive
Write-Output $sourceArchive
