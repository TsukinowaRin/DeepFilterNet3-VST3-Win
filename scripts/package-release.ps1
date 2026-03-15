param(
    [Parameter(Mandatory = $false)]
    [string]$Version = "dev",

    [Parameter(Mandatory = $false)]
    [string]$ArtifactBase = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$bundleDir = Join-Path $repoRoot "target/bundled"
$distDir = Join-Path $repoRoot "dist"
if ([string]::IsNullOrWhiteSpace($ArtifactBase)) {
    $artifactBase = "DeepFilterNet3-VST3-Win-$Version-windows-x86_64"
} else {
    $artifactBase = $ArtifactBase
}
$stageRoot = Join-Path $distDir "stage"
$stageDir = Join-Path $stageRoot $artifactBase
$zipPath = Join-Path $distDir "$artifactBase.zip"
$shaPath = Join-Path $distDir "$artifactBase.zip.sha256"

$requiredPaths = @(
    (Join-Path $bundleDir "deepfilter-vst.vst3"),
    (Join-Path $bundleDir "deepfilter-vst.clap"),
    (Join-Path $repoRoot "README.md"),
    (Join-Path $repoRoot "README_ja.md")
)

foreach ($path in $requiredPaths) {
    if (-not (Test-Path $path)) {
        throw "Required path not found: $path"
    }
}

if (Test-Path $stageDir) {
    Remove-Item $stageDir -Recurse -Force
}

if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

if (Test-Path $shaPath) {
    Remove-Item $shaPath -Force
}

New-Item -ItemType Directory -Path $stageDir -Force | Out-Null

Copy-Item (Join-Path $bundleDir "deepfilter-vst.vst3") $stageDir -Recurse -Force
Copy-Item (Join-Path $bundleDir "deepfilter-vst.clap") $stageDir -Force
Copy-Item (Join-Path $repoRoot "README.md") $stageDir -Force
Copy-Item (Join-Path $repoRoot "README_ja.md") $stageDir -Force

foreach ($licenseName in @("LICENSE", "LICENSE-MIT", "LICENSE-APACHE")) {
    $licensePath = Join-Path $repoRoot $licenseName
    if (Test-Path $licensePath) {
        Copy-Item $licensePath $stageDir -Force
    }
}

if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir -Force | Out-Null
}

Compress-Archive -Path (Join-Path $stageDir "*") -DestinationPath $zipPath -Force

$hash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash *$(Split-Path $zipPath -Leaf)" | Set-Content -Path $shaPath -Encoding ascii

Write-Host "Created package: $zipPath"
Write-Host "Created checksum: $shaPath"
