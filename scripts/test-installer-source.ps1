param(
    [Parameter(Mandatory = $false)]
    [string]$ScriptPath = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if ([string]::IsNullOrWhiteSpace($ScriptPath)) {
    $ScriptPath = Join-Path $repoRoot "installer/deepfilter-vst.iss"
}
$source = Get-Content -Path (Resolve-Path $ScriptPath).Path -Raw

$requiredPatterns = @(
    'AppId=\{\{6ACC1E2B-2F42-40A4-8330-3930DC8B0FB8\}',
    'PrivilegesRequired=admin',
    'ArchitecturesAllowed=x64compatible',
    'ArchitecturesInstallIn64BitMode=x64compatible',
    'Uninstallable=yes',
    'UninstallDisplayName=DeepFilterNet3 VST3',
    'UsePreviousTasks=no',
    'Name: "installvst3"',
    'Name: "installclap"',
    'Description: "Install VST3 plug-in"',
    'Description: "Install CLAP plug-in"',
    'GroupDescription: "Plug-in formats:"',
    'DestDir: "\{commoncf64\}\\VST3\\deepfilter-vst\.vst3"',
    'DestDir: "\{commoncf64\}\\CLAP"',
    'Tasks: installvst3',
    'Tasks: installclap',
    'function NextButtonClick\(CurPageID: Integer\): Boolean',
    'wpSelectTasks',
    "WizardIsTaskSelected\('installvst3'\)",
    "WizardIsTaskSelected\('installclap'\)",
    'Filename: "\{uninstallexe\}"'
)

foreach ($pattern in $requiredPatterns) {
    if ($source -notmatch $pattern) {
        throw "Installer source is missing required contract: $pattern"
    }
}

foreach ($forbidden in @('[UninstallDelete]', '[InstallDelete]', 'uninsneveruninstall', 'deleteafterinstall')) {
    if ($source.Contains($forbidden, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Installer source contains forbidden deletion directive: $forbidden"
    }
}

# Task lines must default-selected: no checkedonce (unchecks after previous install) and no unchecked.
$taskNames = @('installvst3', 'installclap')
foreach ($taskName in $taskNames) {
    $taskLineMatch = [regex]::Match(
        $source,
        "(?m)^Name:\s*`"$([regex]::Escape($taskName))`"[^\r\n]*"
    )
    if (-not $taskLineMatch.Success) {
        throw "Installer source is missing task line for: $taskName"
    }
    $taskLine = $taskLineMatch.Value
    if ($taskLine -match '(?i)\bcheckedonce\b|\bunchecked\b') {
        throw "Task line for $taskName must not use checkedonce or unchecked (must default selected every wizard run): $taskLine"
    }
}

Write-Host "Installer source contract passed."
