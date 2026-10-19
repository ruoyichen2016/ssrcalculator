# Run on Windows with Rust MSVC, Visual Studio C++ Build Tools and Python 3.
[CmdletBinding()]
param([switch]$KeepBuild)
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'Run this script on Windows.' }
$repoRoot = Split-Path $PSScriptRoot -Parent
$target = 'x86_64-pc-windows-msvc'
$buildDir = Join-Path $repoRoot 'target-windows'
$previousFlags = $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS
$previousIncremental = $env:CARGO_INCREMENTAL
Push-Location $repoRoot
try {
    Get-Command cargo, rustup, python -ErrorAction Stop | Out-Null
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
    $env:CARGO_INCREMENTAL = '0'
    & rustup target add $target
    if ($LASTEXITCODE -ne 0) { throw 'Windows Rust target installation failed.' }
    & cargo build --locked --release --target $target --target-dir $buildDir --bin springsharp
    if ($LASTEXITCODE -ne 0) { throw 'CLI build failed.' }
    & cargo build --locked --release --manifest-path gui/Cargo.toml --target $target --target-dir $buildDir --bins
    if ($LASTEXITCODE -ne 0) { throw 'Desktop build failed.' }
    $binaries = Join-Path $buildDir "$target/release"
    & (Join-Path $binaries 'springsharp.exe') version
    if ($LASTEXITCODE -ne 0) { throw 'CLI startup smoke check failed.' }
    & python scripts/package-portable.py --binaries $binaries
    if ($LASTEXITCODE -ne 0) { throw 'Portable packaging failed.' }
} finally {
    if (-not $KeepBuild -and (Test-Path $buildDir)) {
        & cargo clean --target-dir $buildDir
    }
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = $previousFlags
    $env:CARGO_INCREMENTAL = $previousIncremental
    Pop-Location
}
