param(
    [Parameter(Mandatory = $false)]
    [string]$Version = "dev",

    [Parameter(Mandatory = $false)]
    [string]$ArtifactBase = "",

    [Parameter(Mandatory = $false)]
    [string]$InnoCompiler = ""
)

$ErrorActionPreference = "Stop"

if ($Version -notmatch '^[0-9A-Za-z][0-9A-Za-z._+-]{0,63}$') {
    throw "Version contains unsupported characters: $Version"
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$bundleDir = Join-Path $repoRoot "target/bundled"
$distDir = Join-Path $repoRoot "dist"
$setupScript = Join-Path $repoRoot "installer/deepfilter-vst.iss"

if ([string]::IsNullOrWhiteSpace($ArtifactBase)) {
    $artifactBase = "DeepFilterNet3-VST3-Win-$Version-windows-x86_64"
} else {
    $artifactBase = $ArtifactBase
}

if ($artifactBase -notmatch '^[0-9A-Za-z][0-9A-Za-z._-]{0,127}$') {
    throw "ArtifactBase contains unsupported characters: $artifactBase"
}

$installerBase = "$artifactBase-setup"
$installerPath = Join-Path $distDir "$installerBase.exe"
$shaPath = "$installerPath.sha256"

$requiredPaths = @(
    (Join-Path $bundleDir "deepfilter-vst.vst3"),
    (Join-Path $bundleDir "deepfilter-vst.clap"),
    (Join-Path $repoRoot "README.md"),
    (Join-Path $repoRoot "README_ja.md"),
    $setupScript
)

foreach ($path in $requiredPaths) {
    if (-not (Test-Path $path)) {
        throw "Required path not found: $path"
    }
}

& (Join-Path $PSScriptRoot "test-installer-source.ps1") -ScriptPath $setupScript

if ([string]::IsNullOrWhiteSpace($InnoCompiler)) {
    $isccCommand = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue
    if ($null -ne $isccCommand) {
        $InnoCompiler = $isccCommand.Source
    } else {
        # Prefer machine-wide Program Files, then per-user winget/CURRENTUSER layout.
        $candidates = @(
            (Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFilesX86)) "Inno Setup 6/ISCC.exe"),
            (Join-Path $env:LOCALAPPDATA "Programs/Inno Setup 6/ISCC.exe")
        )
        foreach ($candidate in $candidates) {
            if (Test-Path $candidate) {
                $InnoCompiler = $candidate
                break
            }
        }
    }
}

if ([string]::IsNullOrWhiteSpace($InnoCompiler) -or -not (Test-Path $InnoCompiler -PathType Leaf)) {
    throw "Inno Setup compiler was not found. Install Inno Setup 6 or pass -InnoCompiler."
}
$InnoCompiler = (Resolve-Path $InnoCompiler).Path

if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir -Force | Out-Null
}

foreach ($path in @($installerPath, $shaPath)) {
    if (Test-Path $path) {
        Remove-Item $path -Force
    }
}

$isccArguments = @(
    "/Qp",
    "/DMyAppVersion=$Version",
    "/DSourceDir=$bundleDir",
    "/DRepoRoot=$repoRoot",
    "/DOutputDir=$distDir",
    "/DOutputBaseFilename=$installerBase",
    $setupScript
)

& $InnoCompiler @isccArguments
if ($LASTEXITCODE -ne 0) {
    throw "Inno Setup compiler exited with code $LASTEXITCODE."
}

if (-not (Test-Path $installerPath -PathType Leaf)) {
    throw "Inno Setup did not create the expected installer: $installerPath"
}

$hash = (Get-FileHash -Path $installerPath -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash *$(Split-Path $installerPath -Leaf)" | Set-Content -Path $shaPath -Encoding ascii

Write-Host "Created installer: $installerPath"
Write-Host "Created checksum: $shaPath"
