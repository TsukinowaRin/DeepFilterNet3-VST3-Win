param(
    [Parameter(Mandatory = $true)]
    [string]$InstallerPath
)

$ErrorActionPreference = "Stop"

if ($env:GITHUB_ACTIONS -ne "true") {
    throw "Installer smoke test may only modify the ephemeral GitHub Actions runner."
}

$installer = (Resolve-Path $InstallerPath).Path
$vst3Path = Join-Path $env:CommonProgramFiles "VST3\deepfilter-vst.vst3"
$clapPath = Join-Path $env:CommonProgramFiles "CLAP\deepfilter-vst.clap"
$appDir = Join-Path $env:ProgramFiles "DeepFilterNet3 VST3"
$uninstallerPath = Join-Path $appDir "unins000.exe"
$uninstallRoots = @(
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
)

function Get-DeepFilterUninstallEntry {
    foreach ($root in $uninstallRoots) {
        Get-ItemProperty -Path $root -ErrorAction SilentlyContinue |
            Where-Object { $_.DisplayName -eq "DeepFilterNet3 VST3" }
    }
}

function Assert-CleanInstallState {
    foreach ($path in @($vst3Path, $clapPath, $appDir)) {
        if (Test-Path $path) {
            throw "Refusing to overwrite pre-existing installer smoke path: $path"
        }
    }

    if (Get-DeepFilterUninstallEntry) {
        throw "Refusing to overwrite a pre-existing DeepFilterNet3 VST3 uninstall entry."
    }
}

function Assert-PostUninstallClean {
    foreach ($path in @($vst3Path, $clapPath, $appDir)) {
        if (Test-Path $path) {
            throw "Uninstaller left an installed path behind: $path"
        }
    }

    if (Get-DeepFilterUninstallEntry) {
        throw "Uninstaller left its Apps & Features entry behind."
    }
}

function Invoke-SilentUninstall {
    if (-not (Test-Path $uninstallerPath)) {
        throw "Expected uninstaller is missing: $uninstallerPath"
    }

    $uninstallProcess = Start-Process -FilePath $uninstallerPath -ArgumentList @(
        "/VERYSILENT",
        "/SUPPRESSMSGBOXES",
        "/NORESTART"
    ) -Wait -PassThru
    if ($uninstallProcess.ExitCode -ne 0) {
        throw "Uninstaller exited with code $($uninstallProcess.ExitCode)."
    }
}

function Invoke-InstallCase {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [string]$Tasks,
        [Parameter(Mandatory = $true)]
        [bool]$ExpectVst3,
        [Parameter(Mandatory = $true)]
        [bool]$ExpectClap
    )

    Write-Host "Installer smoke case: $Name (TASKS=$Tasks)"
    Assert-CleanInstallState

    $installed = $false
    try {
        # /TASKS= selects only the listed named tasks (others stay unselected).
        $installProcess = Start-Process -FilePath $installer -ArgumentList @(
            "/VERYSILENT",
            "/SUPPRESSMSGBOXES",
            "/NORESTART",
            "/TASKS=`"$Tasks`""
        ) -Wait -PassThru
        if ($installProcess.ExitCode -ne 0) {
            throw "Installer exited with code $($installProcess.ExitCode) for case '$Name'."
        }
        $installed = $true

        if (-not (Test-Path $uninstallerPath)) {
            throw "Installer did not create uninstaller for case '$Name': $uninstallerPath"
        }

        if ($ExpectVst3) {
            if (-not (Test-Path $vst3Path)) {
                throw "Case '$Name' expected VST3 at $vst3Path"
            }
        }
        elseif (Test-Path $vst3Path) {
            throw "Case '$Name' must not install VST3 at $vst3Path"
        }

        if ($ExpectClap) {
            if (-not (Test-Path $clapPath)) {
                throw "Case '$Name' expected CLAP at $clapPath"
            }
        }
        elseif (Test-Path $clapPath) {
            throw "Case '$Name' must not install CLAP at $clapPath"
        }

        $entries = @(Get-DeepFilterUninstallEntry)
        if ($entries.Count -ne 1) {
            throw "Case '$Name' expected one uninstall entry, found $($entries.Count)."
        }
        if ([string]::IsNullOrWhiteSpace($entries[0].UninstallString)) {
            throw "Case '$Name' uninstall entry does not contain UninstallString."
        }
    }
    finally {
        if ($installed -and (Test-Path $uninstallerPath)) {
            Invoke-SilentUninstall
        }
    }

    Assert-PostUninstallClean
    Write-Host "Installer smoke case passed: $Name"
}

Assert-CleanInstallState

$cases = @(
    @{
        Name       = "vst3-only"
        Tasks      = "installvst3"
        ExpectVst3 = $true
        ExpectClap = $false
    },
    @{
        Name       = "clap-only"
        Tasks      = "installclap"
        ExpectVst3 = $false
        ExpectClap = $true
    },
    @{
        Name       = "both-formats"
        Tasks      = "installvst3,installclap"
        ExpectVst3 = $true
        ExpectClap = $true
    }
)

foreach ($case in $cases) {
    Invoke-InstallCase `
        -Name $case.Name `
        -Tasks $case.Tasks `
        -ExpectVst3 $case.ExpectVst3 `
        -ExpectClap $case.ExpectClap
}

Write-Host "Installer install/uninstall smoke test passed (3 format cases)."
