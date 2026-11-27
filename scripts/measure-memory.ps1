# Mede memória (coluna "Memória" do Gerenciador de Tarefas = Working Set privado) e CPU
# do Open Optimizer + processos WebView2, com a janela aberta e minimizada.
# Uso (PowerShell COMO ADMINISTRADOR):  powershell -ExecutionPolicy Bypass -File scripts\measure-memory.ps1
param(
  [string]$Exe = "$env:ProgramFiles\Open Optimizer\open-optimizer.exe",
  [int]$WarmupSec = 25,
  [int]$SampleSec = 20
)

Add-Type -Namespace OO -Name Win -MemberDefinition '[DllImport("user32.dll")] public static extern bool ShowWindow(System.IntPtr h, int cmd);'

function Get-Tree([int]$rootId) {
  $all = Get-CimInstance Win32_Process
  $ids = @($rootId); $added = $true
  while ($added) {
    $added = $false
    foreach ($p in $all) { if ($ids -contains $p.ParentProcessId -and -not ($ids -contains $p.ProcessId)) { $ids += $p.ProcessId; $added = $true } }
  }
  $ids
}

function Sample([string]$label, [int]$rootId) {
  $ids = Get-Tree $rootId
  $cpu0 = (Get-Process -Id $ids -ErrorAction SilentlyContinue | Measure-Object CPU -Sum).Sum
  Start-Sleep $SampleSec
  $ids = Get-Tree $rootId
  $cpu1 = (Get-Process -Id $ids -ErrorAction SilentlyContinue | Measure-Object CPU -Sum).Sum
  $perf = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process | Where-Object { $ids -contains $_.IDProcess }
  [pscustomobject]@{
    Estado          = $label
    'Memória MB'    = [math]::Round((($perf | Measure-Object WorkingSetPrivate -Sum).Sum) / 1MB, 1)
    'Rust MB'       = [math]::Round((($perf | Where-Object IDProcess -eq $rootId).WorkingSetPrivate) / 1MB, 1)
    'CPU % (média)' = [math]::Round((($cpu1 - $cpu0) / $SampleSec) * 100 / [Environment]::ProcessorCount, 2)
  }
}

if (-not (Test-Path $Exe)) { throw "não encontrado: $Exe" }
$ver = (Get-Item $Exe).VersionInfo.ProductVersion
Get-Process open-optimizer -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2
$p = Start-Process -FilePath $Exe -PassThru
Start-Sleep $WarmupSec
$open = Sample 'aberta (Visão geral)' $p.Id
$p.Refresh()
[void][OO.Win]::ShowWindow($p.MainWindowHandle, 6)   # SW_MINIMIZE
Start-Sleep 10
$min = Sample 'minimizada' $p.Id
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
"Open Optimizer $ver"
@($open, $min) | Format-Table -AutoSize
