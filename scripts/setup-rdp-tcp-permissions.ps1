<#
.SYNOPSIS
    Выдать права теневого подключения группам из ТЗ RDS Shadow на RDP-Tcp.

.DESCRIPTION
    Настраивает права Query Information / Remote Control / Message / Logoff
    на подключении RDP-Tcp для трёх доменных групп:
        RDS-Shadow-View    — Query + Remote Control + Message
        RDS-Shadow-Control — Query + Remote Control
        RDS-Shadow-Logoff  — Query + Logoff

    Запускать на КАЖДОМ узле сеансов (Session Host), НЕ на брокере.
    Брокер (RD Connection Broker) права теневого подключения не раздаёт.

    Требует прав администратора. Без -Server работает локально;
    с -Server применяется удалённо (нужны права админа на целевом узле).

.EXAMPLE
    .\setup-rdp-tcp-permissions.ps1 -Domain "EXAMPLE"

.EXAMPLE
    .\setup-rdp-tcp-permissions.ps1 -Domain "EXAMPLE" -Server ts03

.EXAMPLE
    # пересоздать записи (если группы были пересозданы и SID сменился)
    .\setup-rdp-tcp-permissions.ps1 -Domain "EXAMPLE" -Force
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Domain,               # NetBIOS-имя домена (например EXAMPLE)

    [string]$ViewGroup    = "RDS-Shadow-View",
    [string]$ControlGroup = "RDS-Shadow-Control",
    [string]$LogoffGroup  = "RDS-Shadow-Logoff",

    [string]$Server = "",          # пусто = локальный узел

    [switch]$Force                 # удалить существующую запись перед выдачей
)

$ErrorActionPreference = "Stop"
$ns = 'root\CIMV2\TerminalServices'

# PermissionMask для Win32_TSAccount.ModifyPermissions
$WINSTATION_QUERY  = 0
$WINSTATION_LOGOFF = 2
$WINSTATION_SHADOW = 4
$WINSTATION_LOGON  = 5
$WINSTATION_MSG    = 7

$serverArg = @{}
if ($Server) { $serverArg.ComputerName = $Server }

function Get-TsPermissionsSetting {
    Get-CimInstance @serverArg -Namespace $ns -ClassName Win32_TSPermissionsSetting `
        -Filter "TerminalName='RDP-Tcp'"
}

function Get-TsAccount {
    param([string]$Group)
    Get-CimInstance @serverArg -Namespace $ns -ClassName Win32_TSAccount `
        -Filter "TerminalName='RDP-Tcp' AND AccountName='$Group'"
}

function Grant-TsPermissions {
    param(
        [Parameter(Mandatory = $true)][string]$Group,
        [Parameter(Mandatory = $true)][int[]]$PermIds
    )
    Write-Host "== $Group" -ForegroundColor Cyan

    if ($Force) {
        $existing = Get-TsAccount -Group $Group -ErrorAction SilentlyContinue
        if ($existing) {
            Invoke-CimMethod -InputObject $existing -MethodName Delete | Out-Null
            Write-Host "   удалена существующая запись" -ForegroundColor Yellow
        }
    }

    $tsp = Get-TsPermissionsSetting
    $add = Invoke-CimMethod -InputObject $tsp -MethodName AddAccount `
        -Arguments @{ AccountName = $Group; PermissionPreSet = 0 }
    if ($add.ReturnValue -eq 0) {
        Write-Host "   запись создана"
    } else {
        Write-Host "   запись уже существовала (AddAccount rc=$($add.ReturnValue))"
    }

    $acct = Get-TsAccount -Group $Group

    # Снять Logon, который добавляет пресет Guest (PermissionPreSet=0),
    # чтобы маска получилась чистой (без лишнего бита).
    Invoke-CimMethod -InputObject $acct -MethodName ModifyPermissions `
        -Arguments @{ PermissionMask = $WINSTATION_LOGON; Allow = $false } | Out-Null

    foreach ($id in $PermIds) {
        $r = Invoke-CimMethod -InputObject $acct -MethodName ModifyPermissions `
            -Arguments @{ PermissionMask = $id; Allow = $true }
        if ($r.ReturnValue -ne 0) {
            Write-Warning "   ModifyPermissions($id) вернул $($r.ReturnValue)"
        }
    }

    $fresh = Get-TsAccount -Group $Group
    Write-Host ("   PermissionsAllowed = {0}" -f $fresh.PermissionsAllowed)
}

Grant-TsPermissions -Group "$Domain\$ViewGroup"    -PermIds @($WINSTATION_QUERY, $WINSTATION_SHADOW, $WINSTATION_MSG)
Grant-TsPermissions -Group "$Domain\$ControlGroup" -PermIds @($WINSTATION_QUERY, $WINSTATION_SHADOW)
Grant-TsPermissions -Group "$Domain\$LogoffGroup"  -PermIds @($WINSTATION_QUERY, $WINSTATION_LOGOFF)

Write-Host ""
Write-Host "Ожидаемые маски: View=145, Control=17, Logoff=5" -ForegroundColor Green
Write-Host "Напомним: права выдаются на КАЖДОМ Session Host; политика согласия — отдельный GPO." -ForegroundColor Green
