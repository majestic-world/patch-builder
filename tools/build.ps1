#Requires -Version 7.0
# Builds the optimized release binary and publishes it as dist\Builder.exe.
# Cargo's output goes to target\logs; the console shows one line per step.
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'BuildConsole.psm1') -Force

$root = Split-Path -Parent $PSScriptRoot
$logs = Join-Path $root 'target\logs'
$binary = Join-Path $root 'target\release\patch-builder.exe'
$dist = Join-Path $root 'dist'
$published = Join-Path $dist 'Builder.exe'

# Cargo only draws its "Building [===> ] 12/34: crate" bar into a terminal unless told to;
# the spinner reads that bar from the log for a real unit counter.
$cargoEnvironment = @{ CARGO_TERM_PROGRESS_WHEN = 'always'; CARGO_TERM_PROGRESS_WIDTH = '200' }
$previousEnvironment = @{}
foreach ($name in $cargoEnvironment.Keys) {
    $previousEnvironment[$name] = [Environment]::GetEnvironmentVariable($name)
    [Environment]::SetEnvironmentVariable($name, $cargoEnvironment[$name])
}

$exitCode = 0
try {
    Initialize-BuildConsole 'Patch Builder - Release' $root
    New-Item -ItemType Directory -Force -Path $logs | Out-Null

    Start-BuildStep 'Checking toolchain'
    $cargo = Get-Command cargo -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $cargo) {
        Stop-BuildStep 'cargo not found' '' ''
        throw 'cargo is not on PATH. Install Rust from https://rustup.rs'
    }
    $version = (& $cargo.Source --version) -replace '^cargo (\S+).*$', '$1'
    if ($LASTEXITCODE -ne 0) {
        Stop-BuildStep 'cargo --version failed' '' ''
        throw "cargo --version failed with exit code $LASTEXITCODE."
    }
    Complete-BuildStep "cargo $version"

    Start-BuildStep 'Compiling release'
    $log = Join-Path $logs 'cargo-release.log'
    $cargoLog = "$log.stderr"
    $code = Invoke-BuildTool $cargo.Source @('build', '--release', '--color', 'never') $log $root ''
    if ($code -ne 0) {
        # Cargo writes diagnostics to stderr; stdout stays empty.
        $diagnosticLog = if (Test-Path -LiteralPath $cargoLog) { $cargoLog } else { $log }
        Stop-BuildStep "exit code $code" $diagnosticLog '^error(\[\w+\])?:|^-->|\(os error \d+\)'
        throw "cargo build failed with exit code $code. See $diagnosticLog"
    }
    $lines = @(Read-BuildLog $cargoLog | ForEach-Object { $_.Trim() })
    $compiled = @($lines | Where-Object { $_ -match '^Compiling ' }).Count
    $warnings = @($lines | Where-Object { $_ -match '^warning: ' -and $_ -notmatch 'generated \d+ warnings?' }).Count
    $detail = if ($compiled -eq 0) { 'up to date' } else { Format-Count $compiled 'crate' 'crates' }
    if ($warnings -gt 0) { $detail += ', ' + (Format-Count $warnings 'warning' 'warnings') }
    Complete-BuildStep $detail

    Start-BuildStep 'Publishing dist\Builder.exe'
    try {
        New-Item -ItemType Directory -Force -Path $dist | Out-Null
        Copy-Item -LiteralPath $binary -Destination $published -Force
    } catch {
        Stop-BuildStep 'copy failed' '' ''
        throw "Could not write $published (is Builder.exe running?): $($_.Exception.Message)"
    }
    $megabytes = (Get-Item -LiteralPath $published).Length / 1MB
    Complete-BuildStep ([string]::Format([Globalization.CultureInfo]::InvariantCulture, '{0:0.0} MB', $megabytes))

    Complete-BuildConsole "Built $published" $logs
} catch {
    Stop-BuildStep '' '' ''
    [Console]::Error.WriteLine("  $($_.Exception.Message)")
    $exitCode = 1
} finally {
    Close-BuildConsole
    foreach ($name in $previousEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($name, $previousEnvironment[$name])
    }
}
exit $exitCode
