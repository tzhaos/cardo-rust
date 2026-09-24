[CmdletBinding()]
param([switch]$WithUpdate)

$ErrorActionPreference = 'Stop'
$templateRoot = $PSScriptRoot
$cardoRoot = Split-Path -Parent (Split-Path -Parent $templateRoot)
$targetRoot = Join-Path $cardoRoot 'target'
$arguments = @('build', '--manifest-path', (Join-Path $templateRoot 'Cargo.toml'), '--locked', '--release', '--target', 'x86_64-pc-windows-msvc', '--target-dir', $targetRoot)
if ($WithUpdate) { $arguments += @('--features', 'update') }
& cargo @arguments
if ($LASTEXITCODE -ne 0) { throw 'Desktop template build failed' }

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $installation) { throw 'Visual C++ Build Tools are required' }
$versions = Get-ChildItem -LiteralPath (Join-Path $installation 'VC/Redist/MSVC') -Directory | Where-Object Name -match '^\d+\.\d+\.\d+$' | Sort-Object { [version]$_.Name } -Descending
$runtime = $versions | ForEach-Object {
    Get-ChildItem -Path (Join-Path $_.FullName 'x64/Microsoft.VC*.CRT/vcruntime140.dll') -File -ErrorAction SilentlyContinue
} | Select-Object -First 1
if (-not $runtime) { throw 'The x64 Visual C++ redistributable was not found' }
$output = Join-Path $targetRoot 'x86_64-pc-windows-msvc/release'
Copy-Item -LiteralPath $runtime.FullName -Destination (Join-Path $output 'vcruntime140.dll') -Force
Copy-Item -LiteralPath (Join-Path $cardoRoot 'LICENSE') -Destination (Join-Path $output 'Cardo-LICENSE.txt') -Force
Copy-Item -LiteralPath (Join-Path $cardoRoot 'licenses/lucide.txt') -Destination (Join-Path $output 'Lucide-LICENSE.txt') -Force
Copy-Item -LiteralPath (Join-Path $cardoRoot 'licenses/desktop-dependencies.txt') -Destination (Join-Path $output 'Cardo-dependencies-LICENSE.txt') -Force
Write-Host "Application: $(Join-Path $output 'cardo-desk.exe')"
