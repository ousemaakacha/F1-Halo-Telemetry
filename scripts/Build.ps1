[CmdletBinding()]
param(
    [switch]$Bootstrap,
    [switch]$Test,
    [switch]$Examples,
    [switch]$Bench
)

$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$previousPath = $env:Path
$previousFlags = $env:RUSTFLAGS
Push-Location -LiteralPath $workspace
try {
    $cargoDirectory = Join-Path $env:USERPROFILE '.cargo\bin'
    $env:Path = $cargoDirectory + ';' + $env:Path
    $cacheDirectory = Join-Path $workspace '.cache'
    New-Item -ItemType Directory -Force -Path $cacheDirectory | Out-Null
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        if (-not $Bootstrap) { throw 'Rust assente. Rieseguire con -Bootstrap oppure installare Rust da https://rust-lang.org/tools/install/.' }
        $installer = Join-Path $cacheDirectory 'rustup-init.exe'
        Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $installer
        & $installer -y --no-modify-path --profile minimal --default-toolchain stable-x86_64-pc-windows-gnu
        if ($LASTEXITCODE -ne 0) { throw 'Installazione Rust fallita.' }
    }
    $toolchain = 'stable-x86_64-pc-windows-gnu'
    & rustup run $toolchain rustc --version
    if ($LASTEXITCODE -ne 0) {
        if (-not $Bootstrap) { throw 'Toolchain GNU assente. Usare -Bootstrap. È anche possibile compilare manualmente con Rust MSVC e Windows SDK.' }
        & rustup toolchain install $toolchain --profile minimal
        if ($LASTEXITCODE -ne 0) { throw 'Installazione toolchain GNU fallita.' }
    }
    $toolsDirectory = Join-Path $cacheDirectory 'build-tools'
    $dlltool = Join-Path $toolsDirectory 'dlltool.exe'
    if (-not (Test-Path -LiteralPath $dlltool)) {
        if (-not $Bootstrap) { throw 'dlltool assente. Rieseguire con -Bootstrap per scaricare LLVM-MinGW verificato.' }
        $archivePath = Join-Path $cacheDirectory 'llvm-mingw.zip'
        $archiveUri = 'https://github.com/mstorsjo/llvm-mingw/releases/download/20261104/llvm-mingw-20261104-ucrt-x86_64.zip'
        $expectedHash = '552191B5754C49A5A20723276A6FC56541EA1922C4C50B7B669022333FC1B40F'
        if (-not (Test-Path -LiteralPath $archivePath)) { Invoke-WebRequest -Uri $archiveUri -OutFile $archivePath }
        if ((Get-FileHash -Algorithm SHA256 -LiteralPath $archivePath).Hash -ne $expectedHash) {
            throw 'SHA256 LLVM-MinGW non valido. Nessun file estratto è stato eseguito.'
        }
        New-Item -ItemType Directory -Force -Path $toolsDirectory | Out-Null
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $archive = [System.IO.Compression.ZipFile]::OpenRead($archivePath)
        try {
            foreach ($entry in $archive.Entries) {
                if ($entry.FullName -match '^llvm-mingw-[^/]+/bin/(llvm-dlltool\.exe|[^/]+\.dll)$') {
                    [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, (Join-Path $toolsDirectory $entry.Name), $true)
                }
            }
        } finally { $archive.Dispose() }
        Copy-Item -LiteralPath (Join-Path $toolsDirectory 'llvm-dlltool.exe') -Destination $dlltool -Force
    }
    $env:Path = $toolsDirectory + ';' + $env:Path
    $env:RUSTFLAGS = '-C linker=rust-lld -C linker-flavor=ld.lld'
    if ($Test) {
        & rustup component add --toolchain $toolchain rustfmt clippy
        if ($LASTEXITCODE -ne 0) { throw 'Componenti di controllo Rust non disponibili.' }
        & cargo "+$toolchain" fmt -- --check
        if ($LASTEXITCODE -ne 0) { throw 'Formattazione Rust non valida.' }
        & cargo "+$toolchain" test --locked
        if ($LASTEXITCODE -ne 0) { throw 'Test falliti.' }
        & cargo "+$toolchain" clippy --locked --all-targets -- -D warnings
        if ($LASTEXITCODE -ne 0) { throw 'Clippy fallito.' }
    }
    & cargo "+$toolchain" build --locked --release
    if ($LASTEXITCODE -ne 0) { throw 'Build Release fallita.' }
    if ($Examples) {
        & cargo "+$toolchain" build --locked --release --examples
        if ($LASTEXITCODE -ne 0) { throw 'Build strumenti di test fallita.' }
    }
    if ($Bench) {
        & cargo "+$toolchain" bench --locked --bench parser
        if ($LASTEXITCODE -ne 0) { throw 'Benchmark parser fallito.' }
    }
    Write-Output (Join-Path $workspace 'target\release\f1-halo-telemetry.exe')
} finally {
    $env:Path = $previousPath
    $env:RUSTFLAGS = $previousFlags
    Pop-Location
}
