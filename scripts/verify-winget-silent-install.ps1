[CmdletBinding()]
param()

# Static Milestone 1X-I procedure. No tracked workflow invokes this script.
# A future execution requires a separate developer-approved exact run gate.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$ExpectedBranch = 'codex/winget-network-attribution-verification'
$ExpectedFileName = 'AI.Engine.Room_0.1.0_x64-setup.exe'
$ExpectedBytes = 2651735
$ExpectedSha256 = '6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399'
$ExpectedDisplayName = 'AI Engine Room'
$ExpectedPublisher = 'Tartanleaf.com Inc.'
$ExpectedVersion = '0.1.0'
$DownloadUrl = "https://github.com/gregweir/ai-engine-room/releases/download/v0.1.0-preview.1/$ExpectedFileName"
$InstallerPath = Join-Path $env:RUNNER_TEMP $ExpectedFileName
$InstalledDuringRun = $false
$CandidateExecuted = $false
$ObservedProcessNames = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
$ObservedConnections = [System.Collections.Generic.List[object]]::new()
$Evidence = [ordered]@{
  candidate = [ordered]@{
    file_name = $ExpectedFileName
    bytes = $ExpectedBytes
    sha256 = $ExpectedSha256
    authenticode = 'NotSigned'
  }
  runner = [ordered]@{}
  install = [ordered]@{}
  launch = [ordered]@{}
  removal = [ordered]@{}
}

function Assert-Equal {
  param(
    [Parameter(Mandatory)]$Actual,
    [Parameter(Mandatory)]$Expected,
    [Parameter(Mandatory)][string]$Label
  )

  if ($Actual -ne $Expected) {
    throw "$Label mismatch: expected '$Expected', observed '$Actual'."
  }
}

function Get-AiEngineRoomEntries {
  $RegistryPaths = @(
    'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
    'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
    'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*'
  )

  @(
    foreach ($RegistryPath in $RegistryPaths) {
      Get-ItemProperty -Path $RegistryPath -ErrorAction SilentlyContinue |
        Where-Object {
          $DisplayNameProperty = $_.PSObject.Properties['DisplayName']
          $null -ne $DisplayNameProperty -and $DisplayNameProperty.Value -eq $ExpectedDisplayName
        }
    }
  )
}

function Split-RegisteredCommand {
  param([Parameter(Mandatory)][string]$CommandLine)

  if ($CommandLine -match '^\s*"([^"]+)"\s*(.*)$') {
    return [pscustomobject]@{ FilePath = $Matches[1]; Arguments = $Matches[2].Trim() }
  }

  if ($CommandLine -match '^\s*(\S+)\s*(.*)$') {
    return [pscustomobject]@{ FilePath = $Matches[1]; Arguments = $Matches[2].Trim() }
  }

  throw "Could not parse the registered command."
}

function Get-EntryValue {
  param(
    [Parameter(Mandatory)]$Entry,
    [Parameter(Mandatory)][string]$Name
  )

  $Property = $Entry.PSObject.Properties[$Name]
  if ($null -eq $Property) {
    return $null
  }
  $Property.Value
}

function ConvertFrom-RegisteredDisplayIcon {
  param([AllowNull()][string]$DisplayIcon)

  if ([string]::IsNullOrWhiteSpace($DisplayIcon)) {
    return $null
  }

  if ($DisplayIcon -match '^\s*"([^"]+\.exe)"(?:\s*,\s*-?\d+)?\s*$') {
    return $Matches[1]
  }
  if ($DisplayIcon -match '^\s*(.+?\.exe)(?:\s*,\s*-?\d+)?\s*$') {
    return $Matches[1].Trim()
  }

  return $null
}

function Test-PathWithinDirectory {
  param(
    [Parameter(Mandatory)][string]$Path,
    [Parameter(Mandatory)][string]$Directory
  )

  $FullPath = [System.IO.Path]::GetFullPath($Path)
  $FullDirectory = [System.IO.Path]::GetFullPath($Directory).TrimEnd([char[]]@('\', '/'))
  $Prefix = "$FullDirectory$([System.IO.Path]::DirectorySeparatorChar)"
  $FullPath.StartsWith($Prefix, [System.StringComparison]::OrdinalIgnoreCase)
}

function Resolve-InstalledApplication {
  param(
    [Parameter(Mandatory)]$Entry,
    [Parameter(Mandatory)][string]$InstallLocation
  )

  $UninstallerPaths = [System.Collections.Generic.HashSet[string]]::new(
    [System.StringComparer]::OrdinalIgnoreCase
  )
  foreach ($Name in @('QuietUninstallString', 'UninstallString')) {
    $RegisteredCommand = Get-EntryValue -Entry $Entry -Name $Name
    if ($RegisteredCommand) {
      try {
        $ParsedCommand = Split-RegisteredCommand -CommandLine ([string]$RegisteredCommand)
        [void]$UninstallerPaths.Add([System.IO.Path]::GetFullPath($ParsedCommand.FilePath))
      } catch {
        # The existing uninstall validation reports malformed commands separately.
      }
    }
  }

  $DisplayIcon = Get-EntryValue -Entry $Entry -Name 'DisplayIcon'
  $DisplayIconPath = ConvertFrom-RegisteredDisplayIcon -DisplayIcon ([string]$DisplayIcon)
  $DisplayIconPathUsable = $false
  $DisplayIconWithinInstall = $false
  if ($DisplayIconPath -and (Test-Path -LiteralPath $DisplayIconPath -PathType Leaf)) {
    $DisplayIconPathUsable = $true
    $DisplayIconWithinInstall = Test-PathWithinDirectory -Path $DisplayIconPath -Directory $InstallLocation
  }

  $Records = @(
    foreach ($Executable in @(Get-ChildItem -LiteralPath $InstallLocation -Filter '*.exe' -File -Recurse)) {
      $FullPath = [System.IO.Path]::GetFullPath($Executable.FullName)
      if (-not (Test-PathWithinDirectory -Path $FullPath -Directory $InstallLocation)) {
        continue
      }
      $VersionInfo = $Executable.VersionInfo
      $IsUninstaller = $UninstallerPaths.Contains($FullPath) -or $Executable.Name -match '^(?i:unins|uninstall)'
      [pscustomobject]@{
        Path = $FullPath
        RelativePath = [System.IO.Path]::GetRelativePath($InstallLocation, $FullPath)
        Bytes = $Executable.Length
        ProductName = $VersionInfo.ProductName
        CompanyName = $VersionInfo.CompanyName
        FileDescription = $VersionInfo.FileDescription
        FileVersion = $VersionInfo.FileVersion
        IsUninstaller = $IsUninstaller
      }
    }
  )

  $ApplicationRecords = @($Records | Where-Object { -not $_.IsUninstaller })
  $ResolutionMethod = 'none'
  $Candidates = @()
  if ($DisplayIconWithinInstall) {
    $Candidates = @(
      $ApplicationRecords |
        Where-Object { $_.Path -eq [System.IO.Path]::GetFullPath($DisplayIconPath) }
    )
    if ($Candidates.Count -eq 1) {
      $ResolutionMethod = 'registered_display_icon'
    }
  }
  if ($Candidates.Count -ne 1) {
    $Candidates = @($ApplicationRecords | Where-Object { $_.ProductName -eq $ExpectedDisplayName })
    if ($Candidates.Count -eq 1) {
      $ResolutionMethod = 'unique_product_name'
    }
  }
  if ($Candidates.Count -ne 1 -and $ApplicationRecords.Count -eq 1) {
    $Candidates = @($ApplicationRecords)
    $ResolutionMethod = 'unique_non_uninstaller_executable'
  }

  [pscustomobject]@{
    Candidates = $Candidates
    ResolutionMethod = $ResolutionMethod
    DisplayIconRegistered = -not [string]::IsNullOrWhiteSpace([string]$DisplayIcon)
    DisplayIconPathUsable = $DisplayIconPathUsable
    DisplayIconResolvedWithinInstall = $DisplayIconWithinInstall
    DisplayIconResolvedOutsideInstall = $DisplayIconPathUsable -and -not $DisplayIconWithinInstall
    Inventory = @(
      $Records | ForEach-Object {
        [ordered]@{
          relative_path = $_.RelativePath
          bytes = $_.Bytes
          product_name = $_.ProductName
          company_name = $_.CompanyName
          file_description = $_.FileDescription
          file_version = $_.FileVersion
          registered_uninstaller = $_.IsUninstaller
        }
      }
    )
  }
}

function Get-ProcessTreeIds {
  param([Parameter(Mandatory)][int[]]$RootIds)

  $AllProcesses = @(Get-CimInstance Win32_Process)
  $Ids = [System.Collections.Generic.HashSet[int]]::new()
  foreach ($RootId in $RootIds) {
    [void]$Ids.Add($RootId)
  }

  $Changed = $true
  while ($Changed) {
    $Changed = $false
    foreach ($Process in $AllProcesses) {
      if ($Ids.Contains([int]$Process.ParentProcessId) -and $Ids.Add([int]$Process.ProcessId)) {
        $Changed = $true
      }
    }
  }

  @($Ids)
}

function Test-ProcessTree {
  param(
    [Parameter(Mandatory)][int[]]$RootIds,
    [Parameter(Mandatory)][string]$Phase,
    [switch]$RejectWindows
  )

  $TreeIds = @(Get-ProcessTreeIds -RootIds $RootIds)
  foreach ($ProcessId in $TreeIds) {
    $Process = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
    if ($null -eq $Process) {
      continue
    }

    [void]$ObservedProcessNames.Add($Process.ProcessName)
    $DeniedChildNames = @(
      'bitsadmin', 'cmd', 'cscript', 'curl', 'msiexec', 'powershell', 'pwsh',
      'regsvr32', 'rundll32', 'winget', 'wscript'
    )
    if ($ProcessId -notin $RootIds -and $Process.ProcessName -in $DeniedChildNames) {
      throw "$Phase started unexpected child process '$($Process.ProcessName)'."
    }
    if ($RejectWindows -and $Process.MainWindowHandle -ne 0) {
      throw "$Phase displayed a window in process '$($Process.ProcessName)'."
    }
  }

  $LoopbackOrUnspecified = @('0.0.0.0', '::', '127.0.0.1', '::1')
  $Connections = @(
    Get-NetTCPConnection -ErrorAction SilentlyContinue |
      Where-Object {
        $TreeIds -contains $_.OwningProcess -and
        $_.State -in @('SynSent', 'Established') -and
        $_.RemoteAddress -notin $LoopbackOrUnspecified
      }
  )

  foreach ($Connection in $Connections) {
    $OwnerProcess = Get-CimInstance Win32_Process -Filter "ProcessId = $($Connection.OwningProcess)" -ErrorAction SilentlyContinue
    $ParentProcess = $null
    if ($null -ne $OwnerProcess -and $OwnerProcess.ParentProcessId) {
      $ParentProcess = Get-CimInstance Win32_Process -Filter "ProcessId = $($OwnerProcess.ParentProcessId)" -ErrorAction SilentlyContinue
    }
    $ObservedConnections.Add([pscustomobject]@{
      phase = $Phase
      process_id = $Connection.OwningProcess
      process_name = if ($null -ne $OwnerProcess) { $OwnerProcess.Name } else { $null }
      process_attribution = if ($null -ne $OwnerProcess) { 'owner_pid_lookup' } else { 'unavailable' }
      process_tree_role = if ($Connection.OwningProcess -in $RootIds) { 'root' } else { 'descendant' }
      parent_process_id = if ($null -ne $OwnerProcess) { $OwnerProcess.ParentProcessId } else { $null }
      parent_process_name = if ($null -ne $ParentProcess) { $ParentProcess.Name } else { $null }
      remote_address = $Connection.RemoteAddress
      remote_port = $Connection.RemotePort
      state = [string]$Connection.State
    })
  }

  if ($Connections.Count -gt 0) {
    throw "$Phase opened a non-loopback TCP connection."
  }
}

function Wait-BoundedProcess {
  param(
    [Parameter(Mandatory)][System.Diagnostics.Process]$Process,
    [Parameter(Mandatory)][string]$Phase,
    [Parameter(Mandatory)][int]$TimeoutSeconds,
    [switch]$RejectWindows
  )

  $Stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
  while (-not $Process.HasExited) {
    Test-ProcessTree -RootIds @($Process.Id) -Phase $Phase -RejectWindows:$RejectWindows
    if ($Stopwatch.Elapsed.TotalSeconds -ge $TimeoutSeconds) {
      Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
      throw "$Phase exceeded the $TimeoutSeconds-second timeout."
    }
    Start-Sleep -Milliseconds 200
    $Process.Refresh()
  }

  Test-ProcessTree -RootIds @($Process.Id) -Phase $Phase -RejectWindows:$RejectWindows
  $Process.ExitCode
}

function Invoke-RegisteredUninstall {
  param([Parameter(Mandatory)]$Entry)

  $QuietUninstallString = Get-EntryValue -Entry $Entry -Name 'QuietUninstallString'
  $UninstallString = Get-EntryValue -Entry $Entry -Name 'UninstallString'
  $CommandLine = if ($QuietUninstallString) {
    [string]$QuietUninstallString
  } elseif ($UninstallString) {
    "$( [string]$UninstallString ) /S"
  } else {
    throw 'The installed entry has no uninstall command.'
  }

  $Command = Split-RegisteredCommand -CommandLine $CommandLine
  if (-not (Test-Path -LiteralPath $Command.FilePath -PathType Leaf)) {
    throw 'The registered uninstaller does not exist.'
  }

  if ($Command.Arguments -notmatch '(?i)(^|\s)/S(\s|$)') {
    $Command.Arguments = "$($Command.Arguments) /S".Trim()
  }

  $UninstallProcess = Start-Process -FilePath $Command.FilePath -ArgumentList $Command.Arguments -PassThru -WindowStyle Hidden
  $ExitCode = Wait-BoundedProcess -Process $UninstallProcess -Phase 'silent removal' -TimeoutSeconds 60 -RejectWindows
  Assert-Equal -Actual $ExitCode -Expected 0 -Label 'Silent-removal exit code'

  $Deadline = [DateTime]::UtcNow.AddSeconds(30)
  do {
    Start-Sleep -Milliseconds 500
    $RemainingEntries = @(Get-AiEngineRoomEntries)
  } while ($RemainingEntries.Count -gt 0 -and [DateTime]::UtcNow -lt $Deadline)

  if ($RemainingEntries.Count -gt 0) {
    throw 'The Installed Apps entry remained after silent removal.'
  }
}

function Write-Evidence {
  $Evidence.observed_process_names = @($ObservedProcessNames | Sort-Object)
  $Evidence.non_loopback_connections = @($ObservedConnections)
  $Json = $Evidence | ConvertTo-Json -Depth 8
  Write-Host $Json
  if ($env:GITHUB_STEP_SUMMARY) {
    Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY -Value "## AI Engine Room WinGet feasibility evidence`n`n$Json`n"
  }
}

try {
  Assert-Equal -Actual $env:GITHUB_ACTIONS -Expected 'true' -Label 'GitHub Actions boundary'
  Assert-Equal -Actual $env:RUNNER_ENVIRONMENT -Expected 'github-hosted' -Label 'Runner environment'
  Assert-Equal -Actual $env:RUNNER_OS -Expected 'Windows' -Label 'Runner operating system'
  Assert-Equal -Actual $env:RUNNER_ARCH -Expected 'X64' -Label 'Runner architecture'
  Assert-Equal -Actual $env:GITHUB_REPOSITORY -Expected 'gregweir/ai-engine-room' -Label 'Repository identity'
  Assert-Equal -Actual $env:GITHUB_HEAD_REF -Expected $ExpectedBranch -Label 'Approved branch'
  Assert-Equal -Actual $env:AER_WINGET_FEASIBILITY_APPROVED -Expected 'true' -Label 'Explicit workflow gate'

  $OperatingSystem = Get-CimInstance Win32_OperatingSystem
  $Evidence.runner = [ordered]@{
    environment = $env:RUNNER_ENVIRONMENT
    image_os = $env:ImageOS
    image_version = $env:ImageVersion
    caption = $OperatingSystem.Caption
    version = $OperatingSystem.Version
    build = $OperatingSystem.BuildNumber
    architecture = $env:RUNNER_ARCH
  }

  $InitialEntries = @(Get-AiEngineRoomEntries)
  Assert-Equal -Actual $InitialEntries.Count -Expected 0 -Label 'Initial Installed Apps entry count'
  if (Get-Process -Name 'aiengineroom' -ErrorAction SilentlyContinue) {
    throw 'AI Engine Room was already running in the fresh runner.'
  }

  Invoke-WebRequest -Uri $DownloadUrl -OutFile $InstallerPath
  $Installer = Get-Item -LiteralPath $InstallerPath
  Assert-Equal -Actual $Installer.Name -Expected $ExpectedFileName -Label 'Downloaded filename'
  Assert-Equal -Actual $Installer.Length -Expected $ExpectedBytes -Label 'Downloaded byte size'
  $Hash = (Get-FileHash -LiteralPath $InstallerPath -Algorithm SHA256).Hash.ToLowerInvariant()
  Assert-Equal -Actual $Hash -Expected $ExpectedSha256 -Label 'Downloaded SHA-256'
  $SignatureStatus = [string](Get-AuthenticodeSignature -LiteralPath $InstallerPath).Status
  Assert-Equal -Actual $SignatureStatus -Expected 'NotSigned' -Label 'Authenticode status'

  $CandidateExecuted = $true
  $InstallProcess = Start-Process -FilePath $InstallerPath -ArgumentList '/S' -PassThru -WindowStyle Hidden
  $InstallExitCode = Wait-BoundedProcess -Process $InstallProcess -Phase 'silent installation' -TimeoutSeconds 60 -RejectWindows
  Assert-Equal -Actual $InstallExitCode -Expected 0 -Label 'Silent-install exit code'
  $InstalledDuringRun = $true

  $InstallDeadline = [DateTime]::UtcNow.AddSeconds(15)
  do {
    $Entries = @(Get-AiEngineRoomEntries)
    if ($Entries.Count -eq 1) {
      break
    }
    Start-Sleep -Milliseconds 500
  } while ([DateTime]::UtcNow -lt $InstallDeadline)
  Assert-Equal -Actual $Entries.Count -Expected 1 -Label 'Installed Apps entry count'
  $Entry = $Entries[0]
  Assert-Equal -Actual $Entry.DisplayName -Expected $ExpectedDisplayName -Label 'Installed display name'
  Assert-Equal -Actual $Entry.Publisher -Expected $ExpectedPublisher -Label 'Installed publisher'
  Assert-Equal -Actual $Entry.DisplayVersion -Expected $ExpectedVersion -Label 'Installed version'
  $RegisteredInstallLocation = Get-EntryValue -Entry $Entry -Name 'InstallLocation'
  $QuietUninstallString = Get-EntryValue -Entry $Entry -Name 'QuietUninstallString'
  $UninstallString = Get-EntryValue -Entry $Entry -Name 'UninstallString'
  if (-not $RegisteredInstallLocation) {
    throw 'Installed Apps did not expose InstallLocation.'
  }
  if (-not $QuietUninstallString -and -not $UninstallString) {
    throw 'Installed Apps did not expose an uninstall command.'
  }

  $InstallLocation = ([string]$RegisteredInstallLocation).Trim('"')
  if (-not (Test-Path -LiteralPath $InstallLocation -PathType Container)) {
    throw 'The registered install location does not exist.'
  }
  $Resolution = Resolve-InstalledApplication -Entry $Entry -InstallLocation $InstallLocation
  $Evidence.install = [ordered]@{
    exit_code = $InstallExitCode
    display_name = $Entry.DisplayName
    publisher = $Entry.Publisher
    display_version = $Entry.DisplayVersion
    install_scope = if ($Entry.PSPath -like '*HKEY_CURRENT_USER*') { 'user' } else { 'machine' }
    quiet_uninstall_registered = [bool]$QuietUninstallString
    display_icon_registered = $Resolution.DisplayIconRegistered
    display_icon_path_usable = $Resolution.DisplayIconPathUsable
    display_icon_resolved_within_install = $Resolution.DisplayIconResolvedWithinInstall
    display_icon_resolved_outside_install = $Resolution.DisplayIconResolvedOutsideInstall
    executable_resolution_method = $Resolution.ResolutionMethod
    executable_candidate_count = $Resolution.Candidates.Count
    executable_inventory = $Resolution.Inventory
    executable_relative_path = $null
    executable_product_name = $null
    executable_company = $null
  }
  Assert-Equal -Actual $Resolution.DisplayIconResolvedOutsideInstall -Expected $false -Label 'Registered DisplayIcon outside install directory'
  Assert-Equal -Actual $Resolution.Candidates.Count -Expected 1 -Label 'Installed application executable candidate count'
  $ApplicationRecord = $Resolution.Candidates[0]
  $ApplicationPath = $ApplicationRecord.Path
  $Evidence.install['executable_relative_path'] = $ApplicationRecord.RelativePath
  $VersionInfo = (Get-Item -LiteralPath $ApplicationPath).VersionInfo
  Assert-Equal -Actual $VersionInfo.ProductName -Expected $ExpectedDisplayName -Label 'Installed executable product name'
  Assert-Equal -Actual $VersionInfo.CompanyName -Expected $ExpectedPublisher -Label 'Installed executable company'
  $Evidence.install['executable_product_name'] = $VersionInfo.ProductName
  $Evidence.install['executable_company'] = $VersionInfo.CompanyName

  $AppProcess = Start-Process -FilePath $ApplicationPath -PassThru -WindowStyle Hidden
  $LaunchStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
  while ($LaunchStopwatch.Elapsed.TotalSeconds -lt 8) {
    if ($AppProcess.HasExited) {
      throw "The application exited during bounded launch observation with code $($AppProcess.ExitCode)."
    }
    Test-ProcessTree -RootIds @($AppProcess.Id) -Phase 'bounded application launch'
    Start-Sleep -Milliseconds 250
    $AppProcess.Refresh()
  }
  $Evidence.launch = [ordered]@{
    remained_running_seconds = 8
    provider_or_inference_action = $false
    non_loopback_connection_count = $ObservedConnections.Count
  }
  $AppTreeIds = @(Get-ProcessTreeIds -RootIds @($AppProcess.Id))
  Stop-Process -Id $AppTreeIds -Force -ErrorAction SilentlyContinue
  Wait-Process -Id $AppProcess.Id -Timeout 15 -ErrorAction SilentlyContinue

  Invoke-RegisteredUninstall -Entry $Entry
  $InstalledDuringRun = $false

  if (Test-Path -LiteralPath $ApplicationPath) {
    throw 'The application executable remained after silent removal.'
  }
  if (Test-Path -LiteralPath $InstallLocation) {
    $Residue = @(Get-ChildItem -LiteralPath $InstallLocation -Force -ErrorAction SilentlyContinue)
    if ($Residue.Count -gt 0) {
      throw 'The install directory retained files after silent removal.'
    }
  }
  $ApplicationProcessName = [System.IO.Path]::GetFileNameWithoutExtension($ApplicationPath)
  if (Get-Process -Name $ApplicationProcessName -ErrorAction SilentlyContinue) {
    throw 'The application process remained after silent removal.'
  }

  $Evidence.removal = [ordered]@{
    exit_code = 0
    installed_entry_absent = $true
    executable_absent = $true
    process_absent = $true
    install_directory_empty_or_absent = $true
  }
  $Evidence.result = 'pass'
} catch {
  $Evidence.result = 'stop'
  $Evidence.stop_reason = $_.Exception.Message
  throw
} finally {
  if ($InstalledDuringRun -or ($CandidateExecuted -and $Evidence.result -eq 'stop')) {
    try {
      $CleanupEntries = @(Get-AiEngineRoomEntries)
      if ($CleanupEntries.Count -eq 1) {
        Invoke-RegisteredUninstall -Entry $CleanupEntries[0]
        $Evidence.cleanup_after_stop = 'registered silent uninstall completed'
      } elseif ($CleanupEntries.Count -eq 0) {
        $Evidence.cleanup_after_stop = 'no installed entry remained'
      } else {
        $Evidence.cleanup_after_stop = 'multiple installed entries prevented bounded cleanup'
      }
    } catch {
      $Evidence.cleanup_after_stop = "cleanup failed: $($_.Exception.Message)"
    }
  }

  Write-Evidence
  if (Test-Path -LiteralPath $InstallerPath) {
    Remove-Item -LiteralPath $InstallerPath -Force
  }
}
